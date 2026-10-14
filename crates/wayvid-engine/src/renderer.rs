//! Common rendering controls and project-aware renderer selection.
use std::path::Path;
use std::rc::Rc;

use anyhow::{bail, Context, Result};
use wayvid_library::project::{discover_project, ProjectResources};
use wayvid_library::{CompatibilityStatus, SourceType, SteamLibrary, WallpaperType};

use crate::egl::{EglContext, EglWindow};
use crate::mpv::{MpvPlayer, VideoConfig};
use crate::scene::runtime::RustSceneRenderer;
use crate::types::OutputInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererCapabilities {
    pub video_settings: bool,
    pub project_properties: bool,
    pub audio_visualization: bool,
}

pub trait WallpaperRenderer {
    /// Returns true only after rendering an available frame into the target.
    fn render(&mut self, width: i32, height: i32, framebuffer: i32) -> Result<bool>;
    fn pause(&mut self) -> Result<()>;
    fn resume(&mut self) -> Result<()>;
    fn set_volume(&mut self, volume: f32) -> Result<()>;
    fn update_config(&mut self, config: &VideoConfig) -> Result<()>;
    fn set_properties(&mut self, _properties: &serde_json::Value) -> Result<()> {
        bail!("This renderer does not support project properties")
    }
    fn close(&mut self) -> Result<()> {
        Ok(())
    }
    fn capabilities(&self) -> RendererCapabilities;
    fn is_web(&self) -> bool {
        false
    }
}

impl WallpaperRenderer for MpvPlayer {
    fn render(&mut self, w: i32, h: i32, fbo: i32) -> Result<bool> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo as u32);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Viewport(0, 0, w, h);
        }
        MpvPlayer::render(self, w, h, fbo)
    }
    fn pause(&mut self) -> Result<()> {
        MpvPlayer::pause(self)
    }
    fn resume(&mut self) -> Result<()> {
        MpvPlayer::resume(self)
    }
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        MpvPlayer::set_volume(self, f64::from(volume) * 100.0)
    }
    fn update_config(&mut self, config: &VideoConfig) -> Result<()> {
        MpvPlayer::update_config(self, config)
    }
    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            video_settings: true,
            project_properties: false,
            audio_visualization: false,
        }
    }
}

struct Scene {
    renderer: RustSceneRenderer,
    paused: bool,
    presented: bool,
    config: VideoConfig,
}
impl WallpaperRenderer for Scene {
    fn render(&mut self, width: i32, height: i32, framebuffer: i32) -> Result<bool> {
        if self.paused && self.presented {
            return Ok(false);
        }
        self.renderer.render(
            width.try_into()?,
            height.try_into()?,
            framebuffer.try_into()?,
        )?;
        self.presented = true;
        Ok(true)
    }
    fn pause(&mut self) -> Result<()> {
        self.renderer
            .set_audio(true, self.config.mute, self.config.volume as f32)?;
        self.paused = true;
        Ok(())
    }
    fn resume(&mut self) -> Result<()> {
        self.renderer
            .set_audio(false, self.config.mute, self.config.volume as f32)?;
        self.paused = false;
        Ok(())
    }
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.config.volume = f64::from(volume);
        self.renderer
            .set_audio(self.paused, self.config.mute, volume)
    }
    fn update_config(&mut self, config: &VideoConfig) -> Result<()> {
        self.renderer
            .set_audio(self.paused, config.mute, config.volume as f32)?;
        self.config = config.clone();
        Ok(())
    }
    fn close(&mut self) -> Result<()> {
        self.renderer.close()
    }
    fn capabilities(&self) -> RendererCapabilities {
        // Current Scene delivery defers shared audio input and live property controls.
        RendererCapabilities {
            video_settings: false,
            project_properties: false,
            audio_visualization: false,
        }
    }
}

/// A source is resolved before mpv is constructed. Project manifests are never
/// interpreted as video files, including legacy directory/entry-path callers.
pub fn create_renderer(
    path: &Path,
    config: &VideoConfig,
    output: &OutputInfo,
    context: &EglContext,
    window: Rc<EglWindow>,
) -> Result<Box<dyn WallpaperRenderer>> {
    let root = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
            .filter(|root| root.join("project.json").is_file())
    };
    let mut media_path = path.to_path_buf();
    if let Some(root) = root.filter(|root| root.join("project.json").is_file()) {
        let shared = SteamLibrary::try_discover()
            .map(|steam| steam.shared_asset_roots())
            .unwrap_or_default();
        let item = discover_project(root, SourceType::LocalDirectory, None, shared)?;
        if matches!(
            item.compatibility,
            CompatibilityStatus::Invalid | CompatibilityStatus::Unsupported
        ) {
            bail!(
                "{}",
                item.compatibility_reason
                    .unwrap_or_else(|| "Invalid wallpaper project".into())
            );
        }
        match item.wallpaper_type {
            WallpaperType::Scene => {
                let project = item.project.context("Scene project source missing")?;
                let resources = ProjectResources::new(&project)?;
                let renderer = RustSceneRenderer::new_with_properties(
                    project.entry.as_deref().context("Scene entry missing")?,
                    &resources,
                    context.clone(),
                    window,
                    0,
                    &project.properties,
                )?;
                let mut scene = Scene {
                    renderer,
                    paused: false,
                    presented: false,
                    config: config.clone(),
                };
                scene.update_config(config)?;
                return Ok(Box::new(scene));
            }
            WallpaperType::Web => {
                let project = item.project.context("Web project source missing")?;
                return Ok(Box::new(crate::web::WebRenderer::new(
                    &project.root,
                    project.entry.as_deref().context("Web entry missing")?,
                    output.width,
                    output.height,
                    config,
                    &project.properties,
                )?));
            }
            WallpaperType::Video | WallpaperType::Image | WallpaperType::Gif => {
                media_path = item.source_path
            }
            WallpaperType::Unsupported => bail!("Unsupported project type"),
        }
    }
    std::ffi::CString::new(media_path.to_string_lossy().as_bytes())?;
    let extension = media_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    anyhow::ensure!(
        matches!(
            extension.as_str(),
            "mp4"
                | "mkv"
                | "webm"
                | "avi"
                | "mov"
                | "m4v"
                | "wmv"
                | "flv"
                | "png"
                | "jpg"
                | "jpeg"
                | "webp"
                | "bmp"
                | "tiff"
                | "tif"
                | "gif"
        ),
        "Unsupported wallpaper source type"
    );
    anyhow::ensure!(media_path.is_file(), "Wallpaper media source is missing");
    let mut config = config.clone();
    config.source = media_path.to_string_lossy().into_owned();
    let mut player = MpvPlayer::new(&config, output)?;
    player.init_render_context(context)?;
    Ok(Box::new(player))
}
