use super::super::fixtures::{clear_resources, NativeWindow};

#[test]
fn rust_scene_audio_shader_channels_silence_and_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (gpu::counts(), program::counts(), draw::mesh_count());
    let mut resources = fixtures::image_resources();
    resources.0.insert("shaders/fixture.frag".into(), b"uniform float g_AudioSpectrum16Left[16]; uniform float g_AudioSpectrum32Right[32]; uniform float g_AudioLevelLeft; void main(){gl_FragColor=vec4(g_AudioSpectrum16Left[0],g_AudioSpectrum32Right[0],g_AudioLevelLeft,1.0);}".to_vec());
    let mut scene = RustSceneRenderer::new("scene.json", &resources, context, window).unwrap();
    let mut audio = crate::audio::AudioSnapshot::default();
    audio.spectrum[..4].copy_from_slice(&[0.0, 0.25, 0.75, 1.0]);
    audio.spectrum[64..66].copy_from_slice(&[0.25, 0.75]);
    audio.level = [1.0, 0.0];
    let pixel = || {
        let mut rgba = [0u8; 4];
        unsafe {
            gl::ReadPixels(
                16,
                32,
                1,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba.as_mut_ptr().cast(),
            );
        }
        rgba
    };
    scene.set_spectrum(&audio).unwrap();
    scene.render_at(64, 64, 0, 0.0).unwrap();
    assert_eq!(pixel(), [128, 128, 255, 255]);
    scene.render_at(64, 64, 0, 0.1).unwrap();
    assert_eq!(pixel(), [0, 0, 0, 255]);
    audio.spectrum.fill(f32::NAN);
    audio.level.fill(f32::INFINITY);
    scene.set_spectrum(&audio).unwrap();
    scene.render_at(64, 64, 0, 0.2).unwrap();
    assert_eq!(pixel(), [0, 0, 0, 255]);
    drop(scene);
    assert_eq!(
        (gpu::counts(), program::counts(), draw::mesh_count()),
        baseline
    );
}
use super::*;

#[test]
fn rust_scene_multipass_pingpong_parameters_and_failure_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (gpu::counts(), program::counts(), draw::mesh_count());
    let mut resources = fixtures::image_resources();
    resources.0.insert("material.json".into(), br#"{"passes":[{"shader":"fixture","textures":["fixture"]},{"shader":"effect"},{"shader":"effect"}]}"#.to_vec());
    resources.0.insert(
        "shaders/effect.vert".into(),
        resources.0["shaders/fixture.vert"].clone(),
    );
    resources.0.insert("shaders/effect.frag".into(), b"uniform sampler2D g_Texture0;\nuniform float gain; // {\"material\":\"gain\",\"default\":0.5}\nvarying vec2 uv;\nvoid main(){gl_FragColor=vec4(texSample2D(g_Texture0,uv).rgb*gain,1.0);}\n".to_vec());
    let sample = || {
        let mut pixel = [0u8; 4];
        unsafe {
            gl::ReadPixels(
                16,
                32,
                1,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixel.as_mut_ptr().cast(),
            );
        }
        pixel
    };
    for _ in 0..10 {
        let mut scene =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .unwrap();
        scene.render(64, 64, 0).unwrap();
        assert!(
            (i32::from(sample()[0]) - 64).abs() < 3,
            "Two sequential gain passes must produce 0.25 red"
        );
        scene
            .set_material_constants("material.json", serde_json::json!({"gain":0.8}))
            .unwrap();
        scene.render(64, 64, 0).unwrap();
        assert!((i32::from(sample()[0]) - 163).abs() < 3);
        assert!(scene
            .set_material_constants("material.json", serde_json::json!({"gain":"NaN"}))
            .is_err());
        scene.render(64, 64, 0).unwrap();
        assert!(
            (i32::from(sample()[0]) - 163).abs() < 3,
            "Invalid parameters must preserve prior material"
        );
        assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
        drop(scene);
        assert_eq!(
            (gpu::counts(), program::counts(), draw::mesh_count()),
            baseline
        );
    }
    gpu::fail_after_texture();
    assert!(
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).is_err()
    );
    assert_eq!(
        (gpu::counts(), program::counts(), draw::mesh_count()),
        baseline
    );
}

#[test]
fn rust_scene_effect_files_named_targets_copy_bindings_overrides_and_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (gpu::counts(), program::counts(), draw::mesh_count());
    let mut resources = fixtures::image_resources();
    let mut document: serde_json::Value =
        serde_json::from_slice(&resources.0["scene.json"]).unwrap();
    document["objects"] = serde_json::json!([{"id":1,"name":"effects","image":"image.json","origin":"32 32 0","size":"4 4","effects":[{"id":1,"file":"effect.json","passes":[{"constantshadervalues":{"gain":0.5}},{"constantshadervalues":{"gain":0.25}}]},{"id":2,"file":"effect.json","visible":false,"passes":[{"constantshadervalues":{"gain":0.2}},{"constantshadervalues":{"gain":0.2}}]}]}]);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    resources.0.insert("effect.json".into(), br#"{"fbos":[{"name":"_rt_first"},{"name":"_rt_copy"}],"passes":[{"material":"gain.json","target":"_rt_first"},{"command":"copy","source":"_rt_first","target":"_rt_copy"},{"material":"gain.json","bind":[{"index":0,"name":"_rt_copy"}]}]}"#.to_vec());
    resources.0.insert(
        "gain.json".into(),
        br#"{"passes":[{"shader":"gain"}]}"#.to_vec(),
    );
    resources.0.insert(
        "shaders/gain.vert".into(),
        resources.0["shaders/fixture.vert"].clone(),
    );
    resources.0.insert("shaders/gain.frag".into(), b"uniform sampler2D g_Texture0;\nuniform float gain; // {\"material\":\"gain\",\"default\":1.0}\nvarying vec2 uv;\nvoid main(){gl_FragColor=vec4(texSample2D(g_Texture0,uv).rgb*gain,1.0); }\n".to_vec());
    // The separate reference uses this authored shader for its copy command.
    resources.0.insert(
        "shaders/commands/copy.vert".into(),
        resources.0["shaders/fixture.vert"].clone(),
    );
    resources.0.insert("shaders/commands/copy.frag".into(), b"uniform sampler2D g_Texture0; varying vec2 uv; void main(){gl_FragColor=texSample2D(g_Texture0,uv);}".to_vec());
    let sample = || {
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
        pixel
    };
    for iteration in 0..10 {
        let mut scene =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .unwrap();
        unsafe {
            gl::Viewport(7, 8, 12, 13);
        }
        scene.render(64, 64, 0).unwrap();
        assert!(
            (i32::from(sample()[0]) - 32).abs() <= 2,
            "Pass order and overrides: {:?}",
            sample()
        );
        let mut viewport = [0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
        }
        assert_eq!(viewport, [7, 8, 12, 13]);
        if iteration == 0 {
            fixtures::capture_reference_case("common-effects", &resources);
        }
        scene
            .set_material_constants(
                "gain.json#layer1-effect0-pass2",
                serde_json::json!({"gain":0.75}),
            )
            .unwrap();
        scene.render(64, 64, 0).unwrap();
        assert!((i32::from(sample()[0]) - 96).abs() <= 2);
        assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
        drop(scene);
        assert_eq!(
            (gpu::counts(), program::counts(), draw::mesh_count()),
            baseline
        );
    }
    let original = resources.0["effect.json"].clone();
    for invalid in [
        serde_json::json!({"fbos":[{"name":"_rt_first"}],"passes":[{"command":"copy","source":"_rt_first","target":"_rt_first"}]}),
        serde_json::json!({"passes":[{"command":"unsupported","source":"missing","target":"missing"}]}),
        serde_json::json!({"fbos":[{"name":"_rt_first","scale":0}],"passes":[{"material":"gain.json"}]}),
    ] {
        resources
            .0
            .insert("effect.json".into(), serde_json::to_vec(&invalid).unwrap());
        assert!(
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .is_err()
        );
        assert_eq!(
            (gpu::counts(), program::counts(), draw::mesh_count()),
            baseline
        );
    }
    resources.0.insert("effect.json".into(), original);
    gpu::fail_after_texture();
    assert!(
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).is_err()
    );
    assert_eq!(
        (gpu::counts(), program::counts(), draw::mesh_count()),
        baseline
    );
}

#[test]
fn rust_scene_animated_images_atlas_rotation_pause_resume_and_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (gpu::counts(), program::counts(), draw::mesh_count());
    let sample = || {
        let mut p = [0u8; 4];
        unsafe {
            gl::ReadPixels(
                16,
                32,
                1,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                p.as_mut_ptr().cast(),
            );
        }
        p
    };
    for atlas in [false, true] {
        let mut resources = fixtures::image_resources();
        resources.0.insert("shaders/fixture.frag".into(), b"uniform sampler2D g_Texture0;\nuniform vec4 g_Texture0Rotation;\nuniform vec2 g_Texture0Translation;\nvarying vec2 uv;\nvoid main(){vec2 p=vec2(dot(g_Texture0Rotation.xy,uv),dot(g_Texture0Rotation.zw,uv))+g_Texture0Translation; gl_FragColor=texSample2D(g_Texture0,p);}".to_vec());
        let mut pixels = Vec::new();
        for _y in 0..4 {
            for x in 0..4 {
                pixels.extend(if atlas && x >= 2 {
                    [0, 255, 0, 255]
                } else {
                    [255, 0, 0, 255]
                });
            }
        }
        let mut bytes = fixtures::texture(0, &pixels, false);
        bytes[22..26].copy_from_slice(&7u32.to_le_bytes());
        if !atlas {
            bytes[55..59].copy_from_slice(&2u32.to_le_bytes());
            let green = fixtures::texture(0, &[0, 255, 0, 255].repeat(16), false);
            bytes.extend(&green[59..]);
        }
        bytes.extend(b"TEXS0003\0");
        for v in [if atlas { 3u32 } else { 2 }, 4, 4] {
            bytes.extend(v.to_le_bytes());
        }
        let frames = if atlas {
            vec![
                (0u32, [0.0f32, 0.0, 2.0, 0.0, 0.0, 4.0]),
                (0, [2.0, 0.0, 2.0, 0.0, 0.0, 4.0]),
                (0, [2.0, 4.0, 0.0, 2.0, -4.0, 0.0]),
            ]
        } else {
            vec![
                (0, [0.0, 0.0, 4.0, 0.0, 0.0, 4.0]),
                (1, [0.0, 0.0, 4.0, 0.0, 0.0, 4.0]),
            ]
        };
        for (image, rect) in frames {
            bytes.extend(image.to_le_bytes());
            bytes.extend(0.125f32.to_le_bytes());
            for v in rect {
                bytes.extend(v.to_le_bytes());
            }
        }
        resources.0.insert("materials/fixture.tex".into(), bytes);
        for _ in 0..5 {
            let mut scene =
                RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                    .unwrap();
            scene.render_at(64, 64, 0, 0.0).unwrap();
            assert_eq!(sample(), [255, 0, 0, 255]);
            scene.render_at(64, 64, 0, 0.13).unwrap();
            assert_eq!(sample(), [0, 255, 0, 255]);
            scene.set_paused_at(true, 0.13).unwrap();
            scene.render_at(64, 64, 0, 100.0).unwrap();
            assert_eq!(sample(), [0, 255, 0, 255]);
            scene.set_paused_at(false, 200.0).unwrap();
            scene.render_at(64, 64, 0, 200.01).unwrap();
            assert_eq!(sample(), [0, 255, 0, 255]);
            scene.render_at(64, 64, 0, 200.13).unwrap();
            assert_eq!(
                sample(),
                if atlas {
                    [0, 255, 0, 255]
                } else {
                    [255, 0, 0, 255]
                }
            );
            scene.render_at(64, 64, 0, 200.27).unwrap();
            assert_eq!(
                sample(),
                if atlas {
                    [255, 0, 0, 255]
                } else {
                    [0, 255, 0, 255]
                }
            );
            assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
            drop(scene);
            assert_eq!(
                (gpu::counts(), program::counts(), draw::mesh_count()),
                baseline
            );
        }
    }
}

#[test]
fn rust_scene_particle_sprites_seeded_frames_parent_visibility_pause_and_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, particle_draw, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (
        gpu::counts(),
        program::counts(),
        draw::mesh_count(),
        particle_draw::counts(),
    );
    let mut resources = fixtures::image_resources();
    let mut document: serde_json::Value =
        serde_json::from_slice(&resources.0["scene.json"]).unwrap();
    document["objects"] = serde_json::json!([{"id":1,"name":"parent","solid":true,"visible":true},{"id":2,"name":"particles","particle":"particle.json","parent":1,"origin":"32 32 0"}]);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    resources.0.insert("particle.json".into(),br#"{"material":"material.json","maxcount":8,"emitter":[{"name":"boxrandom","rate":0,"instantaneous":1,"distancemin":0,"distancemax":0}],"initializer":[{"name":"sizerandom","min":4,"max":4},{"name":"velocityrandom","min":[32,0,0],"max":[32,0,0]},{"name":"lifetimerandom","min":10,"max":10}],"operator":[{"name":"movement"}]}"#.to_vec());
    resources.0.insert("shaders/fixture.vert".into(),b"attribute vec3 a_Position; attribute vec4 a_TexCoordVec4; uniform mat4 g_ModelViewProjectionMatrix; varying vec2 uv; void main(){uv=a_TexCoordVec4.xy;vec3 p=a_Position+vec3((uv-vec2(0.5))*a_TexCoordVec4.w,0.0);gl_Position=g_ModelViewProjectionMatrix*vec4(p,1.0);}".to_vec());
    let frame = || {
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
        pixels
    };
    let sample =
        |x: usize, pixels: &[u8]| pixels[(32 * 64 + x) * 4..(32 * 64 + x) * 4 + 4].to_vec();
    let mut expected = None;
    for _ in 0..5 {
        let mut scene = RustSceneRenderer::new_with_seed(
            "scene.json",
            &resources,
            context.clone(),
            window.clone(),
            42,
        )
        .unwrap();
        scene.render_at(64, 64, 0, 0.0).unwrap();
        scene.render_at(64, 64, 0, 0.1).unwrap();
        let first = frame();
        assert_eq!(sample(32, &first), [255, 0, 0, 255]);
        scene.render_at(64, 64, 0, 0.2).unwrap();
        let moved = frame();
        assert_ne!(first, moved);
        assert_eq!(sample(35, &moved), [255, 0, 0, 255]);
        if let Some(expected) = &expected {
            assert_eq!(expected, &moved);
        } else {
            expected = Some(moved.clone());
        }
        scene.set_paused_at(true, 0.2).unwrap();
        scene.render_at(64, 64, 0, 100.0).unwrap();
        assert_eq!(frame(), moved);
        scene.set_paused_at(false, 200.0).unwrap();
        scene.render_at(64, 64, 0, 200.1).unwrap();
        assert_eq!(sample(38, &frame()), [255, 0, 0, 255]);
        drop(scene);
        assert_eq!(
            (
                gpu::counts(),
                program::counts(),
                draw::mesh_count(),
                particle_draw::counts()
            ),
            baseline
        );
    }
    document["objects"][0]["visible"] = serde_json::json!(false);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let mut scene = RustSceneRenderer::new_with_seed(
        "scene.json",
        &resources,
        context.clone(),
        window.clone(),
        42,
    )
    .unwrap();
    scene.render_at(64, 64, 0, 0.0).unwrap();
    scene.render_at(64, 64, 0, 0.1).unwrap();
    assert_eq!(sample(32, &frame()), [0, 0, 255, 255]);
    drop(scene);
    assert_eq!(
        (
            gpu::counts(),
            program::counts(),
            draw::mesh_count(),
            particle_draw::counts()
        ),
        baseline
    );
}

#[test]
fn rust_scene_text_font_fallback_layout_budget_visibility_and_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (gpu::counts(), program::counts(), draw::mesh_count());
    let mut resources = fixtures::clear_resources();
    let mut document: serde_json::Value =
        serde_json::from_slice(&resources.0["scene.json"]).unwrap();
    document["general"]["clearcolor"] = serde_json::json!("0 0 1");
    document["objects"] = serde_json::json!([{"id":1,"name":"parent","solid":true},{"id":2,"name":"Text","parent":1,"text":"A\nBB","origin":"32 32 0","size":"48 48","pointsize":16,"color":"1 1 1 1","padding":2}]);
    let frame = || {
        let mut p = vec![0u8; 64 * 64 * 4];
        unsafe {
            gl::ReadPixels(
                0,
                0,
                64,
                64,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                p.as_mut_ptr().cast(),
            );
        }
        p
    };
    let mut expected = None;
    for font in ["", "missing.ttf"] {
        document["objects"][1]["font"] = serde_json::json!(font);
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
        for _ in 0..5 {
            let mut scene =
                RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                    .unwrap();
            scene.render(64, 64, 0).unwrap();
            let pixels = frame();
            assert!(
                pixels
                    .chunks_exact(4)
                    .filter(|v| v[0] > 20 && v[1] > 20)
                    .count()
                    > 40,
                "Text must visibly render"
            );
            if let Some(expected) = &expected {
                assert_eq!(expected, &pixels, "Missing font must use the same fallback");
            } else {
                expected = Some(pixels);
            }
            assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
            scene.close().unwrap();
            drop(scene);
            assert_eq!(
                (gpu::counts(), program::counts(), draw::mesh_count()),
                baseline
            );
        }
    }

    // Test a project-owned font independently of system fallback, including cache reuse.
    let font_path = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ]
    .into_iter()
    .find(|path| std::path::Path::new(path).exists())
    .unwrap();
    resources.0.insert(
        "materials/fonts/project.ttf".into(),
        std::fs::read(font_path).unwrap(),
    );
    document["objects"][1]["font"] = serde_json::json!("project.ttf");
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
    assert_eq!(frame(), expected.unwrap());
    scene.set_text(2, "BBBB", &resources).unwrap();
    scene.render(64, 64, 0).unwrap();
    let updated = frame();
    assert!(scene.set_text(2, &"x".repeat(4097), &resources).is_err());
    scene.render(64, 64, 0).unwrap();
    assert_eq!(
        frame(),
        updated,
        "Bad text update must preserve the last texture"
    );
    scene.set_text(2, "", &resources).unwrap();
    scene.render(64, 64, 0).unwrap();
    assert!(frame().chunks_exact(4).all(|v| v == [0, 0, 255, 255]));
    drop(scene);
    assert_eq!(
        (gpu::counts(), program::counts(), draw::mesh_count()),
        baseline
    );
    document["objects"][0]["visible"] = serde_json::json!(false);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
    assert!(frame().chunks_exact(4).all(|v| v == [0, 0, 255, 255]));
    drop(scene);
    for (field, value) in [
        ("text", serde_json::json!("x".repeat(4097))),
        ("pointsize", serde_json::json!(513)),
        ("size", serde_json::json!("4097 4097")),
        ("font", serde_json::json!("../escape.ttf")),
    ] {
        let mut invalid = document.clone();
        invalid["objects"][1][field] = value;
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(&invalid).unwrap());
        assert!(
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .is_err()
        );
        assert_eq!(
            (gpu::counts(), program::counts(), draw::mesh_count()),
            baseline
        );
    }
    document["objects"][1]["font"] = serde_json::json!("broken.ttf");
    resources.0.insert("broken.ttf".into(), vec![0; 16]);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    assert!(
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).is_err()
    );
    assert_eq!(
        (gpu::counts(), program::counts(), draw::mesh_count()),
        baseline
    );
}

#[test]
fn rust_scene_script_property_and_text_pixels_isolation_pause_and_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program, script};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (
        gpu::counts(),
        program::counts(),
        draw::mesh_count(),
        script::counts(),
    );
    let mut resources = fixtures::image_resources();
    let mut document: serde_json::Value =
        serde_json::from_slice(&resources.0["scene.json"]).unwrap();
    document["objects"][0]["alpha"] = serde_json::json!({"value":1,"script":"export function update(value){return engine.userProperties.opacity;}"});
    resources.0.insert(
        "material.json".into(),
        br#"{"passes":[{"shader":"fixture","textures":["fixture"],"blending":"translucent"}]}"#
            .to_vec(),
    );
    resources.0.insert("shaders/fixture.frag".into(),b"uniform sampler2D g_Texture0;uniform float g_Alpha;varying vec2 uv;void main(){vec4 p=texSample2D(g_Texture0,uv);gl_FragColor=vec4(p.rgb,g_Alpha);}".to_vec());
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let sample = || {
        let mut p = [0u8; 4];
        unsafe {
            gl::ReadPixels(
                16,
                32,
                1,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                p.as_mut_ptr().cast(),
            );
        }
        p
    };
    let mut a =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    let mut b =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    a.set_project_properties(&serde_json::json!({"opacity":0.25}))
        .unwrap();
    b.set_project_properties(&serde_json::json!({"opacity":0.75}))
        .unwrap();
    a.render_at(64, 64, 0, 0.0).unwrap();
    assert!((i32::from(sample()[0]) - 64).abs() < 3);
    b.render_at(64, 64, 0, 0.0).unwrap();
    assert!((i32::from(sample()[0]) - 191).abs() < 3);
    a.set_paused_at(true, 0.0).unwrap();
    a.set_project_properties(&serde_json::json!({"opacity":0.5}))
        .unwrap();
    a.render_at(64, 64, 0, 100.0).unwrap();
    assert!((i32::from(sample()[0]) - 64).abs() < 3);
    a.set_paused_at(false, 200.0).unwrap();
    a.render_at(64, 64, 0, 200.1).unwrap();
    assert!((i32::from(sample()[0]) - 128).abs() < 3);
    drop(a);
    drop(b);
    assert_eq!(
        (
            gpu::counts(),
            program::counts(),
            draw::mesh_count(),
            script::counts()
        ),
        baseline
    );
    document["objects"] = serde_json::json!([{"id":1,"name":"Clock","text":{"value":"A","script":"export function update(value){return engine.time<0.1?'A':'BBBB';}"},"origin":"32 32 0","pointsize":16,"color":"1 1 1 1"}]);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let frame = || {
        let mut p = vec![0u8; 64 * 64 * 4];
        unsafe {
            gl::ReadPixels(
                0,
                0,
                64,
                64,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                p.as_mut_ptr().cast(),
            );
        }
        p
    };
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render_at(64, 64, 0, 0.0).unwrap();
    let first = frame();
    scene.render_at(64, 64, 0, 0.2).unwrap();
    assert_ne!(first, frame(), "Script must update text pixels");
    drop(scene);
    assert_eq!(
        (
            gpu::counts(),
            program::counts(),
            draw::mesh_count(),
            script::counts()
        ),
        baseline
    );
}

#[test]
fn rust_scene_layer_camera_parent_radian_rotation_blending_and_uv_crop() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let baseline = (gpu::counts(), program::counts(), draw::mesh_count());
    let mut resources = fixtures::image_resources();
    let original: serde_json::Value = serde_json::from_slice(&resources.0["scene.json"]).unwrap();
    let pixel = |x, y| {
        let mut pixel = [0u8; 4];
        unsafe {
            gl::ReadPixels(
                x,
                y,
                1,
                1,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixel.as_mut_ptr().cast(),
            );
        }
        pixel
    };
    let mut document = original.clone();
    document["objects"] = serde_json::json!([{"id":1,"name":"Rotated","image":"image.json","origin":"16 16 0","size":"16 8","angles":[0,0,std::f32::consts::FRAC_PI_2]}]);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
    assert!(pixel(16, 48)[0] > 250, "top-down Scene origin");
    assert!(pixel(16, 42)[0] > 250, "Scene angles must be radians");
    assert!(
        pixel(22, 48)[2] > 250,
        "Rotated rectangle must remain bounded"
    );
    fixtures::capture_reference_case("rotation", &resources);
    drop(scene);

    document["objects"] = serde_json::json!([
        {"id":10,"name":"Parent","solid":true,"origin":"32 16 0","scale":"2 1 1"},
        {"id":11,"name":"Child","parent":10,"image":"image.json","origin":"8 8 0","size":"8 8","alpha":0.5},
    ]);
    resources.0.insert("shaders/fixture.frag".into(), b"uniform sampler2D g_Texture0;\nuniform vec3 g_Color;\nuniform float g_Alpha;\nvarying vec2 uv;\nvoid main(){gl_FragColor=vec4(texSample2D(g_Texture0,uv).rgb*g_Color,g_Alpha);}\n".to_vec());
    resources.0.insert(
        "material.json".into(),
        br#"{"passes":[{"shader":"fixture","textures":["fixture"],"blending":"translucent"}]}"#
            .to_vec(),
    );
    for visible in [true, false] {
        document["objects"][0]["visible"] = serde_json::json!(visible);
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
        let mut scene =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .unwrap();
        scene.render(64, 64, 0).unwrap();
        let result = pixel(48, 40);
        if visible {
            assert!(
                (result[0] as i32 - 128).abs() < 3 && (result[2] as i32 - 128).abs() < 3,
                "parent alpha blend {result:?}"
            );
        } else {
            assert!(
                result[0] < 3 && result[2] > 250,
                "Parent visibility must hide child"
            );
        }
        fixtures::capture_reference_case(
            if visible {
                "parent-visible"
            } else {
                "parent-hidden"
            },
            &resources,
        );
        drop(scene);
    }

    resources = fixtures::image_resources();
    let mut payload = Vec::new();
    for _ in 0..4 {
        payload.extend([255, 0, 0, 255].repeat(2));
        payload.extend([0, 255, 0, 255].repeat(2));
    }
    let mut texture = fixtures::texture(0, &payload, false);
    texture[34..38].copy_from_slice(&2u32.to_le_bytes());
    resources.0.insert("materials/fixture.tex".into(), texture);
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
    for x in [4, 28, 36, 60] {
        assert!(
            pixel(x, 32)[0] > 250 && pixel(x, 32)[1] < 3,
            "Logical texture crop must exclude green padding"
        );
    }
    fixtures::capture_reference_case("crop", &resources);
    drop(scene);
    // Shared reference semantics: declared size equals logical texture size.
    // The approved explicit-size/ancestor-visibility differences above remain
    // separate cases rather than being hidden inside a large pixel tolerance.
    resources = fixtures::image_resources();
    document = original.clone();
    document["objects"] = serde_json::json!([{"id":1,"name":"Common rotation","image":"image.json","origin":"16 16 0","size":"4 2","scale":"4 2 1","angles":"0 0 1.5707963267948966"}]);
    let mut texture = fixtures::texture(0, &[255, 0, 0, 255].repeat(16), false);
    texture[38..42].copy_from_slice(&2u32.to_le_bytes());
    resources.0.insert("materials/fixture.tex".into(), texture);
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
    fixtures::capture_reference_case("common-rotation", &resources);
    drop(scene);
    document["objects"] = serde_json::json!([
        {"id":10,"name":"Parent","solid":true,"origin":"32 16 0","scale":"2 1 1"},
        {"id":11,"name":"Common child","parent":10,"image":"image.json","origin":"8 8 0","size":"4 2","alpha":0.5},
    ]);
    resources.0.insert(
        "material.json".into(),
        br#"{"passes":[{"shader":"fixture","textures":["fixture"],"blending":"translucent"}]}"#
            .to_vec(),
    );
    resources.0.insert("shaders/fixture.frag".into(), b"uniform sampler2D g_Texture0;\nuniform float g_Alpha;\nvarying vec2 uv;\nvoid main(){gl_FragColor=vec4(texSample2D(g_Texture0,uv).rgb,g_Alpha);}\n".to_vec());
    resources
        .0
        .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
    fixtures::capture_reference_case("common-parent-alpha", &resources);
    drop(scene);
    assert_eq!(
        (gpu::counts(), program::counts(), draw::mesh_count()),
        baseline
    );
    assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
}

#[test]
fn rust_scene_shader_material_uniforms_defaults_and_driver_failure_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    let native = NativeWindow::new();
    let context = native.context();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let gpu_baseline = gpu::counts();
    let program_baseline = program::counts();
    let mesh_baseline = draw::mesh_count();
    let mut resources = fixtures::image_resources();
    let mut caller_texture = 0;
    unsafe {
        gl::GenTextures(1, &mut caller_texture);
        gl::ActiveTexture(gl::TEXTURE0 + 19);
        gl::BindTexture(gl::TEXTURE_2D, caller_texture);
    }
    resources.0.insert("shaders/fixture.frag".into(), b"uniform sampler2D g_Texture0; // {\"default\":\"fixture\"}\nuniform vec3 tint; // {\"material\":\"tint\",\"default\":\"0.25 0.5 0.75\"}\nuniform float intensity; // {\"material\":\"power\",\"default\":1}\nvarying vec2 uv;\nvoid main(){gl_FragColor=vec4(tint*intensity,texSample2D(g_Texture0,uv).a);}\n".to_vec());
    for (constants, expected) in [
        (serde_json::json!({}), [64, 128, 191]),
        (
            serde_json::json!({"tint":"0.8 0.4 0.2","power":0.5}),
            [102, 51, 26],
        ),
    ] {
        resources.0.insert("material.json".into(), serde_json::to_vec(&serde_json::json!({"passes":[{"shader":"fixture","constantshadervalues":constants}]})).unwrap());
        let mut scene =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .unwrap();
        scene.render(64, 64, 0).unwrap();
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
        for channel in 0..3 {
            assert!(
                (i32::from(pixel[channel]) - expected[channel]).abs() < 3,
                "{pixel:?}"
            );
        }
        assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
        drop(scene);
        let (mut active, mut binding) = (0, 0);
        unsafe {
            gl::GetIntegerv(gl::ACTIVE_TEXTURE, &mut active);
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut binding);
        }
        assert_eq!(active as u32, gl::TEXTURE0 + 19);
        assert_eq!(binding as u32, caller_texture);
        assert_eq!(gpu::counts(), gpu_baseline);
        assert_eq!(program::counts(), program_baseline);
        assert_eq!(draw::mesh_count(), mesh_baseline);
    }
    for constants in [
        serde_json::json!({"tint":"NaN 0 1"}),
        serde_json::json!({"tint":[1,2]}),
        serde_json::json!({"power":"invalid"}),
    ] {
        resources.0.insert("material.json".into(), serde_json::to_vec(&serde_json::json!({"passes":[{"shader":"fixture","constantshadervalues":constants}]})).unwrap());
        let error =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .err()
                .expect("Invalid uniform accepted");
        assert!(format!("{error:#}").contains("uniform"));
        assert_eq!(gpu::counts(), gpu_baseline);
        assert_eq!(program::counts(), program_baseline);
    }
    resources.0.insert(
        "material.json".into(),
        br#"{"passes":[{"shader":"fixture"}]}"#.to_vec(),
    );
    resources.0.insert("shaders/fixture.frag".into(), b"uniform int toggle; // {\"default\":1}\nuniform bool enabled; // {\"default\":true}\nuniform vec4 coefficients; // {\"default\":\"0.2 0.4 0.6 1\"}\nuniform mat3 basis; // {\"default\":\"1 0 0 0 1 0 0 0 1\"}\nuniform float weights[2]; // {\"default\":\"0.25 0.75\"}\nvoid main(){gl_FragColor=vec4((basis*coefficients.rgb)*weights[0]+vec3(weights[1]),enabled&&toggle==1?coefficients.a:0.0);}\n".to_vec());
    let mut scene =
        RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone()).unwrap();
    scene.render(64, 64, 0).unwrap();
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
    for (actual, expected) in pixel[..3].iter().zip([204, 217, 230]) {
        assert!(
            (i32::from(*actual) - expected).abs() < 3,
            "matrix/array/integer defaults {pixel:?}"
        );
    }
    assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
    drop(scene);
    assert_eq!(gpu::counts(), gpu_baseline);
    assert_eq!(program::counts(), program_baseline);
    resources.0.insert("shaders/fixture.frag".into(), b"uniform sampler2D g_Texture0; // {\"default\":\"fixture\",\"require\":{\"MODE\":1}}\nvarying vec2 uv;\nvoid main(){\n#if MODE == 1\n gl_FragColor=texSample2D(g_Texture0,uv);\n#else\n gl_FragColor=vec4(0.0,0.0,1.0,1.0);\n#endif\n}\n".to_vec());
    for mode in [0, 1] {
        resources.0.insert(
            "material.json".into(),
            serde_json::to_vec(
                &serde_json::json!({"passes":[{"shader":"fixture","combos":{"MODE":mode}}]}),
            )
            .unwrap(),
        );
        let mut scene =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .unwrap();
        scene.render(64, 64, 0).unwrap();
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
        if mode == 1 {
            assert!(pixel[0] > 250 && pixel[2] < 3);
        } else {
            assert!(pixel[0] < 3 && pixel[2] > 250);
        }
        drop(scene);
        assert_eq!(gpu::counts(), gpu_baseline);
        assert_eq!(program::counts(), program_baseline);
    }
    program::fail_program_allocation();
    let error = RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
        .err()
        .expect("Injected driver failure accepted");
    assert!(format!("{error:#}").contains("Cannot create Scene shader program"));
    assert_eq!(gpu::counts(), gpu_baseline);
    assert_eq!(program::counts(), program_baseline);
    assert_eq!(draw::mesh_count(), mesh_baseline);
    unsafe {
        gl::DeleteTextures(1, &caller_texture);
        gl::ActiveTexture(gl::TEXTURE0);
    }
}

#[test]
fn rust_scene_loose_package_shared_frames_and_include_compilation() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    use wayvid_library::project::{discover_project, ProjectResources};
    let native = NativeWindow::new();
    let context = native.context();
    let baseline = crate::egl::native_counts();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let gpu_baseline = gpu::counts();
    let program_baseline = program::counts();
    let mesh_baseline = draw::mesh_count();
    let temp = tempfile::tempdir().unwrap();
    let shared = temp.path().join("shared-assets");
    std::fs::create_dir_all(shared.join("materials")).unwrap();
    let mut resources = fixtures::image_resources();
    // Compile nested includes and format macros through the real driver.
    resources.0.insert("shaders/fixture.frag".into(), b"// [COMBO] {\"combo\":\"RED\",\"default\":1}\nuniform sampler2D g_Texture0;\nvarying vec2 uv;\n#include \"color\"\nvoid main(){gl_FragColor=fixtureColor();}\n".to_vec());
    resources.0.insert("shaders/color.h".into(), b"#include \"nested\"\nfloat4 fixtureColor(){return texSample2D(g_Texture0,uv)*float4(saturate(float(RED)),1.0,1.0,1.0);}\n".to_vec());
    resources.0.insert(
        "shaders/nested.h".into(),
        b"// generated nested include\n".to_vec(),
    );
    std::fs::write(
        shared.join("materials/fixture.tex"),
        &resources.0["materials/fixture.tex"],
    )
    .unwrap();
    let mut frames = Vec::new();
    for kind in ["loose", "packed", "shared", "missing-shared"] {
        let root = temp.path().join(kind);
        if kind == "loose" {
            fixtures::export(&resources, &root).unwrap();
        } else {
            std::fs::create_dir(&root).unwrap();
            std::fs::write(root.join("project.json"), &resources.0["project.json"]).unwrap();
            let mut package = fixtures::Resources(resources.0.clone());
            if kind != "packed" {
                package.0.remove("materials/fixture.tex");
            }
            std::fs::write(root.join("scene.pkg"), fixtures::package(&package)).unwrap();
        }
        let project = discover_project(
            &root,
            wayvid_library::SourceType::LocalDirectory,
            None,
            if kind == "shared" {
                vec![shared.clone()]
            } else {
                vec![]
            },
        )
        .unwrap()
        .project
        .unwrap();
        let reader = ProjectResources::new(&project).unwrap();
        let result = RustSceneRenderer::new("scene.json", &reader, context.clone(), window.clone());
        if kind == "missing-shared" {
            let error = result.err().expect("Missing shared texture accepted");
            assert!(format!("{error:#}").contains("materials/fixture.tex"));
        } else {
            let mut scene = result.unwrap();
            scene.render(64, 64, 0).unwrap();
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
            assert!(frame
                .chunks_exact(4)
                .all(|pixel| pixel[0] > 250 && pixel[1] < 3 && pixel[2] < 3));
            frames.push(frame);
            drop(scene);
        }
        assert_eq!(gpu::counts(), gpu_baseline);
        assert_eq!(program::counts(), program_baseline);
        assert_eq!(draw::mesh_count(), mesh_baseline);
    }
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0], frames[1]);
    assert_eq!(frames[0], frames[2]);
    drop(window);
    assert_eq!(crate::egl::native_counts(), baseline);
}

#[test]
fn rust_scene_twelve_texture_formats_shared_upload_and_shader_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    use super::super::{draw, fixtures, gpu, program};
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    struct Counted {
        resources: fixtures::Resources,
        reads: RefCell<BTreeMap<String, usize>>,
    }
    impl SceneResources for Counted {
        fn exists(&self, path: &str) -> Result<bool> {
            self.resources.exists(path)
        }
        fn read(&self, path: &str) -> Result<Vec<u8>> {
            *self.reads.borrow_mut().entry(path.into()).or_default() += 1;
            self.resources.read(path)
        }
    }
    let native = NativeWindow::new();
    let context = native.context();
    let baseline = crate::egl::native_counts();
    let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
    window.make_current().unwrap();
    context.load_gl_functions();
    let gpu_baseline = gpu::counts();
    let program_baseline = program::counts();
    let mesh_baseline = draw::mesh_count();
    let color = [0, 0xf8, 0, 0, 0, 0, 0, 0];
    let payloads = [
        (0, [255, 0, 0, 255].repeat(16)),
        (9, vec![255; 16]),
        (8, [255, 0].repeat(16)),
        (7, color.to_vec()),
        (6, [vec![255; 8], color.to_vec()].concat()),
        (4, [vec![255, 0, 0, 0, 0, 0, 0, 0], color.to_vec()].concat()),
    ];
    for (format, payload) in payloads {
        for compressed in [false, true] {
            let mut resources = fixtures::image_resources();
            resources.0.insert(
                "materials/fixture.tex".into(),
                fixtures::texture(format, &payload, compressed),
            );
            let resources = Counted {
                resources,
                reads: RefCell::new(BTreeMap::new()),
            };
            let mut scene =
                RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                    .unwrap();
            assert_eq!(resources.reads.borrow()["materials/fixture.tex"], 1);
            assert_eq!(gpu::counts(), (gpu_baseline.0 + 1, gpu_baseline.1 + 2));
            assert_eq!(
                program::counts(),
                (program_baseline.0, program_baseline.1 + 1)
            );
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + 3);
                gl::Enable(gl::DEPTH_TEST);
                gl::Disable(gl::BLEND);
            }
            scene.render(64, 64, 0).unwrap();
            let mut active = 0;
            unsafe {
                gl::GetIntegerv(gl::ACTIVE_TEXTURE, &mut active);
            }
            assert_eq!(active as u32, gl::TEXTURE0 + 3);
            assert_eq!(unsafe { gl::IsEnabled(gl::DEPTH_TEST) }, gl::TRUE);
            assert_eq!(unsafe { gl::IsEnabled(gl::BLEND) }, gl::FALSE);
            for x in [16, 48] {
                let mut pixel = [0u8; 4];
                unsafe {
                    gl::ReadPixels(
                        x,
                        32,
                        1,
                        1,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        pixel.as_mut_ptr().cast(),
                    );
                }
                assert!(
                    pixel[0] > 250 && pixel[1] < 3 && pixel[2] < 3,
                    "format {format}, LZ4 {compressed}: {pixel:?}"
                );
            }
            assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
            drop(scene);
            assert_eq!(gpu::counts(), gpu_baseline);
            assert_eq!(program::counts(), program_baseline);
            assert_eq!(draw::mesh_count(), mesh_baseline);
        }
    }
    for (vertex, fragment, diagnostic) in [
        (
            "void main(){ invalid tokens; }",
            "void main(){gl_FragColor=vec4(1.0);}",
            "Vertex shader compilation failed",
        ),
        (
            "void main(){gl_Position=vec4(0.0);}",
            "void main(){ invalid tokens; }",
            "Fragment shader compilation failed",
        ),
        (
            "uniform float mismatch; void main(){gl_Position=vec4(mismatch);}",
            "uniform vec3 mismatch; void main(){gl_FragColor=vec4(mismatch,1.0);}",
            "link failed",
        ),
    ] {
        let mut resources = fixtures::image_resources();
        resources
            .0
            .insert("shaders/fixture.vert".into(), vertex.as_bytes().to_vec());
        resources
            .0
            .insert("shaders/fixture.frag".into(), fragment.as_bytes().to_vec());
        for _ in 0..10 {
            let error =
                RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                    .err()
                    .expect("Invalid shader accepted");
            assert!(format!("{error:#}").contains(diagnostic), "{error:#}");
            assert!(format!("{error:#}").contains("fixture"));
            assert_eq!(gpu::counts(), gpu_baseline);
            assert_eq!(program::counts(), program_baseline);
            assert_eq!(draw::mesh_count(), mesh_baseline);
        }
    }
    drop(window);
    assert_eq!(crate::egl::native_counts(), baseline);
}

#[test]
fn rust_scene_first_frame_state_and_failed_allocation_cleanup() {
    if std::env::var_os("VARPAPER_TEST_X11").is_none() {
        return;
    }
    let native = NativeWindow::new();
    let context = native.context();
    let baseline = crate::egl::native_counts();
    let gpu_baseline = super::super::gpu::counts();
    let resources = clear_resources();
    if std::env::var_os("VARPAPER_EXPECT_SCENE_GL_UNAVAILABLE").is_some() {
        let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
        let error =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .err()
                .expect("Rust Scene accepted missing OpenGL 3.3");
        assert!(error.to_string().contains("OpenGL 3.3"), "{error}");
        drop(window);
        assert_eq!(crate::egl::native_counts(), baseline);
        return;
    }
    for _ in 0..20 {
        let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
        let weak = Rc::downgrade(&window);
        let mut scene =
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .unwrap();
        drop(window);
        assert!(weak.upgrade().is_some());
        unsafe {
            gl::Viewport(3, 4, 12, 13);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(0, 0, 1, 1);
            gl::ColorMask(gl::FALSE, gl::TRUE, gl::FALSE, gl::TRUE);
            gl::ClearColor(0.1, 0.2, 0.3, 0.4);
        }
        scene.render(64, 64, 0).unwrap();
        let mut viewport = [0; 4];
        let mut mask = [0u8; 4];
        let mut color = [0.0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            gl::GetBooleanv(gl::COLOR_WRITEMASK, mask.as_mut_ptr());
            gl::GetFloatv(gl::COLOR_CLEAR_VALUE, color.as_mut_ptr());
        }
        assert_eq!(viewport, [3, 4, 12, 13]);
        assert_eq!(mask, [0, 1, 0, 1]);
        assert_eq!(color, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(unsafe { gl::IsEnabled(gl::SCISSOR_TEST) }, gl::TRUE);
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
        for (channel, expected) in [(0, 64), (1, 128), (2, 191)] {
            assert!(
                (i32::from(pixel[channel]) - expected).abs() < 3,
                "Rust Scene pixel {pixel:?}"
            );
        }
        let live = super::super::gpu::counts();
        super::super::gpu::fail_after_texture();
        assert!(scene.render(32, 32, 0).is_err());
        assert_eq!(super::super::gpu::counts(), live);
        scene.render(64, 64, 0).unwrap();
        // A caller's tiny PBO and non-default row layout must not affect
        // allocation, and must remain bound after a successful resize.
        let mut buffer = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, buffer);
            gl::BufferData(
                gl::PIXEL_UNPACK_BUFFER,
                1,
                std::ptr::null(),
                gl::STREAM_DRAW,
            );
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 8);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 7);
            gl::PixelStorei(gl::UNPACK_SKIP_ROWS, 2);
        }
        scene.render(32, 32, 0).unwrap();
        let mut binding = 0;
        unsafe { gl::GetIntegerv(gl::PIXEL_UNPACK_BUFFER_BINDING, &mut binding) };
        assert_eq!(binding as u32, buffer);
        for (field, expected) in [
            (gl::UNPACK_ALIGNMENT, 8),
            (gl::UNPACK_ROW_LENGTH, 7),
            (gl::UNPACK_SKIP_ROWS, 2),
        ] {
            let mut actual = 0;
            unsafe { gl::GetIntegerv(field, &mut actual) };
            assert_eq!(actual, expected);
        }
        assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR);
        unsafe {
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            gl::DeleteBuffers(1, &buffer);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
            gl::PixelStorei(gl::UNPACK_SKIP_ROWS, 0);
        }
        crate::egl::fail_next_binding();
        drop(scene);
        assert!(weak.upgrade().is_none());
        assert_eq!(crate::egl::native_counts(), baseline);
        assert_eq!(super::super::gpu::counts(), gpu_baseline);
        let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
        window.make_current().unwrap();
        super::super::gpu::fail_after_texture();
        assert!(
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .is_err()
        );
        assert_eq!(super::super::gpu::counts(), gpu_baseline);
        drop(window);
        assert_eq!(crate::egl::native_counts(), baseline);
    }
}
