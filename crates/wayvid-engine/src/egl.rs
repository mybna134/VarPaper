//! EGL context management for OpenGL rendering on Wayland and X11.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use anyhow::{anyhow, bail, Context, Result};
use khronos_egl as egl;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Proxy;
use wayland_egl as wegl;

// Native graphics objects stay on the engine thread. Windows retain their
// context, and contexts retain their display/library until native teardown ends.
thread_local! {
    #[cfg(test)]
    static NATIVE_COUNTS: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
    #[cfg(test)]
    static BIND_FAILURES: Cell<usize> = const { Cell::new(0) };
    static DISPLAYS: RefCell<HashMap<usize, Weak<EglDisplay>>> = RefCell::new(HashMap::new());
}

struct EglDisplay {
    native: usize,
    raw: egl::Display,
    instance: egl::DynamicInstance<egl::EGL1_4>,
    owns_initialization: bool,
}

impl Drop for EglDisplay {
    fn drop(&mut self) {
        if self.owns_initialization {
            if let Err(error) = self.instance.terminate(self.raw) {
                tracing::warn!("Failed to terminate engine EGL display: {}", error);
            }
        }
        DISPLAYS.with(|displays| {
            displays.borrow_mut().remove(&self.native);
        });
    }
}

struct EglOwner {
    display: Rc<EglDisplay>,
    config: egl::Config,
    context: egl::Context,
}

impl Drop for EglOwner {
    fn drop(&mut self) {
        // Never unbind a different renderer's context on this thread.
        if self.display.instance.get_current_context() == Some(self.context) {
            if let Err(error) =
                self.display
                    .instance
                    .make_current(self.display.raw, None, None, None)
            {
                tracing::warn!("Failed to unbind EGL context during cleanup: {}", error);
            }
        }
        let result = self
            .display
            .instance
            .destroy_context(self.display.raw, self.context);
        #[cfg(test)]
        if result.is_ok() {
            native_count(-1, 0);
        }
        if let Err(error) = result {
            tracing::warn!("Failed to destroy EGL context: {}", error);
        }
    }
}

/// A thread-local EGL context. Clones share native ownership; surfaces keep it alive.
#[derive(Clone)]
pub struct EglContext {
    owner: Rc<EglOwner>,
}

/// Owned EGL surface; the native Wayland/X11 window must outlive its destruction.
pub struct EglWindow {
    context: EglContext,
    egl_window: Option<wegl::WlEglSurface>,
    egl_surface: Cell<Option<egl::Surface>>,
    width: Cell<i32>,
    height: Cell<i32>,
}

impl EglContext {
    #[cfg(test)]
    pub(crate) fn is_current(&self) -> bool {
        self.owner.display.instance.get_current_context() == Some(self.owner.context)
    }

    /// Initialize EGL display and create OpenGL context
    pub fn new(wl_display: *mut std::ffi::c_void) -> Result<Self> {
        Self::new_with_visual(wl_display, None)
    }

    /// Initialize EGL for an X11 display using the screen's native visual.
    pub fn new_x11(x11_display: *mut std::ffi::c_void, visual_id: i32) -> Result<Self> {
        Self::new_with_visual(x11_display, Some(visual_id))
    }

    fn new_with_visual(
        native_display: *mut std::ffi::c_void,
        visual_id: Option<i32>,
    ) -> Result<Self> {
        let native = native_display as usize;
        let existing =
            DISPLAYS.with(|displays| displays.borrow().get(&native).and_then(Weak::upgrade));
        let display = if let Some(display) = existing {
            display
        } else {
            let instance = unsafe {
                egl::DynamicInstance::<egl::EGL1_4>::load_required()
                    .context("Failed to load EGL library")?
            };
            let raw = unsafe {
                instance
                    .get_display(native_display as egl::NativeDisplayType)
                    .context("Failed to get EGL display")?
            };
            // EGL initialization is not reference counted by eglInitialize.
            // If another client already initialized this display, borrow it
            // without ever terminating that client's display on our drop.
            let owns_initialization = instance.query_string(Some(raw), egl::VERSION).is_err();
            if owns_initialization {
                instance
                    .initialize(raw)
                    .context("Failed to initialize EGL")?;
            }
            let display = Rc::new(EglDisplay {
                native,
                raw,
                instance,
                owns_initialization,
            });
            DISPLAYS.with(|displays| {
                displays
                    .borrow_mut()
                    .insert(native, Rc::downgrade(&display));
            });
            display
        };
        // The display guard is already alive: every following ? rolls back
        // initialized resources, including config/context creation failures.
        let instance = &display.instance;

        // 4. Bind OpenGL API
        instance
            .bind_api(egl::OPENGL_API)
            .context("Failed to bind OpenGL API")?;
        #[cfg(test)]
        initialization_checkpoint(1)?;

        // 5. Choose EGL config
        let mut config_attribs = vec![
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::DEPTH_SIZE,
            24,
            egl::STENCIL_SIZE,
            8,
        ];
        if let Some(visual_id) = visual_id {
            config_attribs.extend([egl::NATIVE_VISUAL_ID, visual_id]);
        }
        config_attribs.push(egl::NONE);

        let configs = instance
            .choose_first_config(display.raw, &config_attribs)
            .context("Failed to choose EGL config")?
            .ok_or_else(|| anyhow!("No suitable EGL config found"))?;

        tracing::debug!("EGL config selected");
        #[cfg(test)]
        initialization_checkpoint(2)?;

        // Prefer the Scene shader baseline; retain the media renderer's prior
        // OpenGL 3.0 fallback on older drivers. Scene detects/rejects that target.
        let context_attributes = |minor| {
            [
                egl::CONTEXT_MAJOR_VERSION,
                3,
                egl::CONTEXT_MINOR_VERSION,
                minor,
                egl::CONTEXT_OPENGL_PROFILE_MASK,
                egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
                egl::NONE,
            ]
        };
        let context = instance
            .create_context(display.raw, configs, None, &context_attributes(3))
            .or_else(|_| {
                instance.create_context(display.raw, configs, None, &context_attributes(0))
            })
            .context("Failed to create EGL context")?;

        #[cfg(test)]
        native_count(1, 0);
        let result = Self {
            owner: Rc::new(EglOwner {
                display,
                config: configs,
                context,
            }),
        };
        #[cfg(test)]
        initialization_checkpoint(3)?;
        tracing::info!("EGL context created successfully");
        Ok(result)
    }

    /// Create EGL window surface for a Wayland surface
    pub fn create_window(
        &self,
        wl_surface: &WlSurface,
        width: i32,
        height: i32,
    ) -> Result<EglWindow> {
        // 1. Create Wayland EGL window
        let surface_id = wl_surface.id();

        let egl_window = wegl::WlEglSurface::new(surface_id, width, height)
            .context("Failed to create wl_egl_window")?;

        // 2. Create EGL window surface
        let egl_surface = unsafe {
            self.owner
                .display
                .instance
                .create_window_surface(
                    self.owner.display.raw,
                    self.owner.config,
                    egl_window.ptr() as egl::NativeWindowType,
                    None,
                )
                .context("Failed to create EGL window surface")?
        };

        tracing::debug!("EGL window surface created: {}x{}", width, height);

        #[cfg(test)]
        native_count(0, 1);
        Ok(EglWindow {
            egl_window: Some(egl_window),
            context: self.clone(),
            egl_surface: Cell::new(Some(egl_surface)),
            width: Cell::new(width),
            height: Cell::new(height),
        })
    }

    /// Create an EGL surface for an X11 window. The X11 window must outlive this surface.
    pub fn create_x11_window(
        &self,
        window: x11::xlib::Window,
        width: i32,
        height: i32,
    ) -> Result<EglWindow> {
        let egl_surface = unsafe {
            self.owner
                .display
                .instance
                .create_window_surface(
                    self.owner.display.raw,
                    self.owner.config,
                    window as egl::NativeWindowType,
                    None,
                )
                .context("Failed to create X11 EGL window surface")?
        };
        #[cfg(test)]
        native_count(0, 1);
        Ok(EglWindow {
            egl_window: None,
            context: self.clone(),
            egl_surface: Cell::new(Some(egl_surface)),
            width: Cell::new(width),
            height: Cell::new(height),
        })
    }

    /// Make this context current for rendering
    pub fn make_current(&self, window: &EglWindow) -> Result<()> {
        #[cfg(test)]
        if BIND_FAILURES.with(|failures| {
            let count = failures.get();
            failures.set(count.saturating_sub(1));
            count > 0
        }) {
            bail!("Injected recoverable EGL binding failure");
        }
        let surface = self.checked_surface(window)?;
        self.owner
            .display
            .instance
            .make_current(
                self.owner.display.raw,
                Some(surface),
                Some(surface),
                Some(self.owner.context),
            )
            .context("Failed to make EGL context current")?;
        Ok(())
    }

    /// Unbind the current EGL context (make no context current)
    pub fn make_current_none(&self) -> Result<()> {
        self.owner
            .display
            .instance
            .make_current(self.owner.display.raw, None, None, None)
            .context("Failed to unbind EGL context")?;
        Ok(())
    }

    fn checked_surface(&self, window: &EglWindow) -> Result<egl::Surface> {
        if !Rc::ptr_eq(&self.owner, &window.context.owner) {
            bail!("EGL surface belongs to another context");
        }
        window
            .egl_surface
            .get()
            .context("EGL surface has already been released")
    }

    /// Destroy a surface once. Dropping the window afterwards is safe.
    pub fn destroy_surface(&self, window: &EglWindow) -> Result<()> {
        if !Rc::ptr_eq(&self.owner, &window.context.owner) {
            bail!("EGL surface belongs to another context");
        }
        window.release()
    }

    /// Swap buffers to display rendered frame
    pub fn swap_buffers(&self, window: &EglWindow) -> Result<()> {
        let surface = self.checked_surface(window)?;
        self.owner
            .display
            .instance
            .swap_buffers(self.owner.display.raw, surface)
            .context("Failed to swap EGL buffers")?;
        Ok(())
    }

    /// Get OpenGL function address (for loading GL functions)
    pub fn get_proc_address(&self, name: &str) -> *const std::ffi::c_void {
        self.owner
            .display
            .instance
            .get_proc_address(name)
            .map(|f| f as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    }

    /// Load OpenGL functions using this context
    pub fn load_gl_functions(&self) {
        gl::load_with(|s| self.get_proc_address(s));
    }
}

impl EglWindow {
    #[cfg(test)]
    pub(crate) fn identity(&self) -> Option<egl::Surface> {
        self.egl_surface.get()
    }

    /// Bind this surface's original context, including during session teardown.
    pub fn make_current(&self) -> Result<()> {
        self.context.make_current(self)
    }

    fn release(&self) -> Result<()> {
        let Some(surface) = self.egl_surface.get() else {
            return Ok(());
        };
        let owner = &self.context.owner;
        let instance = &owner.display.instance;
        let unbind = if instance.get_current_context() == Some(owner.context)
            && (instance.get_current_surface(egl::DRAW) == Some(surface)
                || instance.get_current_surface(egl::READ) == Some(surface))
        {
            instance
                .make_current(owner.display.raw, None, None, None)
                .context("Failed to unbind EGL surface")
        } else {
            Ok(())
        };
        // Still attempt destruction if unbinding failed. A failed destruction
        // leaves the handle owned, so Drop can retry rather than forgetting it.
        instance
            .destroy_surface(owner.display.raw, surface)
            .context("Failed to destroy EGL surface")?;
        self.egl_surface.set(None);
        #[cfg(test)]
        native_count(0, -1);
        unbind
    }

    /// Get window width
    pub fn width(&self) -> i32 {
        self.width.get()
    }

    /// Get window height
    pub fn height(&self) -> i32 {
        self.height.get()
    }

    /// Resize the EGL window
    pub fn resize(&self, width: i32, height: i32) -> Result<()> {
        if self.egl_surface.get().is_none() {
            bail!("Cannot resize a released EGL surface");
        }
        if self.width.get() == width && self.height.get() == height {
            return Ok(());
        }

        if let Some(window) = &self.egl_window {
            window.resize(width, height, 0, 0);
        }
        self.width.set(width);
        self.height.set(height);

        tracing::debug!("EGL window resized to {}x{}", width, height);
        Ok(())
    }
}

impl Drop for EglWindow {
    fn drop(&mut self) {
        if let Err(error) = self.release() {
            tracing::warn!("Failed to release EGL window: {}", error);
        }
    }
}

#[cfg(test)]
thread_local! {
    static INITIALIZATION_FAILURE: Cell<u8> = const { Cell::new(0) };
}

#[cfg(test)]
fn initialization_checkpoint(stage: u8) -> Result<()> {
    if INITIALIZATION_FAILURE.with(|failure| failure.get()) == stage {
        bail!("Injected EGL initialization failure at stage {}", stage);
    }
    Ok(())
}

#[cfg(test)]
fn native_count(contexts: isize, surfaces: isize) {
    NATIVE_COUNTS.with(|counts| {
        let (c, s) = counts.get();
        counts.set((
            (c as isize + contexts) as usize,
            (s as isize + surfaces) as usize,
        ));
    });
}

#[cfg(test)]
pub(crate) fn native_counts() -> (usize, usize) {
    NATIVE_COUNTS.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn fail_next_binding() {
    BIND_FAILURES.with(|failures| failures.set(1));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use x11::xlib;

    struct NativeWindow {
        display: *mut xlib::Display,
        window: xlib::Window,
        visual: i32,
    }

    impl NativeWindow {
        fn new() -> Self {
            unsafe {
                let display = xlib::XOpenDisplay(ptr::null());
                assert!(!display.is_null(), "Xvfb/X11 display is required");
                let screen = xlib::XDefaultScreen(display);
                let visual =
                    xlib::XVisualIDFromVisual(xlib::XDefaultVisual(display, screen)) as i32;
                let window = xlib::XCreateSimpleWindow(
                    display,
                    xlib::XRootWindow(display, screen),
                    0,
                    0,
                    64,
                    64,
                    0,
                    0,
                    0,
                );
                Self {
                    display,
                    window,
                    visual,
                }
            }
        }

        fn context(&self) -> EglContext {
            EglContext::new_x11(self.display.cast(), self.visual).unwrap()
        }
    }

    impl Drop for NativeWindow {
        fn drop(&mut self) {
            unsafe {
                xlib::XDestroyWindow(self.display, self.window);
                xlib::XCloseDisplay(self.display);
            }
        }
    }

    #[test]
    fn native_surface_drop_and_explicit_release_destroy_handles_once() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let native = NativeWindow::new();
        let context = native.context();
        let surface = context.create_x11_window(native.window, 64, 64).unwrap();
        let raw = surface.egl_surface.get().unwrap();
        let display = &context.owner.display;
        context.make_current(&surface).unwrap();
        drop(surface);
        assert!(display
            .instance
            .query_surface(display.raw, raw, egl::WIDTH)
            .is_err());
        assert_eq!(display.instance.get_current_context(), None);
        let surface = context.create_x11_window(native.window, 64, 64).unwrap();
        let raw = surface.egl_surface.get().unwrap();
        context.destroy_surface(&surface).unwrap();
        context.destroy_surface(&surface).unwrap();
        assert!(context.make_current(&surface).is_err());
        assert!(context.swap_buffers(&surface).is_err());
        drop(surface);
        assert!(display
            .instance
            .query_surface(display.raw, raw, egl::WIDTH)
            .is_err());
        assert!(display
            .instance
            .query_context(display.raw, context.owner.context, egl::CONFIG_ID)
            .is_ok());
    }

    #[test]
    fn surfaces_keep_context_alive_and_foreign_contexts_are_rejected() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let native = NativeWindow::new();
        let context = native.context();
        let peer = native.context();
        let surface = context.create_x11_window(native.window, 64, 64).unwrap();
        let raw_context = context.owner.context;
        let owner = Rc::downgrade(&context.owner);
        assert!(peer.make_current(&surface).is_err());
        assert!(peer.destroy_surface(&surface).is_err());
        drop(context);
        assert!(owner.upgrade().is_some());
        surface.make_current().unwrap();
        drop(surface);
        assert!(owner.upgrade().is_none());
        let display = &peer.owner.display;
        // A second context keeps this display initialized: termination cannot
        // disguise a missing context destructor in this assertion.
        assert!(display
            .instance
            .query_context(display.raw, raw_context, egl::CONFIG_ID)
            .is_err());
        let peer_surface = peer.create_x11_window(native.window, 64, 64).unwrap();
        peer.make_current(&peer_surface).unwrap();
        peer.swap_buffers(&peer_surface).unwrap();
    }

    #[test]
    fn initialization_failures_release_display_and_context_guards() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let native = NativeWindow::new();
        let query = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required().unwrap() };
        let raw = unsafe { query.get_display(native.display.cast()).unwrap() };
        for stage in 1..=3 {
            INITIALIZATION_FAILURE.with(|failure| failure.set(stage));
            let result = EglContext::new_x11(native.display.cast(), native.visual);
            INITIALIZATION_FAILURE.with(|failure| failure.set(0));
            assert!(result.is_err());
            assert!(query.query_string(Some(raw), egl::VERSION).is_err());
            DISPLAYS.with(|displays| {
                assert!(!displays.borrow().contains_key(&(native.display as usize)))
            });
        }
        let context = native.context();
        drop(context);
        assert!(query.query_string(Some(raw), egl::VERSION).is_err());
    }

    #[test]
    fn externally_initialized_display_survives_engine_teardown() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let native = NativeWindow::new();
        let query = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required().unwrap() };
        let raw = unsafe { query.get_display(native.display.cast()).unwrap() };
        query.initialize(raw).unwrap();
        let ui = native.context();
        let ui_surface = ui.create_x11_window(native.window, 64, 64).unwrap();
        let engine = native.context();
        assert!(!engine.owner.display.owns_initialization);
        ui.make_current(&ui_surface).unwrap();
        drop(engine);
        ui.swap_buffers(&ui_surface).unwrap();
        drop(ui_surface);
        drop(ui);
        assert!(query.query_string(Some(raw), egl::VERSION).is_ok());
        query.terminate(raw).unwrap();
    }
}
