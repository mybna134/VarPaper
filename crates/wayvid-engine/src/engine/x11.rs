//! Native X11 desktop windows and RandR output tracking.
//!
//! All Xlib calls stay on the engine thread. EGL surfaces and MPV players are
//! destroyed before their X11 windows and the display connection.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use calloop::channel::Channel;
use tracing::{info, warn};
use x11::{xfixes, xlib, xrandr};

use super::{
    check_battery_status, EngineCommand, EngineConfig, EngineEvent, EngineStatus, WallpaperSession,
};
use crate::egl::EglContext;
use crate::types::{OutputHdrCapabilities, OutputInfo};

const SHAPE_INPUT: i32 = 2;

struct Display {
    raw: *mut xlib::Display,
    root: xlib::Window,
}

impl Display {
    fn open() -> Result<Self> {
        let raw = unsafe { xlib::XOpenDisplay(ptr::null()) };
        if raw.is_null() {
            bail!("Cannot connect to X11 DISPLAY");
        }
        let root = unsafe { xlib::XRootWindow(raw, xlib::XDefaultScreen(raw)) };
        let mut event_base = 0;
        let mut error_base = 0;
        if unsafe { xrandr::XRRQueryExtension(raw, &mut event_base, &mut error_base) } == 0 {
            unsafe {
                xlib::XCloseDisplay(raw);
            }
            bail!("X11 RandR extension is unavailable");
        }
        let (mut major, mut minor) = (0, 0);
        if unsafe { xrandr::XRRQueryVersion(raw, &mut major, &mut minor) } == 0
            || (major, minor) < (1, 5)
        {
            unsafe {
                xlib::XCloseDisplay(raw);
            }
            bail!("X11 RandR 1.5 or newer is required");
        }
        if unsafe { xfixes::XFixesQueryExtension(raw, &mut event_base, &mut error_base) } == 0 {
            unsafe {
                xlib::XCloseDisplay(raw);
            }
            bail!("X11 XFixes extension is unavailable");
        }
        Ok(Self { raw, root })
    }

    fn outputs(&self) -> Result<Vec<OutputInfo>> {
        let mut count = 0;
        let monitors = unsafe { xrandr::XRRGetMonitors(self.raw, self.root, 1, &mut count) };
        if monitors.is_null() && count == 0 {
            return Ok(Vec::new());
        }
        if monitors.is_null() {
            bail!("X11 RandR monitor query failed");
        }
        let mut outputs = Vec::new();
        for monitor in unsafe { std::slice::from_raw_parts(monitors, count.max(0) as usize) } {
            if monitor.width <= 0 || monitor.height <= 0 {
                continue;
            }
            let name_ptr = unsafe { xlib::XGetAtomName(self.raw, monitor.name) };
            let name = if name_ptr.is_null() {
                format!("X11-{}", monitor.name)
            } else {
                let name = unsafe { CStr::from_ptr(name_ptr) }
                    .to_string_lossy()
                    .into_owned();
                unsafe {
                    xlib::XFree(name_ptr.cast());
                }
                name
            };
            outputs.push(OutputInfo {
                name,
                width: monitor.width,
                height: monitor.height,
                scale: 1.0,
                position: (monitor.x, monitor.y),
                active: true,
                hdr_capabilities: OutputHdrCapabilities::default(),
            });
        }
        unsafe {
            xrandr::XRRFreeMonitors(monitors);
        }
        Ok(outputs)
    }

    fn visual_id(&self) -> i32 {
        unsafe {
            let visual = xlib::XDefaultVisual(self.raw, xlib::XDefaultScreen(self.raw));
            xlib::XVisualIDFromVisual(visual) as i32
        }
    }

    fn atom(&self, name: &str) -> xlib::Atom {
        let name = CString::new(name).expect("static atom name");
        unsafe { xlib::XInternAtom(self.raw, name.as_ptr(), 0) }
    }

    fn create_desktop_window(&self, output: &OutputInfo) -> Result<xlib::Window> {
        let window = unsafe {
            xlib::XCreateSimpleWindow(
                self.raw,
                self.root,
                output.position.0,
                output.position.1,
                output.width as u32,
                output.height as u32,
                0,
                0,
                0,
            )
        };
        if window == 0 {
            bail!("Could not create X11 window for {}", output.name);
        }
        let window_type = self.atom("_NET_WM_WINDOW_TYPE");
        let desktop_type = self.atom("_NET_WM_WINDOW_TYPE_DESKTOP");
        let desktop = self.atom("_NET_WM_DESKTOP");
        let all_desktops = u32::MAX as xlib::Atom;
        let skip_taskbar = self.atom("_NET_WM_STATE_SKIP_TASKBAR");
        let skip_pager = self.atom("_NET_WM_STATE_SKIP_PAGER");
        let state = self.atom("_NET_WM_STATE");
        unsafe {
            xlib::XChangeProperty(
                self.raw,
                window,
                window_type,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                (&desktop_type as *const xlib::Atom).cast(),
                1,
            );
            xlib::XChangeProperty(
                self.raw,
                window,
                desktop,
                xlib::XA_CARDINAL,
                32,
                xlib::PropModeReplace,
                (&all_desktops as *const xlib::Atom).cast(),
                1,
            );
            let states = [skip_taskbar, skip_pager];
            xlib::XChangeProperty(
                self.raw,
                window,
                state,
                xlib::XA_ATOM,
                32,
                xlib::PropModeReplace,
                states.as_ptr().cast(),
                states.len() as i32,
            );
            let hints = xlib::XAllocWMHints();
            if !hints.is_null() {
                (*hints).flags = xlib::InputHint;
                (*hints).input = 0;
                xlib::XSetWMHints(self.raw, window, hints);
                xlib::XFree(hints.cast());
            }
            let empty = xfixes::XFixesCreateRegion(self.raw, ptr::null_mut(), 0);
            xfixes::XFixesSetWindowShapeRegion(self.raw, window, SHAPE_INPUT, 0, 0, empty);
            xfixes::XFixesDestroyRegion(self.raw, empty);
            xlib::XMapWindow(self.raw, window);
            xlib::XFlush(self.raw);
        }
        Ok(window)
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.raw);
        }
    }
}

struct ActiveWallpaper {
    window: xlib::Window,
    session: WallpaperSession,
}

pub fn discover_outputs() -> Result<Vec<OutputInfo>> {
    Display::open()?.outputs()
}

pub(super) fn run(
    mut config: EngineConfig,
    events: Sender<EngineEvent>,
    commands: Channel<EngineCommand>,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    let display = Display::open()?;
    let egl = EglContext::new_x11(display.raw.cast(), display.visual_id())
        .context("X11 EGL initialization failed")?;
    unsafe {
        xrandr::XRRSelectInput(
            display.raw,
            display.root,
            xrandr::RRScreenChangeNotifyMask
                | xrandr::RRCrtcChangeNotifyMask
                | xrandr::RROutputChangeNotifyMask
                | xrandr::RROutputPropertyNotifyMask,
        );
    }
    let mut outputs: HashMap<String, OutputInfo> = display
        .outputs()?
        .into_iter()
        .map(|o| (o.name.clone(), o))
        .collect();
    let mut active: HashMap<String, ActiveWallpaper> = HashMap::new();
    let _ = events.send(EngineEvent::Started);
    for output in outputs.values() {
        let _ = events.send(EngineEvent::OutputAdded(output.clone()));
    }
    info!(
        "X11 wallpaper engine started with {} monitors",
        outputs.len()
    );
    let mut last_battery = Instant::now();
    let mut power_paused = false;
    while !shutdown.load(Ordering::Relaxed) {
        while let Ok(command) = commands.try_recv() {
            if matches!(command, EngineCommand::Shutdown) {
                shutdown.store(true, Ordering::Relaxed);
                break;
            }
            handle_command(
                command,
                &mut config,
                &display,
                &egl,
                &outputs,
                &mut active,
                &events,
            );
        }
        let changed = unsafe { xlib::XPending(display.raw) } > 0;
        if changed {
            while unsafe { xlib::XPending(display.raw) } > 0 {
                let mut event = std::mem::MaybeUninit::<xlib::XEvent>::uninit();
                unsafe {
                    xlib::XNextEvent(display.raw, event.as_mut_ptr());
                }
            }
            match display.outputs() {
                Ok(next) => {
                    update_outputs(next, &display, &egl, &mut outputs, &mut active, &events)
                }
                Err(error) => {
                    let _ = events.send(EngineEvent::Error(error.to_string()));
                }
            }
        }
        if last_battery.elapsed() >= Duration::from_secs(10) {
            last_battery = Instant::now();
            let on_battery = config.pause_on_battery && check_battery_status();
            if on_battery != power_paused {
                for wallpaper in active.values_mut() {
                    if on_battery {
                        wallpaper.session.pause();
                    } else {
                        wallpaper.session.resume();
                    }
                }
                power_paused = on_battery;
            }
        }
        {
            for (name, wallpaper) in &mut active {
                if power_paused {
                    wallpaper.session.pause();
                }
                if let Some(output) = outputs.get(name) {
                    if let Err(error) = wallpaper.session.render_frame_to_x11_window(
                        &egl,
                        wallpaper.window,
                        output.width,
                        output.height,
                    ) {
                        warn!("X11 render failed for {name}: {error}");
                        if let Some(path) = wallpaper.session.take_failed_source() {
                            let _ = events.send(EngineEvent::WallpaperFailed {
                                output: name.clone(),
                                path,
                                error: error.to_string(),
                            });
                        } else {
                            let _ = events.send(EngineEvent::Error(format!("{name}: {error}")));
                        }
                    }
                    if let Some(path) = wallpaper.session.take_committed() {
                        let _ = events.send(EngineEvent::WallpaperApplied {
                            output: name.clone(),
                            path,
                        });
                    }
                }
            }
        }
        let frame = config
            .fps_limit
            .filter(|fps| *fps > 0)
            .map(|fps| Duration::from_micros(1_000_000 / fps as u64))
            .unwrap_or(Duration::from_millis(16));
        std::thread::sleep(if power_paused {
            Duration::from_millis(100)
        } else {
            frame
        });
    }
    for (_, wallpaper) in active.drain() {
        destroy_wallpaper(&display, &egl, wallpaper);
    }
    #[cfg(test)]
    {
        drop(egl);
        assert_eq!(
            crate::egl::native_counts(),
            (0, 0),
            "X11 engine leaked native graphics resources"
        );
    }
    let _ = events.send(EngineEvent::Stopped);
    Ok(())
}

fn destroy_wallpaper(display: &Display, egl: &EglContext, mut wallpaper: ActiveWallpaper) {
    wallpaper.session.cleanup_egl(egl);
    // Session Drop retries a recoverable binding failure while this window is
    // still valid. No native window may die before its MPV/EGL resources.
    drop(wallpaper.session);
    unsafe {
        xlib::XDestroyWindow(display.raw, wallpaper.window);
        xlib::XFlush(display.raw);
    }
}

fn update_outputs(
    next: Vec<OutputInfo>,
    display: &Display,
    egl: &EglContext,
    outputs: &mut HashMap<String, OutputInfo>,
    active: &mut HashMap<String, ActiveWallpaper>,
    events: &Sender<EngineEvent>,
) {
    let next: HashMap<_, _> = next.into_iter().map(|o| (o.name.clone(), o)).collect();
    for name in outputs.keys().filter(|name| !next.contains_key(*name)) {
        if let Some(wallpaper) = active.remove(name) {
            destroy_wallpaper(display, egl, wallpaper);
        }
        let _ = events.send(EngineEvent::OutputRemoved(name.clone()));
    }
    for (name, output) in &next {
        match outputs.get(name) {
            None => {
                let _ = events.send(EngineEvent::OutputAdded(output.clone()));
            }
            Some(old)
                if old.position != output.position
                    || old.width != output.width
                    || old.height != output.height =>
            {
                if let Some(wallpaper) = active.get_mut(name) {
                    unsafe {
                        xlib::XMoveResizeWindow(
                            display.raw,
                            wallpaper.window,
                            output.position.0,
                            output.position.1,
                            output.width as u32,
                            output.height as u32,
                        );
                    }
                }
                let _ = events.send(EngineEvent::OutputAdded(output.clone()));
            }
            _ => {}
        }
    }
    *outputs = next;
    unsafe {
        xlib::XFlush(display.raw);
    }
}

fn handle_command(
    command: EngineCommand,
    config: &mut EngineConfig,
    display: &Display,
    egl: &EglContext,
    outputs: &HashMap<String, OutputInfo>,
    active: &mut HashMap<String, ActiveWallpaper>,
    events: &Sender<EngineEvent>,
) {
    match command {
        #[cfg(test)]
        EngineCommand::ResourceCounts(reply) => {
            let _ = reply.send(crate::egl::native_counts());
        }
        EngineCommand::ApplyWallpaper { path, output } => {
            let names: Vec<_> = output
                .map(|name| vec![name])
                .unwrap_or_else(|| outputs.keys().cloned().collect());
            for name in names {
                match apply_wallpaper(&path, &name, config, display, egl, outputs, active) {
                    Ok(()) => {}
                    Err(error) => {
                        let _ = events.send(EngineEvent::WallpaperFailed {
                            output: name,
                            path: path.clone(),
                            error: error.to_string(),
                        });
                    }
                }
            }
        }
        EngineCommand::ClearWallpaper { output } => {
            let names: Vec<_> = output
                .map(|name| vec![name])
                .unwrap_or_else(|| active.keys().cloned().collect());
            for name in names {
                if let Some(wallpaper) = active.remove(&name) {
                    destroy_wallpaper(display, egl, wallpaper);
                    let _ = events.send(EngineEvent::WallpaperCleared { output: name });
                }
            }
        }
        EngineCommand::SetVolume { output, volume } => {
            if let Some(w) = active.get_mut(&output) {
                w.session.set_volume(volume);
            }
        }
        EngineCommand::Pause { output } => {
            for_each_session(active, output, |session| session.pause())
        }
        EngineCommand::Resume { output } => {
            for_each_session(active, output, |session| session.resume())
        }
        EngineCommand::UpdateConfig(next) => {
            for wallpaper in active.values_mut() {
                wallpaper.session.update_config(next.video.clone());
            }
            *config = next;
        }
        EngineCommand::GetOutputs => {
            let _ = events.send(EngineEvent::OutputsList(
                outputs.values().cloned().collect(),
            ));
        }
        EngineCommand::GetStatus => {
            let status = EngineStatus {
                running: true,
                outputs: outputs.values().cloned().collect(),
                active_wallpapers: active
                    .iter()
                    .map(|(name, w)| (name.clone(), w.session.wallpaper_path().map(PathBuf::from)))
                    .collect(),
            };
            let _ = events.send(EngineEvent::Status(status));
        }
        EngineCommand::Shutdown => {}
    }
}

fn for_each_session(
    active: &mut HashMap<String, ActiveWallpaper>,
    output: Option<String>,
    action: impl Fn(&mut WallpaperSession),
) {
    match output {
        Some(name) => {
            if let Some(w) = active.get_mut(&name) {
                action(&mut w.session);
            }
        }
        None => {
            for w in active.values_mut() {
                action(&mut w.session);
            }
        }
    }
}

fn apply_wallpaper(
    path: &Path,
    name: &str,
    config: &EngineConfig,
    display: &Display,
    _egl: &EglContext,
    outputs: &HashMap<String, OutputInfo>,
    active: &mut HashMap<String, ActiveWallpaper>,
) -> Result<()> {
    let output = outputs
        .get(name)
        .with_context(|| format!("X11 monitor {name} not found"))?;
    if let Some(existing) = active.get_mut(name) {
        return existing.session.load_new_wallpaper(path);
    }
    let window = display.create_desktop_window(output)?;
    match WallpaperSession::new(path.to_path_buf(), output.clone(), config.video.clone()) {
        Ok(mut session) => {
            if !config.auto_play {
                session.pause();
            }
            active.insert(name.to_string(), ActiveWallpaper { window, session });
            Ok(())
        }
        Err(error) => {
            unsafe {
                xlib::XDestroyWindow(display.raw, window);
            }
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gif_renderer_produces_repeating_red_and_blue_frames() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let window = display.create_desktop_window(&output).unwrap();
        let surface = std::rc::Rc::new(egl.create_x11_window(window, 64, 64).unwrap());
        surface.make_current().unwrap();
        egl.load_gl_functions();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("colors.gif");
        // Public-domain 1x1 palette fixture: two frames, 200 ms each, red/blue.
        let mut gif = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\xff\x00\x00\x00\x00\xff".to_vec();
        gif.extend_from_slice(b"!\xff\x0bNETSCAPE2.0\x03\x01\x00\x00\x00");
        for color in [0x44, 0x4c] {
            gif.extend_from_slice(
                b"!\xf9\x04\x04\x14\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02",
            );
            gif.extend_from_slice(&[color, 1, 0]);
        }
        gif.push(b';');
        std::fs::write(&path, gif).unwrap();
        let mut renderer = crate::renderer::create_renderer(
            &path,
            &crate::mpv::VideoConfig {
                hwdec: crate::types::HwdecMode::No,
                layout: crate::types::LayoutMode::Stretch,
                ..Default::default()
            },
            &output,
            &egl,
            surface.clone(),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut transitions = Vec::new();
        while Instant::now() < deadline && transitions.len() < 4 {
            if renderer.render(64, 64, 0).unwrap() {
                let mut pixel = [0u8; 4];
                unsafe {
                    gl::ReadPixels(
                        32,
                        32,
                        1,
                        1,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        pixel.as_mut_ptr().cast(),
                    );
                }
                let color = if pixel[0] > 200 && pixel[2] < 30 {
                    Some(0)
                } else if pixel[2] > 200 && pixel[0] < 30 {
                    Some(1)
                } else {
                    None
                };
                if let Some(color) = color {
                    if transitions.last() != Some(&color) {
                        transitions.push(color);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(transitions, vec![0, 1, 0, 1], "GIF did not animate/loop");
        renderer.close().unwrap();
        drop(renderer);
        drop(surface);
        drop(egl);
        unsafe {
            xlib::XDestroyWindow(display.raw, window);
        }
    }

    #[test]
    fn candidate_scene_commits_after_frame_and_failed_replacement_keeps_source() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none()
            || std::env::var_os("VARPAPER_SCENE_LIBRARY").is_none()
        {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let baseline = crate::egl::native_counts();
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let window = display.create_desktop_window(&output).unwrap();
        let image = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
        let mut session =
            WallpaperSession::new(image.clone(), output.clone(), Default::default()).unwrap();
        assert!(session.wallpaper_path().is_none());
        let deadline = Instant::now() + Duration::from_secs(5);
        while session.take_committed().is_none() {
            assert!(Instant::now() < deadline);
            session
                .render_frame_to_x11_window(&egl, window, output.width, output.height)
                .unwrap();
            std::thread::sleep(Duration::from_millis(10));
        }
        let surface = session.surface_identity();
        let project = tempfile::TempDir::new().unwrap();
        std::fs::write(
            project.path().join("project.json"),
            r#"{"type":"scene","title":"Candidate fixture","file":"scene.json"}"#,
        )
        .unwrap();
        std::fs::write(project.path().join("scene.json"), r#"{"camera":{"center":"0 0 0","eye":"0 0 1","up":"0 1 0"},"general":{"orthogonalprojection":{"width":64,"height":64},"clearcolor":"0.25 0.5 0.75"},"objects":[]}"#).unwrap();
        session.load_new_wallpaper(project.path()).unwrap();
        assert_eq!(session.wallpaper_path(), image.to_str());
        assert!(session.take_committed().is_none());
        session
            .render_frame_to_x11_window(&egl, window, output.width, output.height)
            .unwrap();
        assert_eq!(session.take_committed().as_deref(), Some(project.path()));
        assert_eq!(session.surface_identity(), surface);
        let broken = tempfile::TempDir::new().unwrap();
        std::fs::write(
            broken.path().join("project.json"),
            r#"{"type":"scene","title":"Broken fixture","file":"scene.json"}"#,
        )
        .unwrap();
        std::fs::write(broken.path().join("scene.json"), "{}").unwrap();
        assert!(session.load_new_wallpaper(broken.path()).is_err());
        assert_eq!(session.wallpaper_path(), project.path().to_str());
        assert!(session.take_committed().is_none());
        session.pause();
        session.resume();
        session
            .render_frame_to_x11_window(&egl, window, output.width, output.height)
            .unwrap();
        destroy_wallpaper(&display, &egl, ActiveWallpaper { window, session });
        drop(egl);
        assert_eq!(crate::egl::native_counts(), baseline);
    }

    #[test]
    fn playback_initialization_failure_preserves_error_and_rolls_back() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let baseline = crate::egl::native_counts();
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let window = display.create_desktop_window(&output).unwrap();
        let mut session = WallpaperSession::new(
            PathBuf::from("invalid\0path"),
            output.clone(),
            Default::default(),
        )
        .unwrap();
        let error = session
            .render_frame_to_x11_window(&egl, window, output.width, output.height)
            .unwrap_err();
        assert!(error.to_string().contains("nul"), "{error}");
        assert_eq!(crate::egl::native_counts(), (baseline.0 + 1, baseline.1));
        assert!(session.surface_identity().is_none());
        destroy_wallpaper(&display, &egl, ActiveWallpaper { window, session });
        drop(egl);
        assert_eq!(crate::egl::native_counts(), baseline);
    }

    #[test]
    fn consecutive_session_cleanup_rebinds_and_recovers() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let baseline = crate::egl::native_counts();
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
        let mut wallpapers = Vec::new();
        for _ in 0..2 {
            let window = display.create_desktop_window(&output).unwrap();
            let mut session =
                WallpaperSession::new(path.clone(), output.clone(), Default::default()).unwrap();
            session
                .render_frame_to_x11_window(&egl, window, output.width, output.height)
                .unwrap();
            wallpapers.push(ActiveWallpaper { window, session });
        }
        egl.make_current_none().unwrap();
        crate::egl::fail_next_binding();
        assert!(wallpapers[0].session.release_resources().is_err());
        assert_eq!(
            crate::egl::native_counts(),
            (baseline.0 + 1, baseline.1 + 2)
        );
        destroy_wallpaper(&display, &egl, wallpapers.remove(0));
        let second_window = wallpapers[0].window;
        wallpapers[0]
            .session
            .render_frame_to_x11_window(&egl, second_window, output.width, output.height)
            .unwrap();
        destroy_wallpaper(&display, &egl, wallpapers.remove(0));
        assert_eq!(crate::egl::native_counts(), (baseline.0 + 1, baseline.1));
        drop(egl);
        assert_eq!(crate::egl::native_counts(), baseline);
    }

    #[test]
    fn repeated_apply_swap_clear_releases_native_resources() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let baseline = crate::egl::native_counts();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
        let (tx, rx) = std::sync::mpsc::channel();
        let mut config = EngineConfig::default();
        let outputs = HashMap::from([(output.name.clone(), output.clone())]);
        let mut active = HashMap::new();
        for restart in 0..20 {
            let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
            for _ in 0..5 {
                apply_wallpaper(
                    &path,
                    &output.name,
                    &config,
                    &display,
                    &egl,
                    &outputs,
                    &mut active,
                )
                .unwrap();
                let wallpaper = active.get_mut(&output.name).unwrap();
                let original_window = wallpaper.window;
                wallpaper
                    .session
                    .render_frame_to_x11_window(&egl, original_window, output.width, output.height)
                    .unwrap();
                let counts = crate::egl::native_counts();
                let original_surface = wallpaper.session.surface_identity();
                apply_wallpaper(
                    &path,
                    &output.name,
                    &config,
                    &display,
                    &egl,
                    &outputs,
                    &mut active,
                )
                .unwrap();
                assert_eq!(active[&output.name].window, original_window);
                assert_eq!(
                    active[&output.name].session.surface_identity(),
                    original_surface
                );
                assert_eq!(crate::egl::native_counts(), counts);
                handle_command(
                    EngineCommand::ClearWallpaper { output: None },
                    &mut config,
                    &display,
                    &egl,
                    &outputs,
                    &mut active,
                    &tx,
                );
                handle_command(
                    EngineCommand::ClearWallpaper { output: None },
                    &mut config,
                    &display,
                    &egl,
                    &outputs,
                    &mut active,
                    &tx,
                );
                assert!(active.is_empty());
                assert_eq!(crate::egl::native_counts(), (baseline.0 + 1, baseline.1));
                // Query root children instead of querying a destroyed window,
                // which would trigger Xlib's default fatal BadWindow handler.
                let (mut root, mut parent, mut children, mut count) = (0, 0, ptr::null_mut(), 0);
                unsafe {
                    assert_ne!(
                        xlib::XQueryTree(
                            display.raw,
                            display.root,
                            &mut root,
                            &mut parent,
                            &mut children,
                            &mut count
                        ),
                        0
                    );
                    if !children.is_null() {
                        assert!(!std::slice::from_raw_parts(children, count as usize)
                            .contains(&original_window));
                        xlib::XFree(children.cast());
                    }
                }
                while rx.try_recv().is_ok() {}
            }
            drop(egl);
            assert_eq!(
                crate::egl::native_counts(),
                baseline,
                "engine cycle {restart}"
            );
        }
    }

    #[test]
    fn x11_window_and_egl_lifecycle() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let outputs = display.outputs().unwrap();
        assert!(!outputs.is_empty());
        let output = &outputs[0];
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let window = display.create_desktop_window(output).unwrap();
        let surface = egl
            .create_x11_window(window, output.width, output.height)
            .unwrap();
        egl.make_current(&surface).unwrap();
        egl.swap_buffers(&surface).unwrap();
        egl.destroy_surface(&surface).unwrap();
        unsafe {
            xlib::XDestroyWindow(display.raw, window);
            xlib::XSync(display.raw, 0);
        }
    }

    #[test]
    fn x11_wallpaper_session_renders_and_cleans_up() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
        let window = display.create_desktop_window(&output).unwrap();
        let mut session = WallpaperSession::new(path, output.clone(), Default::default()).unwrap();
        session.pause();
        session
            .render_frame_to_x11_window(&egl, window, output.width, output.height)
            .unwrap();
        assert_eq!(
            session.state(),
            super::super::session::PlaybackState::Paused
        );
        session.resume();
        session
            .render_frame_to_x11_window(&egl, window, output.width, output.height)
            .unwrap();
        destroy_wallpaper(&display, &egl, ActiveWallpaper { window, session });
    }

    #[test]
    fn monitor_geometry_and_removal_release_windows() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let display = Display::open().unwrap();
        let output = display.outputs().unwrap().remove(0);
        let egl = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
        let mut outputs = HashMap::from([(output.name.clone(), output.clone())]);
        let mut active = HashMap::new();
        apply_wallpaper(
            &path,
            &output.name,
            &EngineConfig::default(),
            &display,
            &egl,
            &outputs,
            &mut active,
        )
        .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut resized = output.clone();
        resized.width -= 1;
        resized.position.0 += 1;
        update_outputs(
            vec![resized.clone()],
            &display,
            &egl,
            &mut outputs,
            &mut active,
            &tx,
        );
        assert!(
            matches!(rx.try_recv(), Ok(EngineEvent::OutputAdded(info)) if info.width == resized.width)
        );
        assert_eq!(active.len(), 1);
        update_outputs(vec![], &display, &egl, &mut outputs, &mut active, &tx);
        assert!(
            matches!(rx.try_recv(), Ok(EngineEvent::OutputRemoved(name)) if name == output.name)
        );
        assert!(active.is_empty());
    }

    #[test]
    fn x11_engine_apply_clear_and_shutdown() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let (handle, events) = super::super::spawn_engine(EngineConfig::default()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let name = loop {
            assert!(Instant::now() < deadline, "X11 output did not appear");
            match events.recv_timeout(Duration::from_millis(100)) {
                Ok(EngineEvent::OutputAdded(output)) => break output.name,
                Ok(EngineEvent::Error(error)) => panic!("X11 engine failed: {error}"),
                _ => {}
            }
        };
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
        handle
            .send(EngineCommand::ApplyWallpaper {
                path,
                output: Some(name.clone()),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline, "X11 apply event did not arrive");
            match events.recv_timeout(Duration::from_millis(100)) {
                Ok(EngineEvent::WallpaperApplied { output, .. }) if output == name => break,
                Ok(EngineEvent::Error(error)) => panic!("X11 apply failed: {error}"),
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(100));
        handle
            .send(EngineCommand::ClearWallpaper {
                output: Some(name.clone()),
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(Instant::now() < deadline, "X11 clear event did not arrive");
            match events.recv_timeout(Duration::from_millis(100)) {
                Ok(EngineEvent::WallpaperCleared { output }) if output == name => break,
                Ok(EngineEvent::Error(error)) => panic!("X11 clear failed: {error}"),
                _ => {}
            }
        }
        handle.request_shutdown();
        handle.join().unwrap();
    }
}
