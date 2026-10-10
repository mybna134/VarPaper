//! Scene C ABI owner. The output surface and library outlive native destruction.

use std::ffi::{c_char, c_void, CStr, CString};
use std::mem::ManuallyDrop;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::ptr::NonNull;
use std::rc::Rc;

use anyhow::{bail, ensure, Context, Result};
use libloading::Library;

use crate::egl::{EglContext, EglWindow};

/// Project-scoped reader supplied by the application. Paths are relative;
/// implementations must validate resource budgets and directory confinement.
pub trait SceneResources {
    fn exists(&self, name: &str) -> Result<bool>;
    fn read(&self, name: &str) -> Result<Vec<u8>>;
}

#[repr(C)]
struct Host {
    userdata: *mut c_void,
    exists: unsafe extern "C" fn(*mut c_void, *const c_char) -> i32,
    read: unsafe extern "C" fn(*mut c_void, *const c_char, *mut *const u8, *mut usize) -> i32,
    release: unsafe extern "C" fn(*mut c_void, *const u8, usize),
    gl_proc: unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
}

#[repr(C)]
struct Config {
    manifest_json: *const c_char,
    width: u32,
    height: u32,
    scaling: u32,
    volume: f32,
    muted: i32,
}

type Create =
    unsafe extern "C" fn(*const Host, *const Config, *mut *mut c_void, *mut c_char, usize) -> i32;
type Render = unsafe extern "C" fn(
    *mut c_void,
    u32,
    u32,
    u32,
    f64,
    f64,
    *const f32,
    *mut c_char,
    usize,
) -> i32;
type Audio = unsafe extern "C" fn(*mut c_void, i32, i32, f32, *mut c_char, usize) -> i32;
type Destroy = unsafe extern "C" fn(*mut c_void, *mut c_char, usize) -> i32;

struct Api {
    _library: Library,
    create: Create,
    render: Render,
    audio: Audio,
    destroy: Destroy,
}

impl Api {
    fn load(path: &Path) -> Result<Self> {
        // Only an explicit installed library path is accepted. Symbols remain
        // valid because this owner retains the library until all handles drop.
        unsafe {
            let library = Library::new(path).context("Cannot load Scene native library")?;
            let version =
                library.get::<unsafe extern "C" fn() -> u32>(b"vp_scene_abi_version\0")?;
            ensure!(version() == 1, "Unsupported Scene native ABI version");
            Ok(Self {
                create: *library.get(b"vp_scene_create\0")?,
                render: *library.get(b"vp_scene_render\0")?,
                audio: *library.get(b"vp_scene_set_audio\0")?,
                destroy: *library.get(b"vp_scene_destroy\0")?,
                _library: library,
            })
        }
    }
}

struct HostData {
    resources: Box<dyn SceneResources>,
    context: EglContext,
}

unsafe extern "C" fn exists(data: *mut c_void, name: *const c_char) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        let data = &*(data.cast::<HostData>());
        let name = CStr::from_ptr(name).to_str()?;
        Ok::<_, anyhow::Error>(i32::from(data.resources.exists(name)?))
    }))
    .ok()
    .and_then(Result::ok)
    .unwrap_or(0)
}

unsafe extern "C" fn read(
    data: *mut c_void,
    name: *const c_char,
    out: *mut *const u8,
    length: *mut usize,
) -> i32 {
    *out = std::ptr::null();
    *length = 0;
    catch_unwind(AssertUnwindSafe(|| {
        let data = &*(data.cast::<HostData>());
        let name = CStr::from_ptr(name).to_str()?;
        let bytes = data.resources.read(name)?;
        ensure!(
            bytes.len() <= 256 * 1024 * 1024,
            "Scene resource exceeds budget"
        );
        let bytes = bytes.into_boxed_slice();
        *length = bytes.len();
        *out = Box::into_raw(bytes).cast::<u8>();
        Ok::<_, anyhow::Error>(())
    }))
    .ok()
    .and_then(Result::ok)
    .map_or(-1, |()| 0)
}

unsafe extern "C" fn release(_data: *mut c_void, bytes: *const u8, length: usize) {
    // Only buffers returned by read reach this callback, including zero length.
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
        bytes.cast_mut(),
        length,
    )));
}

unsafe extern "C" fn gl_proc(data: *mut c_void, name: *const c_char) -> *mut c_void {
    catch_unwind(AssertUnwindSafe(|| {
        let data = &*(data.cast::<HostData>());
        CStr::from_ptr(name)
            .to_str()
            .map(|name| data.context.get_proc_address(name).cast_mut())
            .unwrap_or(std::ptr::null_mut())
    }))
    .unwrap_or(std::ptr::null_mut())
}

fn checked(status: i32, error: &[c_char]) -> Result<()> {
    if status != 0 {
        // The ABI guarantees a NUL terminator in this caller-owned buffer.
        let error = unsafe { CStr::from_ptr(error.as_ptr()) };
        bail!("Scene renderer: {}", error.to_string_lossy());
    }
    Ok(())
}

/// A render-thread-only handle. Rc and EGL ownership prevent Send/Sync and
/// guarantee that the current-context cleanup happens before surface teardown.
pub struct SceneRenderer {
    handle: NonNull<c_void>,
    api: ManuallyDrop<Api>,
    _host: ManuallyDrop<Box<HostData>>,
    window: ManuallyDrop<Rc<EglWindow>>,
    closed: bool,
}

impl SceneRenderer {
    pub fn new(
        library: &Path,
        manifest_json: &str,
        resources: Box<dyn SceneResources>,
        context: EglContext,
        window: Rc<EglWindow>,
        scaling: u32,
    ) -> Result<Self> {
        window.make_current()?;
        let api = Api::load(library)?;
        ensure!(
            manifest_json.len() <= 1024 * 1024,
            "Scene manifest exceeds budget"
        );
        let manifest = CString::new(manifest_json)?;
        let mut data = Box::new(HostData { resources, context });
        let host = Host {
            userdata: (&mut *data as *mut HostData).cast(),
            exists,
            read,
            release,
            gl_proc,
        };
        let config = Config {
            manifest_json: manifest.as_ptr(),
            width: window.width().try_into()?,
            height: window.height().try_into()?,
            scaling,
            volume: 0.0,
            muted: 1,
        };
        let mut handle = std::ptr::null_mut();
        let mut error = [0; 2048];
        let status =
            unsafe { (api.create)(&host, &config, &mut handle, error.as_mut_ptr(), error.len()) };
        checked(status, &error)?;
        Ok(Self {
            handle: NonNull::new(handle).context("Scene returned an empty successful handle")?,
            api: ManuallyDrop::new(api),
            _host: ManuallyDrop::new(data),
            window: ManuallyDrop::new(window),
            closed: false,
        })
    }

    pub fn render(
        &mut self,
        width: u32,
        height: u32,
        framebuffer: u32,
        time: f64,
        delta: f64,
        spectrum: &[f32; 64],
    ) -> Result<()> {
        ensure!(!self.closed, "Scene handle already released");
        self.window.make_current()?;
        let mut error = [0; 2048];
        let status = unsafe {
            (self.api.render)(
                self.handle.as_ptr(),
                width,
                height,
                framebuffer,
                time,
                delta,
                spectrum.as_ptr(),
                error.as_mut_ptr(),
                error.len(),
            )
        };
        checked(status, &error)
    }

    pub fn set_audio(&mut self, paused: bool, muted: bool, volume: f32) -> Result<()> {
        ensure!(!self.closed, "Scene handle already released");
        self.window.make_current()?;
        ensure!(volume.is_finite(), "Invalid Scene volume");
        let mut error = [0; 2048];
        let status = unsafe {
            (self.api.audio)(
                self.handle.as_ptr(),
                i32::from(paused),
                i32::from(muted),
                volume.clamp(0.0, 1.0),
                error.as_mut_ptr(),
                error.len(),
            )
        };
        checked(status, &error)
    }
}

impl SceneRenderer {
    /// Explicit release lets the session retain ownership and retry on a lost
    /// current binding before destroying its output window.
    pub fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        self.window.make_current()?;
        let mut error = [0; 2048];
        let status =
            unsafe { (self.api.destroy)(self.handle.as_ptr(), error.as_mut_ptr(), error.len()) };
        checked(status, &error)?;
        self.closed = true;
        Ok(())
    }
}

impl Drop for SceneRenderer {
    fn drop(&mut self) {
        // Retry a transient binding failure while the surface is still owned.
        if let Err(error) = self.close().or_else(|_| self.close()) {
            // A context that cannot be rebound cannot safely destroy GPU state.
            // Retain native code, callbacks and surface together; freeing any of
            // them would create dangling callbacks. Report this exceptional path.
            tracing::error!("Cannot safely destroy Scene renderer: {error}");
            return;
        }
        unsafe {
            ManuallyDrop::drop(&mut self._host);
            ManuallyDrop::drop(&mut self.window);
            ManuallyDrop::drop(&mut self.api);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use x11::xlib;

    struct Resources(Rc<Cell<usize>>);
    impl SceneResources for Resources {
        fn exists(&self, name: &str) -> Result<bool> {
            Ok(name == "scene.json")
        }
        fn read(&self, name: &str) -> Result<Vec<u8>> {
            ensure!(name == "scene.json", "Missing fixture resource");
            self.0.set(self.0.get() + 1);
            Ok(br#"{"camera":{"center":"0 0 0","eye":"0 0 1","up":"0 1 0"},"general":{"orthogonalprojection":{"width":64,"height":64},"clearcolor":"0.25 0.5 0.75"},"objects":[]}"#.to_vec())
        }
    }

    struct NativeWindow {
        display: *mut xlib::Display,
        window: xlib::Window,
    }
    impl NativeWindow {
        fn new() -> Self {
            unsafe {
                let display = xlib::XOpenDisplay(std::ptr::null());
                assert!(
                    !display.is_null(),
                    "Xvfb is required for the native Scene fixture"
                );
                let screen = xlib::XDefaultScreen(display);
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
                Self { display, window }
            }
        }
        fn context(&self) -> EglContext {
            let visual = unsafe {
                xlib::XVisualIDFromVisual(xlib::XDefaultVisual(
                    self.display,
                    xlib::XDefaultScreen(self.display),
                ))
            };
            EglContext::new_x11(self.display.cast(), visual as i32).unwrap()
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
    fn native_owner_keeps_surface_and_callbacks_alive_until_cleanup() {
        let Some(library) = std::env::var_os("VARPAPER_SCENE_LIBRARY") else {
            return;
        };
        let library = std::path::PathBuf::from(library);
        let native = NativeWindow::new();
        let context = native.context();
        let baseline = crate::egl::native_counts();
        if std::env::var_os("VARPAPER_EXPECT_SCENE_GL_UNAVAILABLE").is_some() {
            let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
            let result = SceneRenderer::new(
                &library,
                r#"{"type":"scene","title":"Unsupported GL fixture","file":"scene.json"}"#,
                Box::new(Resources(Rc::new(Cell::new(0)))),
                context.clone(),
                window.clone(),
                3,
            );
            let error = match result {
                Err(error) => error,
                Ok(_) => panic!("Scene accepted insufficient OpenGL"),
            };
            assert!(error.to_string().contains("OpenGL 3.3"), "{error}");
            drop(window);
            assert_eq!(crate::egl::native_counts(), baseline);
            return;
        }
        for _ in 0..10 {
            let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
            let weak_window = Rc::downgrade(&window);
            let reads = Rc::new(Cell::new(0));
            let mut renderer = SceneRenderer::new(
                &library,
                r#"{"type":"scene","title":"Rust owner fixture","file":"scene.json"}"#,
                Box::new(Resources(reads.clone())),
                context.clone(),
                window.clone(),
                3,
            )
            .unwrap();
            context.load_gl_functions();
            drop(window);
            assert!(weak_window.upgrade().is_some());
            renderer
                .render(64, 64, 0, 1.0, 1.0 / 60.0, &[0.0; 64])
                .unwrap();
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
            assert!(
                (i32::from(pixel[0]) - 64).abs() < 3,
                "Scene first frame: {pixel:?}"
            );
            assert_eq!(reads.get(), 1);
            renderer.set_audio(true, true, 0.5).unwrap();
            crate::egl::fail_next_binding();
            drop(renderer);
            assert!(weak_window.upgrade().is_none());
            assert_eq!(crate::egl::native_counts(), baseline);
            let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
            assert!(SceneRenderer::new(
                &library,
                r#"{"type":"scene"}"#,
                Box::new(Resources(reads.clone())),
                context.clone(),
                window.clone(),
                3
            )
            .is_err());
            drop(window);
            assert_eq!(crate::egl::native_counts(), baseline);
        }
    }
}
