//! Generated assets exported by the Rust fixture; no Workshop content.
use super::*;
use crate::scene::{runtime::RustSceneRenderer, SceneResources};
use std::fs;
use std::path::Path;
use std::rc::Rc;
use wayvid_library::project::{discover_project, ProjectResources};

const FILES: &[&str] = &[
    "scene.json",
    "image.json",
    "material.json",
    "shaders/fixture.vert",
    "shaders/fixture.frag",
    "materials/fixture.tex",
];
struct Resources(ProjectResources);
impl SceneResources for Resources {
    fn exists(&self, name: &str) -> Result<bool> {
        self.0.exists(name)
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        self.0.read(name)
    }
}
fn copy_resource(source: &Path, root: &Path, name: &str) {
    let target = root.join(name);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(source.join(name), target).unwrap();
}
fn package(source: &Path, destination: &Path, names: &[&str]) {
    let mut bytes = Vec::new();
    let mut data = Vec::new();
    bytes.extend(8u32.to_le_bytes());
    bytes.extend(b"PKGV0001");
    bytes.extend((names.len() as u32).to_le_bytes());
    for name in names {
        let content = fs::read(source.join(name)).unwrap();
        bytes.extend((name.len() as u32).to_le_bytes());
        bytes.extend(name.as_bytes());
        bytes.extend((data.len() as u32).to_le_bytes());
        bytes.extend((content.len() as u32).to_le_bytes());
        data.extend(content);
    }
    bytes.extend(data);
    fs::write(destination, bytes).unwrap();
}

#[test]
fn textured_scene_loose_packed_shared_and_missing_resources() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    let authored = tempfile::tempdir().unwrap();
    let fixture = authored.path().join("generated");
    crate::scene::fixtures::export(&crate::scene::fixtures::image_resources(), &fixture).unwrap();
    let baseline = crate::egl::native_counts();
    let display = Display::open().unwrap();
    let mut output = display.outputs().unwrap().remove(0);
    output.width = 64;
    output.height = 64;
    let context = EglContext::new_x11(display.raw.cast(), display.visual_id()).unwrap();
    let window = display.create_desktop_window(&output).unwrap();
    let surface = Rc::new(context.create_x11_window(window, 64, 64).unwrap());
    surface.make_current().unwrap();
    context.load_gl_functions();
    let temp = tempfile::tempdir().unwrap();
    let shared = temp.path().join("shared-assets");
    fs::create_dir(&shared).unwrap();
    copy_resource(&fixture, &shared, "materials/fixture.tex");
    let manifest = fs::read_to_string(fixture.join("project.json")).unwrap();
    let mut frames = Vec::new();
    for kind in ["loose", "packed", "shared", "missing-shared"] {
        let root = temp.path().join(kind);
        fs::create_dir(&root).unwrap();
        fs::write(root.join("project.json"), &manifest).unwrap();
        match kind {
            "loose" => {
                for name in FILES {
                    copy_resource(&fixture, &root, name);
                }
            }
            "packed" => package(&fixture, &root.join("scene.pkg"), FILES),
            _ => {
                // Both loose texture fallback and package->shared fallback are used.
                package(&fixture, &root.join("scene.pkg"), &FILES[..FILES.len() - 1]);
            }
        }
        let roots = if kind == "shared" {
            vec![shared.clone()]
        } else {
            vec![]
        };
        let project = discover_project(
            &root,
            wayvid_library::SourceType::LocalDirectory,
            None,
            roots,
        )
        .unwrap()
        .project
        .unwrap();
        let resources = Resources(ProjectResources::new(&project).unwrap());
        let result =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), surface.clone());
        if kind == "missing-shared" {
            let error = match result {
                Ok(_) => panic!("Missing shared texture was accepted"),
                Err(error) => error,
            };
            assert!(
                error.to_string().contains("materials/fixture.tex"),
                "{error}"
            );
            continue;
        }
        let mut scene = result.unwrap();
        scene.render_at(64, 64, 0, 0.0).unwrap();
        let mut frame = vec![0u8; 64 * 64 * 4];
        unsafe {
            gl::ReadPixels(
                0,
                0,
                64,
                64,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                frame.as_mut_ptr().cast(),
            );
        }
        for x in [16, 48] {
            // X11 RGB visuals need not provide an alpha channel.
            assert_eq!(
                &frame[(32 * 64 + x) * 4..(32 * 64 + x) * 4 + 3],
                &[255, 0, 0]
            );
        }
        frames.push(frame);
        scene.close().unwrap();
    }
    assert_eq!(frames.len(), 3);
    assert!(
        frames.windows(2).all(|pair| pair[0] == pair[1]),
        "Resource origin changed the rendered frame"
    );
    drop(surface);
    drop(context);
    unsafe {
        xlib::XDestroyWindow(display.raw, window);
    }
    assert_eq!(crate::egl::native_counts(), baseline);
}
