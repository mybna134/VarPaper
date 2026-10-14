//! Rust Scene renderer. No Scene dynamic library, C ABI, or reference checkout.
use super::{
    assets::Assets,
    draw::DrawAssets,
    gpu::{State, Target},
    SceneResources,
};
use crate::egl::{EglContext, EglWindow};
use anyhow::{ensure, Context, Result};
use std::rc::Rc;

pub struct RustSceneRenderer {
    assets: Assets,
    draw: Option<DrawAssets>,
    target: Option<Target>,
    window: Rc<EglWindow>,
    _context: EglContext,
    closed: bool,
    epoch: std::time::Instant,
    clock: super::clock::AnimationClock,
    sounds: Vec<super::sound::Stream>,
    audio: (bool, bool, f32),
    pending_audio: Option<crate::audio::AudioSnapshot>,
    scripts: Option<super::script::Scripts>,
    script_started: bool,
}
impl RustSceneRenderer {
    /// Validate a complete material parameter update before publishing it.
    pub fn set_material_constants(
        &mut self,
        name: &str,
        constants: serde_json::Value,
    ) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        ensure!(
            constants
                .as_object()
                .is_some_and(|values| values.len() <= 256),
            "Scene material constants must be an object with at most 256 entries"
        );
        ensure!(
            serde_json::to_vec(&constants)?.len() <= 1024 * 1024,
            "Scene material constant budget exceeded"
        );
        let material = self
            .assets
            .materials
            .get_mut(name)
            .context("Scene material not found")?;
        let previous: Vec<_> = material
            .passes
            .iter_mut()
            .map(|pass| std::mem::replace(&mut pass.constants, constants.clone()))
            .collect();
        let result = self
            .draw
            .as_ref()
            .context("Scene draw assets unavailable")?
            .validate_material(name, material);
        if result.is_err() {
            for (pass, constants) in material.passes.iter_mut().zip(previous) {
                pass.constants = constants;
            }
        }
        result
    }
    pub fn new(
        entry: &str,
        resources: &dyn SceneResources,
        context: EglContext,
        window: Rc<EglWindow>,
    ) -> Result<Self> {
        Self::new_with_seed(entry, resources, context, window, 0)
    }
    pub fn new_with_seed(
        entry: &str,
        resources: &dyn SceneResources,
        context: EglContext,
        window: Rc<EglWindow>,
        seed: u64,
    ) -> Result<Self> {
        Self::new_with_properties(
            entry,
            resources,
            context,
            window,
            seed,
            &serde_json::json!({}),
        )
    }
    pub fn new_with_properties(
        entry: &str,
        resources: &dyn SceneResources,
        context: EglContext,
        window: Rc<EglWindow>,
        seed: u64,
        properties: &serde_json::Value,
    ) -> Result<Self> {
        ensure!(
            properties.is_object() && serde_json::to_vec(properties)?.len() <= 1024 * 1024,
            "Invalid Scene project property budget"
        );
        let mut assets = Assets::load(entry, resources)?;
        if assets.document.auto_projection {
            let mut extent = [0.0f32; 2];
            for object in &assets.objects {
                if matches!(object.kind, super::assets::ObjectKind::Image(_)) {
                    for (axis, max) in extent.iter_mut().enumerate() {
                        *max = max.max(object.origin[axis].abs() + object.size[axis] / 2.0);
                    }
                }
            }
            let dimensions = if extent.iter().all(|value| *value > 0.0) {
                extent.map(|value| (value * 2.0).ceil())
            } else {
                [window.width() as f32, window.height() as f32]
            };
            ensure!(
                dimensions
                    .iter()
                    .all(|value| value.is_finite() && (1.0..=16384.0).contains(value)),
                "Scene auto projection exceeds dimension budget"
            );
            assets.document.width = dimensions[0] as u32;
            assets.document.height = dimensions[1] as u32;
        }
        window.make_current()?;
        context.load_gl_functions();
        let mut major = 0;
        let mut minor = 0;
        unsafe {
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        }
        ensure!(
            major > 3 || major == 3 && minor >= 3,
            "Scene requires OpenGL 3.3; found {major}.{minor}"
        );
        let scripted = assets.objects.iter().any(|object| {
            object.properties.as_object().is_some_and(|properties| {
                properties
                    .values()
                    .any(|value| value.get("script").is_some())
            })
        });
        let scripts = if scripted {
            Some(super::script::Scripts::new(&assets.objects, properties)?)
        } else {
            None
        };
        let sounds = super::sound::load(&assets)?;
        let draw = DrawAssets::new(&assets, resources, seed)?;
        let target = Target::new(window.width().try_into()?, window.height().try_into()?)?;
        Ok(Self {
            assets,
            draw: Some(draw),
            target: Some(target),
            window,
            _context: context,
            closed: false,
            epoch: std::time::Instant::now(),
            clock: super::clock::AnimationClock::default(),
            sounds,
            audio: (true, true, 0.0),
            pending_audio: None,
            scripts,
            script_started: false,
        })
    }
    pub fn set_project_properties(&mut self, properties: &serde_json::Value) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        ensure!(
            properties.is_object() && serde_json::to_vec(properties)?.len() <= 1024 * 1024,
            "Invalid Scene project property budget"
        );
        if let Some(scripts) = &self.scripts {
            scripts.set_properties(properties)?;
        }
        Ok(())
    }
    /// Each snapshot is consumed by one render; absent inputs become silence.
    pub fn set_spectrum(&mut self, snapshot: &crate::audio::AudioSnapshot) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        self.pending_audio = Some(snapshot.clone());
        Ok(())
    }
    fn tick_scripts(&mut self, delta: f64, audio: &crate::audio::AudioSnapshot) -> Result<()> {
        let Some(scripts) = self.scripts.as_mut() else {
            return Ok(());
        };
        scripts.set_audio(audio)?;
        let updates = scripts.tick(self.clock.elapsed(), delta)?;
        for (id, properties) in updates {
            let index = *self
                .assets
                .object_index
                .get(&id)
                .context("SceneScript layer unavailable")?;
            let prior = &self.assets.objects[index];
            let mut value = prior.properties.clone();
            for name in [
                "origin",
                "scale",
                "angles",
                "size",
                "visible",
                "color",
                "alpha",
                "brightness",
                "text",
                "pointsize",
            ] {
                if let Some(property) = properties.get(name) {
                    value[name] = property.clone();
                }
            }
            let candidate =
                super::assets::Object::parse(&value).context("SceneScript layer update")?;
            if let super::assets::ObjectKind::Text(_) = &candidate.kind {
                let changed = ["text", "pointsize", "size"]
                    .iter()
                    .any(|name| candidate.properties[*name] != prior.properties[*name]);
                if changed {
                    let text = super::text::value(&candidate)?.to_string();
                    // set_text validates and uploads before replacing the old coverage.
                    let old = self.assets.objects[index].clone();
                    self.assets.objects[index] = candidate;
                    if let Err(error) = self.set_text(id, &text, &super::text::CachedOnly) {
                        self.assets.objects[index] = old;
                        return Err(error);
                    }
                    continue;
                }
            }
            self.assets.objects[index] = candidate;
        }
        Ok(())
    }
    /// Rasterize/upload a candidate before publishing text and replacing its texture.
    pub fn set_text(&mut self, id: i64, text: &str, resources: &dyn SceneResources) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        let index = *self
            .assets
            .object_index
            .get(&id)
            .context("Scene text layer not found")?;
        let mut object = self.assets.objects[index].clone();
        ensure!(
            matches!(object.kind, super::assets::ObjectKind::Text(_)),
            "Scene layer is not text"
        );
        object.properties["text"] = serde_json::Value::String(text.into());
        object.kind = super::assets::ObjectKind::Text(serde_json::Value::String(text.into()));
        let texture = super::text::raster(&object, text, resources, &mut self.assets.fonts)?;
        let name = format!("materials/__rust_text_{id}.tex");
        let total: usize = self
            .assets
            .textures
            .iter()
            .filter(|(key, _)| **key != name)
            .map(|(_, texture)| {
                texture
                    .images
                    .iter()
                    .flatten()
                    .map(|mip| mip.rgba.len())
                    .sum::<usize>()
            })
            .sum();
        ensure!(
            total
                .checked_add(texture.images[0][0].rgba.len())
                .is_some_and(|v| v <= 256 * 1024 * 1024),
            "Scene texture cache exceeds 256 MiB"
        );
        self.window.make_current()?;
        let candidate = super::gpu::TextureUpload::new(&texture)?;
        let draw = self
            .draw
            .as_mut()
            .context("Scene draw assets unavailable")?;
        draw.replace_texture(name.clone(), candidate);
        let model = self
            .assets
            .models
            .get_mut(&self.assets.text_models[&id])
            .context("Scene text model unavailable")?;
        model.width = Some(texture.width);
        model.height = Some(texture.height);
        self.assets.textures.insert(name, texture);
        self.assets.objects[index] = object;
        Ok(())
    }
    pub fn set_audio(&mut self, paused: bool, muted: bool, volume: f32) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        ensure!(
            volume.is_finite() && (0.0..=1.0).contains(&volume),
            "Invalid Scene volume"
        );
        for stream in &self.sounds {
            stream.controls(paused, muted, volume)?;
        }
        self.audio = (paused, muted, volume);
        self.set_paused(paused)
    }
    pub fn set_paused(&mut self, paused: bool) -> Result<()> {
        self.set_paused_at(paused, self.epoch.elapsed().as_secs_f64())
    }
    pub fn set_paused_at(&mut self, paused: bool, monotonic: f64) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        let mut candidate = self.clock.clone();
        candidate.set_paused(paused, monotonic)?;
        for stream in &self.sounds {
            stream.controls(paused, self.audio.1, self.audio.2)?;
        }
        self.audio.0 = paused;
        self.clock = candidate;
        Ok(())
    }
    pub fn render(&mut self, width: u32, height: u32, framebuffer: u32) -> Result<()> {
        self.render_at(
            width,
            height,
            framebuffer,
            self.epoch.elapsed().as_secs_f64(),
        )
    }
    pub fn render_at(
        &mut self,
        width: u32,
        height: u32,
        framebuffer: u32,
        monotonic: f64,
    ) -> Result<()> {
        ensure!(!self.closed, "Scene renderer is closed");
        let delta = self.clock.tick(monotonic)?;
        let audio = self.pending_audio.take().unwrap_or_default();
        self.window.make_current()?;
        let _state = State::capture();
        if !self.clock.is_paused() && (delta > 0.0 || !self.script_started) {
            self.tick_scripts(delta, &audio)?;
            self.script_started = true;
        }

        if self
            .target
            .as_ref()
            .map(|target| (target.width, target.height))
            != Some((width, height))
        {
            let candidate = Target::new(width, height)?;
            self.target = Some(candidate);
        }
        let target = self
            .target
            .as_ref()
            .context("Scene target is unavailable")?;
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target.framebuffer);
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::Disable(gl::SCISSOR_TEST);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            let color = self.assets.document.clear_color;
            gl::ClearColor(color[0], color[1], color[2], 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.draw
            .as_mut()
            .context("Scene draw assets unavailable")?
            .render(&self.assets, target, self.clock.elapsed(), delta, &audio)?;
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, target.framebuffer);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer);
            gl::BlitFramebuffer(
                0,
                0,
                width as i32,
                height as i32,
                0,
                0,
                width as i32,
                height as i32,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        if !self.closed {
            self.window.make_current()?;
            self.scripts = None;
            self.pending_audio = None;
            self.sounds.clear();
            self.draw = None;
            self.target = None;
            self.closed = true;
        }
        Ok(())
    }
}
impl Drop for RustSceneRenderer {
    fn drop(&mut self) {
        if let Err(error) = self.close().or_else(|_| self.close()) {
            // Context deletion eventually frees the handles; never issue GL
            // calls into another current context on a failed rebind.
            if let Some(draw) = self.draw.take() {
                std::mem::forget(draw);
            }
            if let Some(target) = self.target.take() {
                std::mem::forget(target);
            }
            tracing::error!("Cannot bind Scene context for cleanup: {error}");
        }
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
