//! Common rendering controls and project-aware renderer selection.
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use wayvid_library::project::{discover_project, load_manifest, ProjectResources};
use wayvid_library::{CompatibilityStatus, SourceType, SteamLibrary, WallpaperType};

use crate::egl::{EglContext, EglWindow};
use crate::mpv::{MpvPlayer, VideoConfig};
use crate::scene::{SceneRenderer, SceneResources};
use crate::types::{LayoutMode, OutputInfo};

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

impl SceneResources for ProjectResources {
    fn exists(&self, name: &str) -> Result<bool> {
        ProjectResources::exists(self, name)
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        ProjectResources::read(self, name)
    }
}

struct Scene {
    native: SceneRenderer,
    previous: Instant,
    time: f64,
    paused: bool,
    presented: bool,
    config: VideoConfig,
}
impl WallpaperRenderer for Scene {
    fn render(&mut self, width: i32, height: i32, framebuffer: i32) -> Result<bool> {
        if self.paused && self.presented {
            return Ok(false);
        }
        let now = Instant::now();
        let delta = if self.presented {
            now.duration_since(self.previous).as_secs_f64().min(1.0)
        } else {
            0.0
        };
        self.previous = now;
        self.time += delta;
        self.native.render(
            width.try_into()?,
            height.try_into()?,
            framebuffer.try_into()?,
            self.time,
            delta,
            &[0.0; 64],
        )?;
        self.presented = true;
        Ok(true)
    }
    fn pause(&mut self) -> Result<()> {
        self.native
            .set_audio(true, self.config.mute, self.config.volume as f32)?;
        self.paused = true;
        Ok(())
    }
    fn resume(&mut self) -> Result<()> {
        self.native
            .set_audio(false, self.config.mute, self.config.volume as f32)?;
        self.previous = Instant::now();
        self.paused = false;
        Ok(())
    }
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.config.volume = f64::from(volume);
        self.native.set_audio(self.paused, self.config.mute, volume)
    }
    fn update_config(&mut self, config: &VideoConfig) -> Result<()> {
        self.native
            .set_audio(self.paused, config.mute, config.volume as f32)?;
        self.config = config.clone();
        Ok(())
    }
    fn close(&mut self) -> Result<()> {
        self.native.close()
    }
    fn capabilities(&self) -> RendererCapabilities {
        // Shared audio input and live property controls are still being implemented.
        RendererCapabilities {
            video_settings: false,
            project_properties: false,
            audio_visualization: false,
        }
    }
}

pub fn scene_library_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VARPAPER_SCENE_LIBRARY") {
        return Ok(PathBuf::from(path));
    }
    let executable = std::env::current_exe()?;
    let parent = executable
        .parent()
        .context("Missing executable directory")?;
    Ok(parent.join("lib").join("libvarpaper_scene.so"))
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
                let mut metadata = load_manifest(&project.root)?;
                metadata.title = Some(item.name);
                let manifest = serde_json::to_string(&metadata)?;
                let scaling = match config.layout {
                    LayoutMode::Contain => 1,
                    LayoutMode::Fill | LayoutMode::Cover => 2,
                    LayoutMode::Stretch => 3,
                    LayoutMode::Centre => 0,
                };
                let native = SceneRenderer::new(
                    &scene_library_path()?,
                    &manifest,
                    Box::new(ProjectResources::new(&project)?),
                    context.clone(),
                    window,
                    scaling,
                )?;
                let mut scene = Scene {
                    native,
                    previous: Instant::now(),
                    time: 0.0,
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
