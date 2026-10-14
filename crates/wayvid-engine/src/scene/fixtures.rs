//! Generated test assets. No Workshop content or reference checkout required.
use super::SceneResources;
use anyhow::{Context, Result};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Resources(pub BTreeMap<String, Vec<u8>>);
impl SceneResources for Resources {
    fn exists(&self, name: &str) -> Result<bool> {
        Ok(self.0.contains_key(name))
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        self.0
            .get(name)
            .cloned()
            .with_context(|| format!("Missing fixture resource {name}"))
    }
}
pub fn clear_scene() -> String {
    r#"{"camera":{"center":"0 0 0","eye":"0 0 1","up":"0 1 0"},"general":{"orthogonalprojection":{"width":64,"height":64},"clearcolor":"0.25 0.5 0.75"},"objects":[]}"#.into()
}
pub fn clear_resources() -> Resources {
    Resources(BTreeMap::from([(
        "scene.json".into(),
        clear_scene().into_bytes(),
    )]))
}
/// Assets matching the former two-layer red pixel fixture, authored in code.
pub fn image_resources() -> Resources {
    Resources(BTreeMap::from([
        ("project.json".into(), br#"{"title":"Rust generated Scene","type":"scene","file":"scene.json"}"#.to_vec()),
        ("scene.json".into(), br#"{"camera":{"center":"0 0 0","eye":"0 0 1","up":"0 1 0"},"general":{"orthogonalprojection":{"width":64,"height":64},"clearcolor":"0 0 1"},"objects":[{"id":1,"name":"Left","image":"image.json","origin":"16 32 0","size":"32 64"},{"id":2,"name":"Right","image":"image.json","origin":"48 32 0","size":"32 64"}]}"#.to_vec()),
        ("image.json".into(), br#"{"material":"material.json","width":4,"height":4}"#.to_vec()),
        ("material.json".into(), br#"{"passes":[{"shader":"fixture","textures":["fixture"]}]}"#.to_vec()),
        ("shaders/fixture.vert".into(), b"attribute vec3 a_Position;\nattribute vec2 a_TexCoord;\nuniform mat4 g_ModelViewProjectionMatrix;\nvarying vec2 uv;\nvoid main() { gl_Position=g_ModelViewProjectionMatrix*vec4(a_Position,1.0); uv=a_TexCoord; }\n".to_vec()),
        ("shaders/fixture.frag".into(), b"uniform sampler2D g_Texture0;\nvarying vec2 uv;\nvoid main() { gl_FragColor=texSample2D(g_Texture0,uv); }\n".to_vec()),
        ("materials/fixture.tex".into(), texture(0, &[255, 0, 0, 255].repeat(16), false)),
        ("sounds/fixture.wav".into(), wave()),
    ]))
}
pub fn export(resources: &Resources, path: &std::path::Path) -> Result<()> {
    anyhow::ensure!(!path.exists(), "Fixture export requires a new directory");
    std::fs::create_dir(path)?;
    for (name, bytes) in &resources.0 {
        let path = path.join(wayvid_library::project::resource_name(name)?);
        std::fs::create_dir_all(path.parent().context("Fixture parent")?)?;
        std::fs::write(path, bytes)?;
    }
    Ok(())
}
pub fn package(resources: &Resources) -> Vec<u8> {
    let entries: Vec<_> = resources
        .0
        .iter()
        .filter(|(name, _)| name.as_str() != "project.json")
        .collect();
    let mut index = Vec::new();
    let mut payload: Vec<u8> = Vec::new();
    index.extend(8u32.to_le_bytes());
    index.extend(b"PKGV0001");
    index.extend((entries.len() as u32).to_le_bytes());
    for (name, bytes) in entries {
        index.extend((name.len() as u32).to_le_bytes());
        index.extend(name.as_bytes());
        index.extend((payload.len() as u32).to_le_bytes());
        index.extend((bytes.len() as u32).to_le_bytes());
        payload.extend(bytes);
    }
    index.extend(payload);
    index
}
/// Export identical authored assets for a separate reference renderer process.
pub fn capture_reference_case(name: &str, resources: &Resources) {
    let Some(root) = std::env::var_os("VARPAPER_TEST_SCENE_REFERENCE_OUTPUT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(name);
    export(resources, &path).unwrap();
    let mut pixels = vec![0u8; 64 * 64 * 4];
    unsafe {
        gl::ReadPixels(
            0,
            0,
            64,
            64,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr().cast(),
        );
    }
    let image = image::RgbaImage::from_raw(64, 64, pixels).unwrap();
    image::imageops::flip_vertical(&image)
        .save(path.join("rust.png"))
        .unwrap();
}
pub fn texture(format: u32, pixels: &[u8], compressed: bool) -> Vec<u8> {
    let mut out = b"TEXV0005\0TEXI0001\0".to_vec();
    let push = |out: &mut Vec<u8>, value: u32| out.extend(value.to_le_bytes());
    push(&mut out, format);
    push(&mut out, 3);
    for _ in 0..4 {
        push(&mut out, 4);
    }
    push(&mut out, 0);
    out.extend(b"TEXB0002\0");
    for v in [1, 1, 4, 4, u32::from(compressed), pixels.len() as u32] {
        push(&mut out, v);
    }
    let payload = if compressed {
        lz4_flex::block::compress(pixels)
    } else {
        pixels.to_vec()
    };
    push(&mut out, payload.len() as u32);
    out.extend(payload);
    out
}
pub fn wave() -> Vec<u8> {
    let samples = 4800u32;
    let mut out = b"RIFF".to_vec();
    out.extend((36 + samples * 2).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    for v in [1u16, 1] {
        out.extend(v.to_le_bytes());
    }
    for v in [48000u32, 96000] {
        out.extend(v.to_le_bytes());
    }
    for v in [2u16, 16] {
        out.extend(v.to_le_bytes());
    }
    out.extend(b"data");
    out.extend((samples * 2).to_le_bytes());
    out.resize(44 + samples as usize * 2, 0);
    out
}
#[test]
fn assets_are_deterministic_and_have_exact_payload_lengths() {
    let pixels = [255, 0, 0, 255].repeat(16);
    for compressed in [false, true] {
        assert_eq!(
            texture(0, &pixels, compressed),
            texture(0, &pixels, compressed)
        );
    }
    assert_eq!(wave().len(), 44 + 4800 * 2);
    assert_eq!(&wave()[..4], b"RIFF");
}

#[test]
fn generated_project_exports_are_reproducible() {
    let first = image_resources();
    let second = image_resources();
    assert_eq!(first.0, second.0);
    let directory = tempfile::tempdir().unwrap();
    for name in ["one", "two"] {
        let path = directory.path().join(name);
        export(&first, &path).unwrap();
        for (name, bytes) in &first.0 {
            assert_eq!(&std::fs::read(path.join(name)).unwrap(), bytes);
        }
        assert!(export(&first, &path).is_err());
    }
    if let Some(path) = std::env::var_os("VARPAPER_TEST_SCENE_FIXTURE_EXPORT") {
        export(&first, std::path::Path::new(&path)).unwrap();
    }
}

/// Private Xvfb window; surface/context must be released before this owner.
pub struct NativeWindow {
    display: *mut x11::xlib::Display,
    pub window: x11::xlib::Window,
}
impl NativeWindow {
    pub fn new() -> Self {
        use x11::xlib;
        unsafe {
            let display = xlib::XOpenDisplay(std::ptr::null());
            assert!(
                !display.is_null(),
                "Native Rust Scene fixture requires Xvfb"
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
    pub fn context(&self) -> crate::egl::EglContext {
        use x11::xlib;
        let visual = unsafe {
            xlib::XVisualIDFromVisual(xlib::XDefaultVisual(
                self.display,
                xlib::XDefaultScreen(self.display),
            ))
        };
        crate::egl::EglContext::new_x11(self.display.cast(), visual as i32).unwrap()
    }
}
impl Drop for NativeWindow {
    fn drop(&mut self) {
        unsafe {
            x11::xlib::XDestroyWindow(self.display, self.window);
            x11::xlib::XCloseDisplay(self.display);
        }
    }
}
