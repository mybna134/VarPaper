//! Rust image draw assets; additional object/effect integration is tracked separately.
use super::geometry::Geometry;
use super::{
    assets::{Assets, Blend, EffectStep, ObjectKind},
    gpu::{State, Target, TextureUpload},
    program::Program,
    SceneResources,
};
use anyhow::{ensure, Context, Result};
use glam::{Mat4, Vec3};
use std::collections::BTreeMap;

#[cfg(test)]
thread_local! { static MESHES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(super) fn mesh_count() -> usize {
    MESHES.with(std::cell::Cell::get)
}
struct Mesh {
    vao: u32,
    buffer: u32,
}
impl Mesh {
    fn new() -> Result<Self> {
        let mut mesh = Self { vao: 0, buffer: 0 };
        #[cfg(test)]
        MESHES.with(|count| count.set(count.get() + 1));
        let vertices: [f32; 20] = [
            -0.5, -0.5, 0.0, 0.0, 1.0, 0.5, -0.5, 0.0, 1.0, 1.0, -0.5, 0.5, 0.0, 0.0, 0.0, 0.5,
            0.5, 0.0, 1.0, 0.0,
        ];
        unsafe {
            gl::GenVertexArrays(1, &mut mesh.vao);
            gl::GenBuffers(1, &mut mesh.buffer);
            ensure!(
                mesh.vao != 0 && mesh.buffer != 0,
                "Cannot allocate Scene image mesh"
            );
            gl::BindVertexArray(mesh.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, mesh.buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&vertices) as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 20, std::ptr::null());
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 20, 12usize as *const _);
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            ensure!(
                gl::GetError() == gl::NO_ERROR,
                "Scene image mesh allocation failed"
            );
        }
        Ok(mesh)
    }
}
impl Mesh {
    unsafe fn update(&self, uv: [f32; 2], input_target: bool) {
        let top = if input_target { 1.0 } else { 0.0 };
        let bottom = if input_target { 0.0 } else { uv[1] };
        let vertices: [f32; 20] = [
            -0.5, -0.5, 0.0, 0.0, top, -0.5, 0.5, 0.0, 0.0, bottom, 0.5, -0.5, 0.0, uv[0], top,
            0.5, 0.5, 0.0, uv[0], bottom,
        ];
        gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            std::mem::size_of_val(&vertices) as isize,
            vertices.as_ptr().cast(),
        );
    }
}
impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.buffer);
        }
        #[cfg(test)]
        MESHES.with(|count| count.set(count.get() - 1));
    }
}
pub(super) struct DrawAssets {
    mesh: Option<Mesh>,
    programs: BTreeMap<String, Vec<Program>>,
    textures: BTreeMap<String, TextureUpload>,
    layers: BTreeMap<i64, [Target; 2]>,
    named: BTreeMap<String, Target>,
    particles: BTreeMap<i64, super::particle_draw::ParticleDraw>,
}
impl DrawAssets {
    pub fn replace_texture(&mut self, name: String, texture: TextureUpload) {
        self.textures.insert(name, texture);
    }
    pub fn validate_material(&self, name: &str, material: &super::assets::Material) -> Result<()> {
        let programs = self
            .programs
            .get(name)
            .context("Scene material program unavailable")?;
        for (index, (pass, program)) in material.passes.iter().zip(programs).enumerate() {
            program.validate_with_input(pass, index > 0 || name.contains("#layer"))?;
        }
        Ok(())
    }
    pub fn new(assets: &Assets, _resources: &dyn SceneResources, seed: u64) -> Result<Self> {
        let _state = State::capture();
        for object in &assets.objects {
            match &object.kind {
                ObjectKind::Image(name) => {
                    Geometry::image(object, name, assets)?;
                }
                ObjectKind::Text(_) => {
                    Geometry::image(object, &assets.text_models[&object.id], assets)?;
                }
                _ => {}
            }
            ensure!(
                matches!(
                    object.kind,
                    ObjectKind::Image(_)
                        | ObjectKind::Group
                        | ObjectKind::Particle(_)
                        | ObjectKind::Text(_)
                        | ObjectKind::Sound(_)
                ),
                "Rust Scene object integration is incomplete: {}",
                object.name
            );
        }
        let mut draw = Self {
            mesh: None,
            programs: BTreeMap::new(),
            textures: BTreeMap::new(),
            layers: BTreeMap::new(),
            named: BTreeMap::new(),
            particles: BTreeMap::new(),
        };
        for (name, material) in &assets.materials {
            let mut programs = Vec::new();
            for pass in &material.passes {
                let [vertex, fragment] = pass
                    .sources
                    .as_ref()
                    .context("Scene shader sources not resolved")?;
                let program = Program::new(&pass.shader, vertex, fragment)?;
                program.validate_with_input(pass, true)?;
                programs.push(program);
            }
            draw.programs.insert(name.clone(), programs);
        }
        for (name, model) in &assets.models {
            ensure!(
                model
                    .properties
                    .get("puppet")
                    .is_none_or(|value| value.is_null() || value.as_str() == Some("")),
                "Unsupported required Scene puppet model '{name}'"
            );
        }
        for (name, texture) in &assets.textures {
            draw.textures.insert(
                name.clone(),
                TextureUpload::new(texture)
                    .with_context(|| format!("Scene texture upload '{name}'"))?,
            );
        }
        let mut layer_bytes = 0u64;
        for object in &assets.objects {
            if let ObjectKind::Image(name) = &object.kind {
                let material = &assets.materials[&assets.models[name].material];
                let pipeline = &assets.effects[&object.id];
                // The first base pass cannot sample an implicit previous image.
                draw.programs[&assets.models[name].material][0]
                    .validate_with_input(&material.passes[0], false)?;
                if pipeline.steps.len() > 1 {
                    let size = Geometry::image(object, name, assets)?.size;
                    ensure!(
                        size.iter().all(|value| (1.0..=16384.0).contains(value)),
                        "Invalid effect layer dimensions"
                    );
                    let (width, height) = (size[0].ceil() as u32, size[1].ceil() as u32);
                    layer_bytes += u64::from(width) * u64::from(height) * 8;
                    ensure!(
                        layer_bytes <= 256 * 1024 * 1024,
                        "Scene effect targets exceed 256 MiB"
                    );
                    draw.layers.insert(
                        object.id,
                        [Target::new(width, height)?, Target::new(width, height)?],
                    );
                    for (name, scale) in &pipeline.targets {
                        let dimensions = [width as f32 / scale, height as f32 / scale];
                        ensure!(
                            dimensions
                                .iter()
                                .all(|v| v.is_finite() && (1.0..=16384.0).contains(v)),
                            "Invalid effect target dimensions '{name}'"
                        );
                        let (width, height) =
                            (dimensions[0].ceil() as u32, dimensions[1].ceil() as u32);
                        layer_bytes += u64::from(width) * u64::from(height) * 4;
                        ensure!(
                            layer_bytes <= 256 * 1024 * 1024,
                            "Scene effect targets exceed 256 MiB"
                        );
                        draw.named.insert(name.clone(), Target::new(width, height)?);
                    }
                }
            }
        }
        for object in &assets.objects {
            if let ObjectKind::Particle(name) = &object.kind {
                let definition = &assets.particles[name];
                let material = &assets.materials[&definition.material];
                ensure!(
                    material.passes.len() == 1,
                    "Rust particle multi-pass integration is incomplete"
                );
                draw.programs[&definition.material][0]
                    .validate_with_input(&material.passes[0], false)?;
                draw.particles.insert(
                    object.id,
                    super::particle_draw::ParticleDraw::new(
                        definition,
                        seed.wrapping_add(object.id as u64),
                    )?,
                );
            }
        }
        if !assets.objects.is_empty() {
            draw.mesh = Some(Mesh::new()?);
        }
        Ok(draw)
    }
    pub fn render(
        &mut self,
        assets: &Assets,
        destination: &Target,
        seconds: f64,
        delta: f64,
        audio: &crate::audio::AudioSnapshot,
    ) -> Result<()> {
        for object in &assets.objects {
            if let ObjectKind::Particle(name) = &object.kind {
                let material = &assets.particles[name].material;
                self.particles
                    .get_mut(&object.id)
                    .context("Scene particle renderer unavailable")?
                    .render(
                        object,
                        assets,
                        destination,
                        &self.programs[material][0],
                        &self.textures,
                        (
                            super::clock::FrameTime {
                                elapsed: seconds,
                                delta,
                            },
                            audio,
                        ),
                    )?;
                continue;
            }
            let name = match &object.kind {
                ObjectKind::Image(name) => name,
                ObjectKind::Text(_) => &assets.text_models[&object.id],
                _ => continue,
            };
            let geometry = Geometry::image(object, name, assets)?;
            if !geometry.visible {
                continue;
            }
            let model = &assets.models[name];
            let material = &assets.materials[&model.material];
            let matrix = (geometry.projection * geometry.model).to_cols_array();
            let pipeline = &assets.effects[&object.id];
            let mut previous: Option<&Target> = None;
            let mut pingpong = 0;
            for (index, step) in pipeline.steps.iter().enumerate() {
                let EffectStep::Draw {
                    material: material_name,
                    pass: pass_index,
                    source,
                    target,
                } = step
                else {
                    let EffectStep::Copy { source, target } = step else {
                        unreachable!()
                    };
                    let source = self
                        .named
                        .get(source)
                        .context("Effect copy source unavailable")?;
                    let target = self
                        .named
                        .get(target)
                        .context("Effect copy target unavailable")?;
                    ensure!(
                        source.framebuffer != target.framebuffer,
                        "Effect copy cannot read its destination"
                    );
                    unsafe {
                        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, source.framebuffer);
                        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target.framebuffer);
                        gl::BlitFramebuffer(
                            0,
                            0,
                            source.width as i32,
                            source.height as i32,
                            0,
                            0,
                            target.width as i32,
                            target.height as i32,
                            gl::COLOR_BUFFER_BIT,
                            gl::LINEAR,
                        );
                    }
                    previous = Some(target);
                    continue;
                };
                let pass = &assets.materials[material_name].passes[*pass_index];
                let program = &self.programs[material_name][*pass_index];
                let last = index + 1 == pipeline.steps.len() && target.is_none();
                let input = source
                    .as_ref()
                    .map(|name| self.named.get(name).context("Effect source unavailable"))
                    .transpose()?
                    .or(previous);
                let output = if let Some(name) = target {
                    self.named.get(name).context("Effect target unavailable")?
                } else if last {
                    destination
                } else {
                    &self.layers[&object.id][pingpong]
                };
                ensure!(
                    input.is_none_or(|v| v.framebuffer != output.framebuffer),
                    "Effect pass cannot sample its destination"
                );
                for name in pass
                    .textures
                    .values()
                    .filter(|name| name.starts_with("_rt_"))
                {
                    let texture = self
                        .named
                        .get(name)
                        .with_context(|| format!("Undeclared effect texture '{name}'"))?;
                    ensure!(
                        texture.framebuffer != output.framebuffer,
                        "Effect pass cannot sample its destination"
                    );
                }
                let texture_zero = pass.textures.get(&0);
                let input_target = texture_zero
                    .map(|name| name.starts_with("_rt_"))
                    .unwrap_or(input.is_some());
                let matrix = if last {
                    matrix
                } else {
                    Mat4::from_scale(Vec3::new(2.0, -2.0, 1.0)).to_cols_array()
                };
                unsafe {
                    gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, output.framebuffer);
                    gl::Viewport(0, 0, output.width as i32, output.height as i32);
                    if !last {
                        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                    }
                    gl::UseProgram(program.id);
                    program.apply_constants(pass)?;
                    apply_audio_uniforms(program, audio)?;
                    gl::Uniform1f(program.uniform("g_Time")?, seconds as f32);
                    gl::Uniform4f(program.uniform("g_Texture0Rotation")?, 1.0, 0.0, 0.0, 1.0);
                    gl::Uniform2f(program.uniform("g_Texture0Translation")?, 0.0, 0.0);
                    gl::BindVertexArray(
                        self.mesh
                            .as_ref()
                            .context("Scene image mesh unavailable")?
                            .vao,
                    );
                    gl::UniformMatrix4fv(
                        program.uniform("g_ModelViewProjectionMatrix")?,
                        1,
                        gl::FALSE,
                        matrix.as_ptr(),
                    );
                    self.mesh
                        .as_ref()
                        .context("Scene image mesh unavailable")?
                        .update(
                            if input_target { [1.0; 2] } else { geometry.uv },
                            input_target,
                        );
                    for name in ["g_Alpha", "g_UserAlpha"] {
                        gl::Uniform1f(program.uniform(name)?, geometry.alpha);
                    }
                    gl::Uniform1f(program.uniform("g_Brightness")?, geometry.brightness);
                    for name in ["g_Color", "g_CompositeColor"] {
                        gl::Uniform3fv(program.uniform(name)?, 1, geometry.color.as_ptr());
                    }
                    let color4 = [geometry.color[0], geometry.color[1], geometry.color[2], 1.0];
                    gl::Uniform4fv(program.uniform("g_Color4")?, 1, color4.as_ptr());
                    gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
                    let blend = if last {
                        material.passes[0].blend
                    } else {
                        Blend::Normal
                    };
                    let (source, dest) = match blend {
                        Blend::Normal => (gl::ONE, gl::ZERO),
                        Blend::Additive => (gl::SRC_ALPHA, gl::ONE),
                        Blend::Translucent => (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
                    };
                    gl::Enable(gl::BLEND);
                    gl::BlendFuncSeparate(source, dest, source, dest);
                    if pass.cull {
                        gl::Enable(gl::CULL_FACE);
                    } else {
                        gl::Disable(gl::CULL_FACE);
                    }
                    gl::CullFace(gl::BACK);
                    gl::FrontFace(gl::CCW);
                    if pass.depth_test {
                        gl::Enable(gl::DEPTH_TEST);
                        gl::DepthFunc(gl::LEQUAL);
                    } else {
                        gl::Disable(gl::DEPTH_TEST);
                    }
                    gl::DepthMask(if pass.depth_write {
                        gl::TRUE
                    } else {
                        gl::FALSE
                    });
                    if let Some(input) = input {
                        if !pass.textures.contains_key(&0) {
                            gl::ActiveTexture(gl::TEXTURE0);
                            gl::BindTexture(gl::TEXTURE_2D, input.texture());
                            gl::Uniform1i(program.uniform("g_Texture0")?, 0);
                            let resolution = [
                                input.width as f32,
                                input.height as f32,
                                input.width as f32,
                                input.height as f32,
                            ];
                            gl::Uniform4fv(
                                program.uniform("g_Texture0Resolution")?,
                                1,
                                resolution.as_ptr(),
                            );
                        }
                    }
                    for (unit, name) in &pass.textures {
                        let (texture, resolution) = if name.starts_with("_rt_") {
                            let target = self.named.get(name).with_context(|| {
                                format!("Unresolved Scene pass target '{name}'")
                            })?;
                            (
                                target.texture(),
                                [
                                    target.width as f32,
                                    target.height as f32,
                                    target.width as f32,
                                    target.height as f32,
                                ],
                            )
                        } else {
                            let texture = self
                                .textures
                                .get(&format!("materials/{name}.tex"))
                                .with_context(|| {
                                    format!("Unresolved Scene pass texture '{name}'")
                                })?;
                            let asset = &assets.textures[&format!("materials/{name}.tex")];
                            let animation = asset.frame_at(seconds)?;
                            if *unit == 0 {
                                gl::Uniform4fv(
                                    program.uniform("g_Texture0Rotation")?,
                                    1,
                                    animation.rotation.as_ptr(),
                                );
                                gl::Uniform2fv(
                                    program.uniform("g_Texture0Translation")?,
                                    1,
                                    animation.translation.as_ptr(),
                                );
                            }
                            (
                                texture.images[animation.image],
                                [
                                    asset.images[animation.image][0].width as f32,
                                    asset.images[animation.image][0].height as f32,
                                    asset.logical_width as f32,
                                    asset.logical_height as f32,
                                ],
                            )
                        };
                        gl::ActiveTexture(gl::TEXTURE0 + *unit);
                        gl::BindTexture(gl::TEXTURE_2D, texture);
                        gl::Uniform1i(program.uniform(&format!("g_Texture{unit}"))?, *unit as i32);
                        gl::Uniform4fv(
                            program.uniform(&format!("g_Texture{unit}Resolution"))?,
                            1,
                            resolution.as_ptr(),
                        );
                    }
                    gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
                }
                previous = Some(output);
                if target.is_none() && !last {
                    pingpong = 1 - pingpong;
                }
            }
        }
        Ok(())
    }
}

pub(super) fn apply_audio_uniforms(
    program: &Program,
    audio: &crate::audio::AudioSnapshot,
) -> Result<()> {
    for resolution in [16usize, 32, 64] {
        let mut left = [0.0f32; 64];
        let mut right = [0.0f32; 64];
        let stride = crate::audio::BANDS / resolution;
        for index in 0..resolution {
            let start = index * stride;
            let end = start + stride;
            left[index] = audio.spectrum[start..end]
                .iter()
                .map(|value| finite_audio(*value))
                .sum::<f32>()
                / stride as f32;
            right[index] = audio.spectrum[crate::audio::BANDS + start..crate::audio::BANDS + end]
                .iter()
                .map(|value| finite_audio(*value))
                .sum::<f32>()
                / stride as f32;
        }
        unsafe {
            gl::Uniform1fv(
                program.uniform(&format!("g_AudioSpectrum{resolution}Left"))?,
                resolution as i32,
                left.as_ptr(),
            );
            gl::Uniform1fv(
                program.uniform(&format!("g_AudioSpectrum{resolution}Right"))?,
                resolution as i32,
                right.as_ptr(),
            );
        }
    }
    unsafe {
        gl::Uniform1f(
            program.uniform("g_AudioLevelLeft")?,
            finite_audio(audio.level[0]),
        );
        gl::Uniform1f(
            program.uniform("g_AudioLevelRight")?,
            finite_audio(audio.level[1]),
        );
    }
    Ok(())
}

fn finite_audio(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
