//! Flutter-owned service facade.
//!
//! This module deliberately contains no persistence, CLI, IPC, or tray code.
//! Flutter owns state in Isar and uses this facade for Rust-side work.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use wayvid_engine::{EngineConfig, VideoConfig};
use wayvid_library::{FolderScanner, SteamLibrary, ThumbnailGenerator};

use crate::engine::EngineController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BridgeError {}

impl From<anyhow::Error> for BridgeError {
    fn from(error: anyhow::Error) -> Self {
        Self::message("operation_failed", error.to_string())
    }
}

impl BridgeError {
    fn message(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperMetadataDto {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub duration_secs: Option<f64>,
    pub resolution_width: Option<u32>,
    pub resolution_height: Option<u32>,
    pub file_size: Option<u64>,
    pub workshop_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperDto {
    pub id: String,
    pub name: String,
    pub source_path: String,
    pub thumbnail_path: Option<String>,
    pub source_type: String,
    pub wallpaper_category: String,
    pub wallpaper_type: String,
    pub metadata: WallpaperMetadataDto,
    pub added_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDto {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub primary: bool,
    pub current_wallpaper: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfigDto {
    pub volume: f64,
    pub fps_limit: Option<u32>,
    pub loop_playback: bool,
    pub layout: String,
    pub hwdec: String,
    pub mute: bool,
    pub start_time: f64,
    pub playback_rate: f64,
    pub hdr_mode: String,
    pub tone_mapping_algorithm: String,
    pub tone_mapping_param: f64,
    pub tone_mapping_mode: String,
    pub tone_mapping_compute_peak: bool,
    pub auto_play: bool,
    pub pause_on_battery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub workshop_available: bool,
    pub engine_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewDto {
    pub image_path: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    EngineStarted,
    EngineStopped,
    WallpaperApplied { output: String, path: String },
    WallpaperCleared { output: String },
    OutputsChanged { outputs: Vec<MonitorDto> },
    Error { code: String, message: String },
}

struct ServiceInner {
    engine: EngineController,
    events: VecDeque<ServiceEvent>,
}

#[derive(Clone)]
pub struct WayvidService {
    inner: Arc<Mutex<ServiceInner>>,
}

impl WayvidService {
    pub fn new() -> Result<Self, BridgeError> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("wayvid_gui=info".parse().unwrap()),
            )
            .try_init()
            .ok();

        Ok(Self {
            inner: Arc::new(Mutex::new(ServiceInner {
                engine: EngineController::new(),
                events: VecDeque::new(),
            })),
        })
    }

    pub fn initialize(&self) -> Result<ServiceInfo, BridgeError> {
        Ok(ServiceInfo {
            workshop_available: SteamLibrary::try_discover().is_some(),
            engine_running: self.lock()?.engine.is_running(),
        })
    }

    pub async fn scan_folder(&self, path: String) -> Result<Vec<WallpaperDto>, BridgeError> {
        let path_buf = PathBuf::from(path);
        let items = tokio::task::spawn_blocking(move || {
            FolderScanner::new()
                .scan_folder_parallel(&path_buf, true)
                .map_err(BridgeError::from)
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))??;
        Ok(items.iter().map(wallpaper_to_dto).collect())
    }

    pub async fn scan_workshop(&self) -> Result<Vec<WallpaperDto>, BridgeError> {
        let items = tokio::task::spawn_blocking(|| {
            let mut scanner =
                wayvid_library::WorkshopScanner::discover().map_err(BridgeError::from)?;
            scanner
                .scan_all()
                .map_err(|error| BridgeError::message("scan_failed", error.to_string()))
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))??;
        Ok(items.iter().map(wallpaper_to_dto).collect())
    }

    pub async fn load_preview(
        &self,
        wallpaper_id: String,
        path: String,
        wallpaper_type: String,
        width: u32,
        height: u32,
    ) -> Result<PreviewDto, BridgeError> {
        let source = PathBuf::from(path);
        if wallpaper_type == "image" && is_direct_image(&source) {
            return Ok(PreviewDto {
                image_path: Some(source.to_string_lossy().to_string()),
                bytes: Vec::new(),
            });
        }

        let cache_path = preview_cache_path(&wallpaper_id);
        if let Ok(bytes) = tokio::fs::read(&cache_path).await {
            return Ok(PreviewDto {
                image_path: None,
                bytes,
            });
        }

        let bytes = tokio::task::spawn_blocking(move || {
            ThumbnailGenerator::with_size(width, height)
                .generate(&source)
                .map(|result| result.data)
                .map_err(|error| BridgeError::message("preview_failed", error.to_string()))
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))??;

        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        tokio::fs::write(cache_path, &bytes).await.ok();
        Ok(PreviewDto {
            image_path: None,
            bytes,
        })
    }

    pub async fn refresh_monitors(&self) -> Result<Vec<MonitorDto>, BridgeError> {
        let mut guard = self.lock()?;
        if guard.engine.is_running() {
            guard
                .engine
                .wait_for_outputs(Duration::from_secs(5))
                .map_err(|message| BridgeError::message("monitors_unavailable", message))?;
            let mut outputs: Vec<_> = guard
                .engine
                .outputs()
                .iter()
                .map(monitor_from_engine)
                .collect();
            outputs.sort_by(|a, b| a.name.cmp(&b.name));
            if let Some(first) = outputs.first_mut() {
                first.primary = true;
            }
            Ok(outputs)
        } else if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("x11") {
            let mut outputs: Vec<_> = wayvid_engine::discover_x11_outputs()
                .map_err(|error| BridgeError::message("monitors_unavailable", error.to_string()))?
                .iter()
                .map(monitor_from_engine)
                .collect();
            outputs.sort_by(|a, b| a.name.cmp(&b.name));
            if let Some(first) = outputs.first_mut() {
                first.primary = true;
            }
            Ok(outputs)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn create_engine(&self, config: EngineConfigDto) -> Result<(), BridgeError> {
        let mut guard = self.lock()?;
        if !guard.engine.is_running() {
            guard
                .engine
                .start(config_to_engine(config))
                .map_err(|message| BridgeError::message("engine_start_failed", message))?;
            guard.events.push_back(ServiceEvent::EngineStarted);
        }
        Ok(())
    }

    pub async fn update_engine_config(&self, config: EngineConfigDto) -> Result<(), BridgeError> {
        self.lock()?
            .engine
            .update_config(config_to_engine(config))
            .map_err(|message| BridgeError::message("engine_config_failed", message))
    }

    pub async fn stop_engine(&self) -> Result<(), BridgeError> {
        let mut guard = self.lock()?;
        guard.engine.stop();
        guard.events.push_back(ServiceEvent::EngineStopped);
        Ok(())
    }

    pub async fn apply_wallpaper(
        &self,
        path: String,
        output: Option<String>,
    ) -> Result<(), BridgeError> {
        let mut guard = self.lock()?;
        guard
            .engine
            .wait_for_outputs(Duration::from_secs(5))
            .map_err(|message| BridgeError::message("engine_not_ready", message))?;
        guard
            .engine
            .apply_wallpaper(output.clone(), PathBuf::from(&path))
            .map_err(|message| BridgeError::message("apply_failed", message))?;
        guard.events.push_back(ServiceEvent::WallpaperApplied {
            output: output.unwrap_or_else(|| "all".to_string()),
            path,
        });
        Ok(())
    }

    pub async fn clear_wallpaper(&self, output: Option<String>) -> Result<(), BridgeError> {
        self.lock()?
            .engine
            .clear_wallpaper(output)
            .map_err(|message| BridgeError::message("clear_failed", message))
    }

    pub async fn pause(&self, output: Option<String>) -> Result<(), BridgeError> {
        self.lock()?
            .engine
            .pause(output)
            .map_err(|message| BridgeError::message("pause_failed", message))
    }

    pub async fn resume(&self, output: Option<String>) -> Result<(), BridgeError> {
        self.lock()?
            .engine
            .resume(output)
            .map_err(|message| BridgeError::message("resume_failed", message))
    }

    pub fn poll_events(&self) -> Result<Vec<ServiceEvent>, BridgeError> {
        let mut guard = self.lock()?;
        for event in guard.engine.poll_events() {
            match event {
                wayvid_engine::EngineEvent::Started => {
                    guard.events.push_back(ServiceEvent::EngineStarted)
                }
                wayvid_engine::EngineEvent::Stopped => {
                    guard.events.push_back(ServiceEvent::EngineStopped)
                }
                wayvid_engine::EngineEvent::WallpaperApplied { output, path } => {
                    guard.events.push_back(ServiceEvent::WallpaperApplied {
                        output,
                        path: path.to_string_lossy().to_string(),
                    })
                }
                wayvid_engine::EngineEvent::WallpaperCleared { output } => guard
                    .events
                    .push_back(ServiceEvent::WallpaperCleared { output }),
                wayvid_engine::EngineEvent::OutputAdded(_)
                | wayvid_engine::EngineEvent::OutputRemoved(_) => {
                    let mut outputs: Vec<_> = guard
                        .engine
                        .outputs()
                        .iter()
                        .map(monitor_from_engine)
                        .collect();
                    outputs.sort_by(|a, b| a.name.cmp(&b.name));
                    if let Some(first) = outputs.first_mut() {
                        first.primary = true;
                    }
                    guard
                        .events
                        .push_back(ServiceEvent::OutputsChanged { outputs })
                }
                wayvid_engine::EngineEvent::Error(message) => {
                    guard.events.push_back(ServiceEvent::Error {
                        code: "engine_error".to_string(),
                        message,
                    })
                }
                wayvid_engine::EngineEvent::OutputsList(_)
                | wayvid_engine::EngineEvent::Status(_) => {}
            }
        }
        Ok(guard.events.drain(..).collect())
    }

    pub async fn open_url(&self, url: String) -> Result<(), BridgeError> {
        open::that(url).map_err(|error| BridgeError::message("open_url_failed", error.to_string()))
    }

    pub async fn shutdown(&self) -> Result<(), BridgeError> {
        self.lock()?.engine.stop();
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ServiceInner>, BridgeError> {
        self.inner
            .lock()
            .map_err(|_| BridgeError::message("service_poisoned", "Rust service lock poisoned"))
    }
}

fn config_to_engine(config: EngineConfigDto) -> EngineConfig {
    EngineConfig {
        video: VideoConfig {
            loop_playback: config.loop_playback,
            layout: parse_layout(&config.layout),
            hwdec: parse_hwdec(&config.hwdec),
            mute: config.mute,
            volume: config.volume.clamp(0.0, 1.0),
            start_time: config.start_time.max(0.0),
            playback_rate: config.playback_rate.clamp(0.1, 10.0),
            hdr_mode: parse_hdr_mode(&config.hdr_mode),
            tone_mapping: wayvid_engine::ToneMappingConfig {
                algorithm: parse_tone_mapping_algorithm(&config.tone_mapping_algorithm),
                param: config.tone_mapping_param,
                compute_peak: config.tone_mapping_compute_peak,
                mode: config.tone_mapping_mode,
            },
            ..VideoConfig::default()
        },
        auto_play: config.auto_play,
        fps_limit: config.fps_limit.filter(|value| *value > 0),
        pause_on_battery: config.pause_on_battery,
    }
}

fn parse_layout(value: &str) -> wayvid_engine::LayoutMode {
    match value {
        "contain" => wayvid_engine::LayoutMode::Contain,
        "stretch" => wayvid_engine::LayoutMode::Stretch,
        "cover" => wayvid_engine::LayoutMode::Cover,
        "centre" | "center" => wayvid_engine::LayoutMode::Centre,
        _ => wayvid_engine::LayoutMode::Fill,
    }
}

fn parse_hwdec(value: &str) -> wayvid_engine::HwdecMode {
    match value {
        "force" => wayvid_engine::HwdecMode::Force,
        "none" | "no" => wayvid_engine::HwdecMode::No,
        _ => wayvid_engine::HwdecMode::Auto,
    }
}

fn parse_hdr_mode(value: &str) -> wayvid_engine::HdrMode {
    match value {
        "force" => wayvid_engine::HdrMode::Force,
        "disable" | "disabled" => wayvid_engine::HdrMode::Disable,
        _ => wayvid_engine::HdrMode::Auto,
    }
}

fn parse_tone_mapping_algorithm(value: &str) -> wayvid_engine::types::ToneMappingAlgorithm {
    match value {
        "mobius" => wayvid_engine::types::ToneMappingAlgorithm::Mobius,
        "reinhard" => wayvid_engine::types::ToneMappingAlgorithm::Reinhard,
        "bt2390" | "bt.2390" => wayvid_engine::types::ToneMappingAlgorithm::Bt2390,
        "clip" => wayvid_engine::types::ToneMappingAlgorithm::Clip,
        _ => wayvid_engine::types::ToneMappingAlgorithm::Hable,
    }
}

fn wallpaper_to_dto(item: &wayvid_library::WallpaperItem) -> WallpaperDto {
    let (resolution_width, resolution_height) = item
        .metadata
        .resolution
        .map(|(width, height)| (Some(width), Some(height)))
        .unwrap_or((None, None));
    WallpaperDto {
        id: item.id.clone(),
        name: item.name.clone(),
        source_path: item.source_path.to_string_lossy().to_string(),
        thumbnail_path: item
            .thumbnail_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        source_type: item.source_type.as_str().to_string(),
        wallpaper_category: match item.wallpaper_type {
            wayvid_library::WallpaperType::Scene => "scene",
            wayvid_library::WallpaperType::Video
            | wayvid_library::WallpaperType::Gif
            | wayvid_library::WallpaperType::Image => "video",
        }
        .to_string(),
        wallpaper_type: item.wallpaper_type.as_str().to_string(),
        metadata: WallpaperMetadataDto {
            title: item.metadata.title.clone(),
            author: item.metadata.author.clone(),
            description: item.metadata.description.clone(),
            tags: item.metadata.tags.clone(),
            duration_secs: item.metadata.duration_secs,
            resolution_width,
            resolution_height,
            file_size: item.metadata.file_size,
            workshop_id: item.metadata.workshop_id,
        },
        added_at: item.added_at.to_rfc3339(),
        last_used: item.last_used.as_ref().map(|date| date.to_rfc3339()),
    }
}

fn monitor_from_engine(info: &wayvid_engine::OutputInfo) -> MonitorDto {
    MonitorDto {
        name: info.name.clone(),
        width: info.width.max(0) as u32,
        height: info.height.max(0) as u32,
        x: info.position.0,
        y: info.position.1,
        scale: info.scale,
        primary: false,
        current_wallpaper: None,
    }
}

fn is_direct_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "bmp")
    )
}

fn preview_cache_path(id: &str) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    id.hash(&mut hasher);
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("wayvid")
        .join("previews")
        .join(format!("{:x}.webp", hasher.finish()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayvid_engine::types::ToneMappingAlgorithm;
    use wayvid_engine::{HdrMode, HwdecMode, LayoutMode};
    use wayvid_library::{SourceType, WallpaperItem, WallpaperType};

    fn config_dto() -> EngineConfigDto {
        EngineConfigDto {
            volume: 0.5,
            fps_limit: Some(30),
            loop_playback: true,
            layout: "contain".into(),
            hwdec: "force".into(),
            mute: false,
            start_time: 2.0,
            playback_rate: 1.5,
            hdr_mode: "disabled".into(),
            tone_mapping_algorithm: "mobius".into(),
            tone_mapping_param: 0.4,
            tone_mapping_mode: "rgb".into(),
            tone_mapping_compute_peak: false,
            auto_play: false,
            pause_on_battery: true,
        }
    }

    #[test]
    fn bridge_error_display_and_conversion() {
        let error = BridgeError::message("code", "boom");
        assert_eq!(error.to_string(), "code: boom");
        let converted: BridgeError = anyhow::anyhow!("failed").into();
        assert_eq!(converted.code, "operation_failed");
        assert_eq!(converted.message, "failed");
    }

    #[test]
    fn converts_engine_config() {
        let config = config_to_engine(config_dto());
        assert_eq!(config.video.layout, LayoutMode::Contain);
        assert_eq!(config.video.hwdec, HwdecMode::Force);
        assert_eq!(config.video.hdr_mode, HdrMode::Disable);
        assert_eq!(
            config.video.tone_mapping.algorithm,
            ToneMappingAlgorithm::Mobius
        );
        assert_eq!(config.video.tone_mapping.param, 0.4);
        assert_eq!(config.video.tone_mapping.mode, "rgb");
        assert!(!config.video.tone_mapping.compute_peak);
        assert_eq!(config.video.volume, 0.5);
        assert_eq!(config.video.playback_rate, 1.5);
        assert_eq!(config.fps_limit, Some(30));
        assert!(!config.auto_play);
        assert!(config.pause_on_battery);
    }

    #[test]
    fn clamps_out_of_range_engine_config() {
        let config = config_to_engine(EngineConfigDto {
            volume: 3.0,
            fps_limit: Some(0),
            start_time: -5.0,
            playback_rate: 100.0,
            ..config_dto()
        });
        assert_eq!(config.video.volume, 1.0);
        assert_eq!(config.video.start_time, 0.0);
        assert_eq!(config.video.playback_rate, 10.0);
        assert_eq!(config.fps_limit, None);
    }

    #[test]
    fn parses_string_enums_with_fallbacks() {
        assert_eq!(parse_layout("stretch"), LayoutMode::Stretch);
        assert_eq!(parse_layout("cover"), LayoutMode::Cover);
        assert_eq!(parse_layout("center"), LayoutMode::Centre);
        assert_eq!(parse_layout("centre"), LayoutMode::Centre);
        assert_eq!(parse_layout("bogus"), LayoutMode::Fill);

        assert_eq!(parse_hwdec("none"), HwdecMode::No);
        assert_eq!(parse_hwdec("no"), HwdecMode::No);
        assert_eq!(parse_hwdec("auto"), HwdecMode::Auto);

        assert_eq!(parse_hdr_mode("force"), HdrMode::Force);
        assert_eq!(parse_hdr_mode("disable"), HdrMode::Disable);
        assert_eq!(parse_hdr_mode(""), HdrMode::Auto);

        assert_eq!(
            parse_tone_mapping_algorithm("reinhard"),
            ToneMappingAlgorithm::Reinhard
        );
        assert_eq!(
            parse_tone_mapping_algorithm("bt.2390"),
            ToneMappingAlgorithm::Bt2390
        );
        assert_eq!(
            parse_tone_mapping_algorithm("bt2390"),
            ToneMappingAlgorithm::Bt2390
        );
        assert_eq!(
            parse_tone_mapping_algorithm("clip"),
            ToneMappingAlgorithm::Clip
        );
        assert_eq!(
            parse_tone_mapping_algorithm("other"),
            ToneMappingAlgorithm::Hable
        );
    }

    #[test]
    fn converts_wallpaper_item_to_dto() {
        let mut item = WallpaperItem::new(
            PathBuf::from("/walls/scene/project.json"),
            "Scene".into(),
            SourceType::SteamWorkshop,
            WallpaperType::Scene,
        );
        item.thumbnail_path = Some(PathBuf::from("/walls/scene/preview.jpg"));
        item.metadata.resolution = Some((1920, 1080));
        item.metadata.workshop_id = Some(42);
        item.metadata.tags = vec!["anime".into()];
        item.last_used = Some(item.added_at);

        let dto = wallpaper_to_dto(&item);
        assert_eq!(dto.id, item.id);
        assert_eq!(dto.source_path, "/walls/scene/project.json");
        assert_eq!(
            dto.thumbnail_path.as_deref(),
            Some("/walls/scene/preview.jpg")
        );
        assert_eq!(dto.source_type, "workshop");
        assert_eq!(dto.wallpaper_category, "scene");
        assert_eq!(dto.wallpaper_type, "scene");
        assert_eq!(dto.metadata.resolution_width, Some(1920));
        assert_eq!(dto.metadata.resolution_height, Some(1080));
        assert_eq!(dto.metadata.workshop_id, Some(42));
        assert_eq!(dto.metadata.tags, vec!["anime".to_string()]);
        assert_eq!(dto.last_used, Some(dto.added_at.clone()));

        let gif = WallpaperItem::new(
            PathBuf::from("/walls/a.gif"),
            "Gif".into(),
            SourceType::LocalFile,
            WallpaperType::Gif,
        );
        let dto = wallpaper_to_dto(&gif);
        assert_eq!(dto.wallpaper_category, "video");
        assert_eq!(dto.wallpaper_type, "gif");
        assert_eq!(dto.metadata.resolution_width, None);
        assert_eq!(dto.thumbnail_path, None);
        assert_eq!(dto.last_used, None);
    }

    #[test]
    fn converts_output_to_monitor() {
        let info = wayvid_engine::OutputInfo {
            name: "HDMI-A-1".into(),
            width: -1,
            height: 1080,
            scale: 1.25,
            position: (1920, 0),
            active: true,
            hdr_capabilities: Default::default(),
        };
        let monitor = monitor_from_engine(&info);
        assert_eq!(monitor.name, "HDMI-A-1");
        assert_eq!(monitor.width, 0);
        assert_eq!(monitor.height, 1080);
        assert_eq!((monitor.x, monitor.y), (1920, 0));
        assert_eq!(monitor.scale, 1.25);
        assert!(!monitor.primary);
    }

    #[test]
    fn detects_direct_images() {
        assert!(is_direct_image(Path::new("a.PNG")));
        assert!(is_direct_image(Path::new("/x/b.jpeg")));
        assert!(is_direct_image(Path::new("c.webp")));
        assert!(!is_direct_image(Path::new("d.mp4")));
        assert!(!is_direct_image(Path::new("noext")));
    }

    #[test]
    fn preview_cache_path_is_stable_per_id() {
        let first = preview_cache_path("abc");
        assert_eq!(first, preview_cache_path("abc"));
        assert_ne!(first, preview_cache_path("abd"));
        assert!(first.ends_with(first.file_name().unwrap()));
        assert_eq!(first.extension().unwrap(), "webp");
        assert!(first.parent().unwrap().ends_with("wayvid/previews"));
    }
}
