//! Wallpaper playback session for a single output
//!
//! A session manages the MPV player for rendering video/image wallpaper
//! on a specific Wayland output via the shared EGL context.

use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};
use wayland_client::protocol::wl_surface::WlSurface;

use crate::egl::{EglContext, EglWindow};
use crate::mpv::VideoConfig;
use crate::renderer::{create_renderer_with_properties, WallpaperRenderer};
use crate::types::OutputInfo;

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Session created but not playing
    Stopped,
    /// Actively playing
    Playing,
    /// Playback paused
    Paused,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct SessionSnapshot {
    pub path: Option<PathBuf>,
    pub state: PlaybackState,
    pub buffer_size: Option<(i32, i32)>,
    pub presented_frames: u64,
}

/// A wallpaper playback session for a single output
pub struct WallpaperSession {
    /// Output information
    output_info: OutputInfo,
    /// Path to current wallpaper
    wallpaper_path: Option<PathBuf>,
    /// Video configuration
    video_config: VideoConfig,
    initial_properties: serde_json::Value,
    /// MPV player instance
    player: Option<Box<dyn WallpaperRenderer>>,
    candidate: Option<(PathBuf, Box<dyn WallpaperRenderer>, Instant)>,
    committed: Option<PathBuf>,
    failed_source: Option<PathBuf>,
    first_frame: bool,
    redraw_requested: bool,
    terminal_failure: bool,
    initialized_at: Option<Instant>,
    /// EGL window for this surface
    egl_window: Option<Rc<EglWindow>>,
    /// Retains the GL owner through player destruction, including error exits.
    egl_context: Option<EglContext>,
    /// Current playback state
    state: PlaybackState,
    manually_paused: bool,
    power_paused: bool,
    /// Current volume (0.0 - 1.0)
    volume: f32,
    /// Whether resources are initialized
    initialized: bool,
    /// Whether OpenGL functions are loaded
    gl_loaded: bool,
    #[cfg(test)]
    presented_frames: u64,
}

impl WallpaperSession {
    pub(super) fn awaiting_first_frame(&self) -> bool {
        !self.first_frame && !self.terminal_failure
    }
    #[cfg(test)]
    pub(super) fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            path: self.wallpaper_path.clone(),
            state: self.state,
            buffer_size: self
                .egl_window
                .as_ref()
                .map(|window| (window.width(), window.height())),
            presented_frames: self.presented_frames,
        }
    }
    pub(super) fn update_output_info(&mut self, output: OutputInfo) {
        self.output_info = output;
    }
    #[cfg(test)]
    pub(super) fn surface_identity(&self) -> Option<khronos_egl::Surface> {
        self.egl_window
            .as_ref()
            .and_then(|window| window.identity())
    }

    /// Create a new wallpaper session
    pub fn new(
        wallpaper_path: PathBuf,
        output_info: OutputInfo,
        video_config: VideoConfig,
    ) -> Result<Self> {
        Self::new_with_properties(
            wallpaper_path,
            output_info,
            video_config,
            serde_json::json!({}),
        )
    }

    pub fn new_with_properties(
        wallpaper_path: PathBuf,
        output_info: OutputInfo,
        video_config: VideoConfig,
        initial_properties: serde_json::Value,
    ) -> Result<Self> {
        info!(
            "Creating WallpaperSession for {} ({}x{})",
            output_info.name, output_info.width, output_info.height
        );

        Ok(Self {
            output_info,
            wallpaper_path: Some(wallpaper_path),
            video_config,
            initial_properties,
            player: None,
            candidate: None,
            committed: None,
            failed_source: None,
            first_frame: false,
            redraw_requested: false,
            terminal_failure: false,
            initialized_at: None,
            egl_window: None,
            egl_context: None,
            state: PlaybackState::Stopped,
            manually_paused: false,
            power_paused: false,
            volume: 0.0,
            initialized: false,
            gl_loaded: false,
            #[cfg(test)]
            presented_frames: 0,
        })
    }

    /// Initialize rendering resources lazily (on first render)
    fn initialize_resources(
        &mut self,
        egl_context: &EglContext,
        egl_window: EglWindow,
    ) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        info!(
            "Initializing rendering resources for {} ({}x{})",
            self.output_info.name,
            egl_window.width(),
            egl_window.height()
        );
        info!("  ✓ EGL window created");

        // Make context current and load GL functions
        if let Err(error) = egl_context.make_current(&egl_window) {
            if let Err(cleanup_error) = egl_context.destroy_surface(&egl_window) {
                warn!(
                    "Failed to destroy EGL surface after make-current failure: {}",
                    cleanup_error
                );
            }
            return Err(error);
        }

        if !self.gl_loaded {
            egl_context.load_gl_functions();
            self.gl_loaded = true;
            info!("  ✓ OpenGL functions loaded");
        }

        // Keep the EGL window local until every initialization step succeeds.
        // If MPV or its render context fails, destroy this surface immediately
        // instead of leaving it in the session for the next retry.
        let egl_window = Rc::new(egl_window);
        let mut candidate_config = self.video_config.clone();
        candidate_config.mute = true;
        let player_result = create_renderer_with_properties(
            self.wallpaper_path
                .as_deref()
                .context("Wallpaper source missing")?,
            &candidate_config,
            &self.output_info,
            egl_context,
            egl_window.clone(),
            &self.initial_properties,
        );

        let player = match player_result {
            Ok(player) => player,
            Err(error) => {
                if let Err(cleanup_error) = egl_context.destroy_surface(&egl_window) {
                    warn!(
                        "Failed to destroy EGL surface after rendering initialization failure: {}",
                        cleanup_error
                    );
                }
                return Err(error);
            }
        };

        // Commit both resources only after the complete initialization succeeds.
        let start_paused = self.state == PlaybackState::Paused;
        self.egl_context = Some(egl_context.clone());
        self.egl_window = Some(egl_window);
        self.player = Some(player);
        self.initialized = true;
        self.initialized_at = Some(Instant::now());
        self.state = if start_paused {
            PlaybackState::Paused
        } else {
            PlaybackState::Playing
        };

        info!("✅ Session fully initialized for {}", self.output_info.name);

        Ok(())
    }

    /// Render a frame to a Wayland surface
    pub fn render_frame_to_surface(
        &mut self,
        egl_context: &EglContext,
        wl_surface: &WlSurface,
        width: i32,
        height: i32,
    ) -> Result<()> {
        if self.terminal_failure {
            return Ok(());
        }
        // Lazy initialization
        if !self.initialized {
            let window = egl_context.create_window(wl_surface, width, height)?;
            if let Err(error) = self.initialize_resources(egl_context, window) {
                self.terminal_failure = true;
                self.failed_source = self.wallpaper_path.clone();
                return Err(error);
            }
        }

        self.render_initialized_frame(egl_context, width, height)
    }

    /// Render to a native X11 window using the same MPV playback state.
    pub fn render_frame_to_x11_window(
        &mut self,
        egl_context: &EglContext,
        window: x11::xlib::Window,
        width: i32,
        height: i32,
    ) -> Result<()> {
        if self.terminal_failure {
            return Ok(());
        }
        if !self.initialized {
            let surface = egl_context.create_x11_window(window, width, height)?;
            if let Err(error) = self.initialize_resources(egl_context, surface) {
                self.terminal_failure = true;
                self.failed_source = self.wallpaper_path.clone();
                return Err(error);
            }
        }
        self.render_initialized_frame(egl_context, width, height)
    }

    fn render_initialized_frame(
        &mut self,
        egl_context: &EglContext,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let resized = self
            .egl_window
            .as_ref()
            .is_some_and(|window| window.width() != width || window.height() != height);
        if self.state != PlaybackState::Playing
            && self.first_frame
            && self.candidate.is_none()
            && !resized
            && !self.redraw_requested
        {
            return Ok(());
        }

        // Get EGL window
        let egl_window = match self.egl_window.as_ref() {
            Some(w) => w,
            None => return Ok(()),
        };

        // Resize if needed
        if egl_window.width() != width || egl_window.height() != height {
            egl_window.resize(width, height)?;
        }

        // Make context current
        egl_context.make_current(egl_window)?;

        if let Some((path, candidate, started)) = self.candidate.as_mut() {
            match candidate.render(width, height, 0) {
                Ok(true) => {
                    egl_context.swap_buffers(egl_window)?;
                    #[cfg(test)]
                    {
                        self.presented_frames += 1;
                    }
                    candidate.update_config(&self.video_config)?;
                    if self.state == PlaybackState::Paused {
                        candidate.pause()?;
                    }
                    let (path, candidate, _) = self.candidate.take().unwrap();
                    if let Some(player) = self.player.as_mut() {
                        player.close()?;
                    }
                    self.player = Some(candidate);
                    self.wallpaper_path = Some(path.clone());
                    self.committed = Some(path);
                    self.first_frame = true;
                    return Ok(());
                }
                Ok(false) if started.elapsed() < Duration::from_secs(10) => {}
                result => {
                    let message = match result {
                        Err(error) => format!("Candidate {} failed: {error}", path.display()),
                        _ => format!("Candidate {} did not produce a first frame", path.display()),
                    };
                    self.failed_source = Some(path.clone());
                    drop(self.candidate.take());
                    anyhow::bail!(message);
                }
            }
        }
        if let Some(player) = self.player.as_mut() {
            self.redraw_requested = false;
            let rendered = match player.render(width, height, 0) {
                Ok(rendered) => rendered,
                Err(error) => {
                    self.terminal_failure = true;
                    self.failed_source = self.wallpaper_path.clone();
                    return Err(error);
                }
            };
            if !rendered
                && !self.first_frame
                && self
                    .initialized_at
                    .is_some_and(|start| start.elapsed() >= Duration::from_secs(10))
            {
                self.terminal_failure = true;
                self.failed_source = self.wallpaper_path.clone();
                anyhow::bail!("Wallpaper did not produce a first frame");
            }
            if rendered {
                egl_context.swap_buffers(egl_window)?;
                #[cfg(test)]
                {
                    self.presented_frames += 1;
                }
                if !self.first_frame {
                    player.update_config(&self.video_config)?;
                    if self.state == PlaybackState::Paused {
                        player.pause()?;
                    }
                    self.first_frame = true;
                    self.committed = self.wallpaper_path.clone();
                }
            }
        }

        Ok(())
    }

    /// Render a frame (legacy method for compatibility)
    pub fn render_frame(&mut self) -> Result<()> {
        // This method is no longer used - rendering is done via render_frame_to_surface
        Ok(())
    }

    /// Change the manual pause reason without overriding power policy.
    pub fn pause(&mut self) {
        self.manually_paused = true;
        self.sync_pause();
    }

    pub fn resume(&mut self) {
        self.manually_paused = false;
        self.sync_pause();
    }

    /// Battery and tray pause reasons are independent for every renderer.
    pub fn set_power_paused(&mut self, paused: bool) {
        self.power_paused = paused;
        self.sync_pause();
    }

    fn sync_pause(&mut self) {
        let paused = self.manually_paused || self.power_paused;
        if paused && self.state != PlaybackState::Paused {
            debug!("Pausing session for {}", self.output_info.name);
            if self.first_frame {
                if let Some(player) = &mut self.player {
                    let _ = player.pause();
                }
            }
            self.state = PlaybackState::Paused;
        } else if !paused && self.state == PlaybackState::Paused {
            debug!("Resuming session for {}", self.output_info.name);
            if let Some(player) = &mut self.player {
                let _ = player.resume();
            }
            self.state = PlaybackState::Playing;
        }
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(player) = &mut self.player {
            let _ = player.set_volume(self.volume);
        }
    }

    /// Update settings supplied by Flutter without recreating the surface.
    pub fn update_config(&mut self, config: VideoConfig) {
        self.redraw_requested = true;
        self.volume = config.volume.clamp(0.0, 1.0) as f32;
        self.video_config = config;
        if let Some(player) = &mut self.player {
            let _ = player.update_config(&self.video_config);
        }
    }

    /// Load a new wallpaper without recreating the EGL surface (hot-swap)
    /// This provides seamless wallpaper transitions without flicker
    pub fn load_new_wallpaper(&mut self, path: &std::path::Path) -> Result<()> {
        self.load_with_properties(path, &serde_json::json!({}))
    }

    pub fn load_with_properties(
        &mut self,
        path: &std::path::Path,
        properties: &serde_json::Value,
    ) -> Result<()> {
        info!(
            "Hot-swapping wallpaper for {}: {}",
            self.output_info.name,
            path.display()
        );

        if self.initialized {
            let context = self
                .egl_context
                .as_ref()
                .context("Missing renderer context")?;
            let window = self
                .egl_window
                .as_ref()
                .context("Missing renderer surface")?;
            window.make_current()?;
            let mut config = self.video_config.clone();
            // Candidates produce no sound until their first frame commits.
            config.mute = true;
            let candidate = create_renderer_with_properties(
                path,
                &config,
                &self.output_info,
                context,
                window.clone(),
                properties,
            )?;
            self.candidate = Some((path.to_path_buf(), candidate, Instant::now()));
        } else {
            self.terminal_failure = false;
            self.wallpaper_path = Some(path.to_path_buf());
            self.initial_properties = properties.clone();
        }

        Ok(())
    }

    pub fn set_properties(
        &mut self,
        path: &std::path::Path,
        updates: &serde_json::Value,
    ) -> Result<()> {
        anyhow::ensure!(
            self.first_frame && self.wallpaper_path.as_deref() == Some(path),
            "Wallpaper assignment changed before property update"
        );
        anyhow::ensure!(self.candidate.is_none(), "Wallpaper replacement is pending");
        let player = self
            .player
            .as_mut()
            .context("Wallpaper renderer is not ready")?;
        anyhow::ensure!(
            player.capabilities().project_properties,
            "This renderer does not support live project properties"
        );
        self.egl_window
            .as_ref()
            .context("Missing renderer surface")?
            .make_current()?;
        player.set_properties(updates)
    }

    pub(super) fn stop_web(&mut self) -> Result<bool> {
        if self
            .candidate
            .as_ref()
            .is_some_and(|(_, renderer, _)| renderer.is_web())
        {
            if let Some(window) = &self.egl_window {
                window.make_current()?;
            }
            if let Some((_, renderer, _)) = self.candidate.as_mut() {
                renderer.close()?;
            }
            drop(self.candidate.take());
        }
        if self
            .player
            .as_ref()
            .is_some_and(|renderer| renderer.is_web())
        {
            self.release_resources()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn take_failed_source(&mut self) -> Option<PathBuf> {
        self.failed_source
            .take()
            .or_else(|| self.candidate.as_ref().map(|(path, _, _)| path.clone()))
    }

    pub(super) fn take_committed(&mut self) -> Option<PathBuf> {
        self.committed.take()
    }

    /// Get current wallpaper path
    pub fn wallpaper_path(&self) -> Option<&str> {
        if !self.first_frame || self.terminal_failure {
            return None;
        }
        self.wallpaper_path
            .as_ref()
            .map(|p| p.to_str().unwrap_or(""))
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get output name
    pub fn output_name(&self) -> &str {
        &self.output_info.name
    }

    /// Release playback before the backend destroys its native window.
    /// The original context must be current when libmpv releases GL objects.
    pub fn cleanup_egl(&mut self, _egl_context: &EglContext) {
        if let Err(error) = self.release_resources() {
            warn!(
                "Failed to clean playback for {}: {}",
                self.output_info.name, error
            );
        }
    }

    pub(super) fn release_resources(&mut self) -> Result<()> {
        if self.player.is_some() || self.candidate.is_some() {
            if let Some(window) = &self.egl_window {
                // Keep ownership if binding fails; Drop retries before releasing
                // fields, and backend cleanup must leave the native window alive.
                window.make_current()?;
            }
            if let Some((_, candidate, _)) = self.candidate.as_mut() {
                candidate.close()?;
            }
            drop(self.candidate.take());
            if let Some(player) = self.player.as_mut() {
                player.close()?;
            }
            drop(self.player.take());
        }
        // EglWindow's destructor destroys the surface before wl_egl_window.
        drop(self.egl_window.take());
        self.egl_context = None;
        self.initialized = false;
        Ok(())
    }
}

impl Drop for WallpaperSession {
    fn drop(&mut self) {
        debug!("Dropping WallpaperSession for {}", self.output_info.name);
        if let Err(error) = self.release_resources() {
            warn!(
                "Playback cleanup retry for {}: {}",
                self.output_info.name, error
            );
            // A recoverable binding failure gets a retry while the native window
            // and original context still exist. Permanent driver failure is not
            // part of the normal lifecycle contract.
            if let Err(error) = self.release_resources() {
                warn!(
                    "Playback cleanup failed for {}: {}",
                    self.output_info.name, error
                );
            }
        }
    }
}

#[cfg(test)]
mod control_tests {
    use super::*;
    use crate::renderer::RendererCapabilities;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Calls {
        paused: bool,
        volume: f32,
        muted: bool,
        loops: bool,
    }
    struct Renderer(Rc<RefCell<Calls>>, bool);
    impl WallpaperRenderer for Renderer {
        fn render(&mut self, _: i32, _: i32, _: i32) -> Result<bool> {
            Ok(true)
        }
        fn pause(&mut self) -> Result<()> {
            self.0.borrow_mut().paused = true;
            Ok(())
        }
        fn resume(&mut self) -> Result<()> {
            self.0.borrow_mut().paused = false;
            Ok(())
        }
        fn set_volume(&mut self, volume: f32) -> Result<()> {
            self.0.borrow_mut().volume = volume;
            Ok(())
        }
        fn update_config(&mut self, config: &VideoConfig) -> Result<()> {
            let mut calls = self.0.borrow_mut();
            calls.volume = config.volume as f32;
            calls.muted = config.mute;
            calls.loops = config.loop_playback;
            Ok(())
        }
        fn capabilities(&self) -> RendererCapabilities {
            RendererCapabilities {
                video_settings: self.1,
                project_properties: false,
                audio_visualization: false,
            }
        }
    }

    #[test]
    fn all_types_keep_manual_and_battery_pause_independent_and_receive_config() {
        for kind in ["video", "image", "gif", "scene", "web"] {
            let output = OutputInfo {
                name: kind.into(),
                width: 64,
                height: 32,
                scale: 1.0,
                position: (0, 0),
                active: true,
                hdr_capabilities: Default::default(),
            };
            let mut session =
                WallpaperSession::new(PathBuf::from("fixture"), output, VideoConfig::default())
                    .unwrap();
            let calls = Rc::new(RefCell::new(Calls::default()));
            session.player = Some(Box::new(Renderer(
                calls.clone(),
                !matches!(kind, "scene" | "web"),
            )));
            session.state = PlaybackState::Playing;
            session.first_frame = true;
            session.pause();
            session.set_power_paused(true);
            session.set_power_paused(false);
            assert_eq!(session.state(), PlaybackState::Paused, "{kind}");
            assert!(calls.borrow().paused);
            session.resume();
            assert!(!calls.borrow().paused);
            session.set_power_paused(true);
            session.resume();
            assert_eq!(session.state(), PlaybackState::Paused, "{kind}");
            assert!(calls.borrow().paused);
            session.update_config(VideoConfig {
                volume: 0.4,
                mute: false,
                loop_playback: false,
                ..Default::default()
            });
            assert!(
                calls.borrow().paused,
                "configuration must preserve pause for {kind}"
            );
            assert_eq!(calls.borrow().volume, 0.4);
            assert!(!calls.borrow().muted);
            assert!(!calls.borrow().loops);
            session.set_volume(0.6);
            assert_eq!(calls.borrow().volume, 0.6);
            session.set_power_paused(false);
            assert_eq!(session.state(), PlaybackState::Playing, "{kind}");
            assert!(!calls.borrow().paused);
        }
    }
}
