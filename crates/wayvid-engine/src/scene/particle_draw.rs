//! Render-thread-owned sprite geometry for the qualified particle core.
use super::{
    assets::{Assets, Blend, Object},
    geometry::Geometry,
    gpu::{Target, TextureUpload},
    model::{bound, scalar, vector},
    particle::Simulation,
    program::Program,
};
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;
#[cfg(test)]
thread_local! {static LIVE:std::cell::Cell<usize>=const {std::cell::Cell::new(0)};}
#[cfg(test)]
pub(super) fn counts() -> usize {
    LIVE.with(std::cell::Cell::get)
}
pub(super) struct ParticleDraw {
    simulation: Simulation,
    vao: u32,
    buffer: u32,
    material: String,
}
impl ParticleDraw {
    pub fn new(definition: &super::particle::Definition, seed: u64) -> Result<Self> {
        let mut owner = Self {
            simulation: Simulation::new(definition.clone(), seed),
            vao: 0,
            buffer: 0,
            material: definition.material.clone(),
        };
        #[cfg(test)]
        LIVE.with(|v| v.set(v.get() + 1));
        unsafe {
            gl::GenVertexArrays(1, &mut owner.vao);
            gl::GenBuffers(1, &mut owner.buffer);
        }
        ensure!(
            owner.vao != 0 && owner.buffer != 0,
            "Cannot allocate Scene particle geometry"
        );
        Ok(owner)
    }
    pub fn render(
        &mut self,
        object: &Object,
        assets: &Assets,
        destination: &Target,
        program: &Program,
        textures: &BTreeMap<String, TextureUpload>,
        inputs: (super::clock::FrameTime, &crate::audio::AudioSnapshot),
    ) -> Result<()> {
        let (time, audio) = inputs;
        self.simulation.advance(time.delta)?;
        let (transform, projection, visible) = Geometry::layer_transform(object, assets)?;
        if !visible || self.simulation.particles.is_empty() {
            return Ok(());
        }
        let material = &assets.materials[&self.material];
        ensure!(
            material.passes.len() == 1,
            "Rust particle multi-pass integration is incomplete"
        );
        let pass = &material.passes[0];
        let mut vertices = Vec::with_capacity(self.simulation.particles.len() * 6 * 17);
        for particle in &self.simulation.particles {
            if particle.size <= 0.0 {
                continue;
            }
            for (u, v) in [
                (0.0, 1.0),
                (1.0, 1.0),
                (1.0, 0.0),
                (1.0, 0.0),
                (0.0, 0.0),
                (0.0, 1.0),
            ] {
                vertices.extend(particle.position.to_array());
                vertices.extend([u, v, particle.rotation.z, particle.size]);
                vertices.extend(particle.color.to_array());
                vertices.push(particle.alpha);
                vertices.extend(particle.velocity.to_array());
                vertices.push(particle.age / particle.lifetime);
                vertices.extend([particle.rotation.x, particle.rotation.y]);
            }
        }
        let alpha = scalar(&object.properties["alpha"], 1.0)?;
        let color = vector(bound(&object.properties["color"]), [1.0f32; 3])?;
        ensure!(
            (0.0..=1.0).contains(&alpha)
                && color
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
            "Invalid Scene particle color/alpha"
        );
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, destination.framebuffer);
            gl::Viewport(0, 0, destination.width as i32, destination.height as i32);
            gl::UseProgram(program.id);
            program.apply_constants(pass)?;
            super::draw::apply_audio_uniforms(program, audio)?;
            let matrix = (projection * transform).to_cols_array();
            for name in ["g_ModelViewProjectionMatrix", "g_ViewProjectionMatrix"] {
                gl::UniformMatrix4fv(program.uniform(name)?, 1, gl::FALSE, matrix.as_ptr());
            }
            gl::Uniform1f(program.uniform("g_Time")?, time.elapsed as f32);
            for name in ["g_Alpha", "g_UserAlpha"] {
                gl::Uniform1f(program.uniform(name)?, alpha);
            }
            for name in ["g_Color", "g_CompositeColor"] {
                gl::Uniform3fv(program.uniform(name)?, 1, color.as_ptr());
            }
            gl::Uniform1f(program.uniform("g_Brightness")?, 1.0);
            gl::Uniform3f(program.uniform("g_OrientationUp")?, 0.0, 1.0, 0.0);
            gl::Uniform3f(program.uniform("g_OrientationRight")?, 1.0, 0.0, 0.0);
            gl::Uniform3f(program.uniform("g_OrientationForward")?, 0.0, 0.0, 1.0);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(vertices.as_slice()) as isize,
                vertices.as_ptr().cast(),
                gl::STREAM_DRAW,
            );
            for (name, count, offset) in [
                (c"a_Position", 3, 0),
                (c"a_TexCoordVec4", 4, 3),
                (c"a_TexCoord", 2, 3),
                (c"a_Color", 4, 7),
                (c"a_TexCoordVec4C1", 4, 11),
                (c"a_TexCoordC2", 2, 15),
            ] {
                let attribute = gl::GetAttribLocation(program.id, name.as_ptr());
                if attribute >= 0 {
                    gl::EnableVertexAttribArray(attribute as u32);
                    gl::VertexAttribPointer(
                        attribute as u32,
                        count,
                        gl::FLOAT,
                        gl::FALSE,
                        68,
                        (offset * 4) as *const _,
                    );
                }
            }
            for (unit, name) in &pass.textures {
                let key = format!("materials/{name}.tex");
                let texture = textures
                    .get(&key)
                    .with_context(|| format!("Unresolved particle texture '{name}'"))?;
                let asset = &assets.textures[&key];
                let animation = asset.frame_at(time.elapsed)?;
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                gl::BindTexture(gl::TEXTURE_2D, texture.images[animation.image]);
                gl::Uniform1i(program.uniform(&format!("g_Texture{unit}"))?, *unit as i32);
                let mip = &asset.images[animation.image][0];
                gl::Uniform4f(
                    program.uniform(&format!("g_Texture{unit}Resolution"))?,
                    mip.width as f32,
                    mip.height as f32,
                    asset.logical_width as f32,
                    asset.logical_height as f32,
                );
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
            }
            gl::Enable(gl::BLEND);
            gl::BlendEquationSeparate(gl::FUNC_ADD, gl::FUNC_ADD);
            let (source, dest) = match pass.blend {
                Blend::Normal => (gl::ONE, gl::ZERO),
                Blend::Additive => (gl::SRC_ALPHA, gl::ONE),
                Blend::Translucent => (gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA),
            };
            gl::BlendFuncSeparate(source, dest, source, dest);
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::FALSE);
            gl::Disable(gl::CULL_FACE);
            gl::DrawArrays(gl::TRIANGLES, 0, (vertices.len() / 17) as i32);
            ensure!(gl::GetError() == gl::NO_ERROR, "Scene particle draw failed");
        }
        Ok(())
    }
}
impl Drop for ParticleDraw {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.buffer);
        }
        #[cfg(test)]
        LIVE.with(|v| v.set(v.get() - 1));
    }
}
