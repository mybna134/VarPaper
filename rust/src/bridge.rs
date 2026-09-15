//! Stable Flutter-facing service facade.

use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use wayvid_core::{SourceType, WallpaperItem, WallpaperType};
use wayvid_library::{FolderScanner, LibraryDatabase, SteamLibrary, ThumbnailGenerator};

use crate::engine::{default_engine_config, EngineController};
use crate::settings::{AppSettings, AutostartManager};
use crate::single_instance;
use crate::tray::{SystemTray, TrayAction};

/// Error returned by all public bridge operations.
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
pub struct GuiSettingsDto {
    pub window_width: u32,
    pub window_height: u32,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub theme: String,
    pub language: String,
    pub renderer: String,
    pub sidebar_collapsed: bool,
    pub detail_panel_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSettingsDto {
    pub volume: f32,
    pub fps_limit: Option<u32>,
    pub preferred_monitor: Option<String>,
    pub loop_mode: bool,
    pub shuffle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerSettingsDto {
    pub pause_on_battery: bool,
    pub pause_on_fullscreen: bool,
    pub battery_fps_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDto {
    pub gui: GuiSettingsDto,
    pub playback: PlaybackSettingsDto,
    pub autostart_enabled: bool,
    pub restore_last_wallpaper: bool,
    pub power: PowerSettingsDto,
    pub library_folders: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsPatch {
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub minimize_to_tray: Option<bool>,
    pub start_minimized: Option<bool>,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub renderer: Option<String>,
    pub sidebar_collapsed: Option<bool>,
    pub detail_panel_visible: Option<bool>,
    pub volume: Option<f32>,
    pub fps_limit: Option<FpsLimitPatch>,
    pub pause_on_battery: Option<bool>,
    pub pause_on_fullscreen: Option<bool>,
    pub autostart_enabled: Option<bool>,
    pub restore_last_wallpaper: Option<bool>,
    pub library_folders: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FpsLimitPatch {
    Unlimited,
    Value(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationSnapshot {
    pub settings: SettingsDto,
    pub workshop_available: bool,
    pub engine_running: bool,
    pub tray_available: bool,
    pub another_instance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceEvent {
    EngineStarted,
    EngineStopped,
    WallpaperApplied { output: String, path: String },
    WallpaperCleared { output: String },
    OutputsChanged { outputs: Vec<MonitorDto> },
    ShowWindow,
    TrayAction { action: String },
    Error { code: String, message: String },
}

struct ServiceInner {
    settings: AppSettings,
    engine: EngineController,
    tray: Option<SystemTray>,
    wallpapers: Vec<WallpaperItem>,
    events: VecDeque<ServiceEvent>,
    monitors: Vec<MonitorDto>,
    another_instance: bool,
}

/// Long-lived Rust backend owned by the Flutter application.
#[derive(Clone)]
pub struct WayvidService {
    inner: Arc<Mutex<ServiceInner>>,
}

impl WayvidService {
    /// Construct the service without starting the playback engine.
    pub fn new(args: Vec<String>) -> Result<Self, BridgeError> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("wayvid_gui=info".parse().unwrap()),
            )
            .try_init()
            .ok();

        let mut settings = AppSettings::load().unwrap_or_default();
        let force_minimized = args.iter().any(|arg| arg == "--minimized");
        if force_minimized {
            settings.gui.start_minimized = true;
        }

        let another_instance = single_instance::is_another_instance_running();
        if another_instance {
            let _ = single_instance::request_show_window();
        }

        let tray = if settings.gui.minimize_to_tray {
            SystemTray::new()
        } else {
            None
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(ServiceInner {
                settings,
                engine: EngineController::new(),
                tray,
                wallpapers: Vec::new(),
                events: VecDeque::new(),
                monitors: Vec::new(),
                another_instance,
            })),
        })
    }

    pub fn initialize(&self) -> Result<InitializationSnapshot, BridgeError> {
        let guard = self.lock()?;
        Ok(InitializationSnapshot {
            settings: settings_to_dto(&guard.settings),
            workshop_available: SteamLibrary::try_discover().is_some(),
            engine_running: guard.engine.is_running(),
            tray_available: guard.tray.is_some(),
            another_instance: guard.another_instance,
        })
    }

    pub async fn load_library(&self) -> Result<Vec<WallpaperDto>, BridgeError> {
        let result = tokio::task::spawn_blocking(|| {
            let db = LibraryDatabase::open(LibraryDatabase::default_path())?;
            Ok::<_, anyhow::Error>(db.list_wallpapers(&wayvid_library::WallpaperFilter::default())?)
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))??;

        let result_dto = result.iter().map(wallpaper_to_dto).collect::<Vec<_>>();
        self.lock()?.wallpapers = result;
        Ok(result_dto)
    }

    pub async fn scan_folder(&self, path: String) -> Result<Vec<WallpaperDto>, BridgeError> {
        let path_buf = PathBuf::from(path.clone());
        let items = tokio::task::spawn_blocking(move || {
            let scanner = FolderScanner::new();
            let items = scanner.scan_folder_parallel(&path_buf, true)?;
            let db = LibraryDatabase::open(LibraryDatabase::default_path())?;
            db.add_folder(&path_buf, true)?;
            for item in &items {
                db.upsert_wallpaper(item)?;
            }
            Ok::<_, anyhow::Error>(items)
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))??;

        let dto = items.iter().map(wallpaper_to_dto).collect::<Vec<_>>();
        let mut guard = self.lock()?;
        merge_wallpapers(&mut guard.wallpapers, items);
        guard.settings.library.folders = guard
            .settings
            .library
            .folders
            .iter()
            .cloned()
            .chain([PathBuf::from(path)])
            .collect();
        guard.settings.save().map_err(BridgeError::from)?;
        Ok(dto)
    }

    pub async fn scan_workshop(&self) -> Result<Vec<WallpaperDto>, BridgeError> {
        let items = tokio::task::spawn_blocking(|| {
            let mut scanner = wayvid_library::WorkshopScanner::discover()?;
            scanner.scan_all().map_err(anyhow::Error::from)
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))??;

        let dto = items.iter().map(wallpaper_to_dto).collect::<Vec<_>>();
        let mut guard = self.lock()?;
        merge_wallpapers(&mut guard.wallpapers, items);
        Ok(dto)
    }

    pub async fn refresh_monitors(&self) -> Result<Vec<MonitorDto>, BridgeError> {
        let monitors = tokio::task::spawn_blocking(detect_monitors)
            .await
            .map_err(|error| BridgeError::message("join_failed", error.to_string()))?;
        let mut guard = self.lock()?;
        guard.monitors = monitors.clone();
        guard.events.push_back(ServiceEvent::OutputsChanged {
            outputs: monitors.clone(),
        });
        Ok(monitors)
    }

    pub async fn load_thumbnail(
        &self,
        wallpaper_id: String,
        path: String,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, BridgeError> {
        let cache_dir = thumbnail_cache_dir();
        let cache_path = thumbnail_cache_path(&cache_dir, &wallpaper_id);
        if let Ok(data) = tokio::fs::read(&cache_path).await {
            return Ok(data);
        }

        let source = PathBuf::from(path);
        let result = tokio::task::spawn_blocking(move || {
            ThumbnailGenerator::with_size(width, height)
                .generate(&source)
                .map(|result| result.data)
        })
        .await
        .map_err(|error| BridgeError::message("join_failed", error.to_string()))?
        .map_err(|error| BridgeError::message("thumbnail_failed", error.to_string()))?;

        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        tokio::fs::write(cache_path, &result).await.ok();
        Ok(result)
    }

    pub async fn start_engine(&self) -> Result<(), BridgeError> {
        let mut guard = self.lock()?;
        if guard.engine.is_running() {
            return Ok(());
        }
        let config = default_engine_config(&guard.settings);
        guard
            .engine
            .start(config)
            .map_err(|message| BridgeError::message("engine_start_failed", message))?;
        guard.settings.autostart.engine_running = true;
        guard.settings.save().map_err(BridgeError::from)?;
        guard.events.push_back(ServiceEvent::EngineStarted);
        Ok(())
    }

    pub async fn stop_engine(&self) -> Result<(), BridgeError> {
        let mut guard = self.lock()?;
        guard.engine.stop();
        guard.settings.autostart.engine_running = false;
        guard.settings.save().map_err(BridgeError::from)?;
        guard.events.push_back(ServiceEvent::EngineStopped);
        Ok(())
    }

    pub async fn apply_wallpaper(
        &self,
        wallpaper_id: String,
        output: Option<String>,
    ) -> Result<(), BridgeError> {
        if !self.lock()?.engine.is_running() {
            self.start_engine().await?;
        }
        let mut guard = self.lock()?;
        guard
            .engine
            .wait_for_outputs(Duration::from_secs(5))
            .map_err(|message| BridgeError::message("engine_not_ready", message))?;
        let wallpaper = guard
            .wallpapers
            .iter()
            .find(|item| item.id == wallpaper_id)
            .cloned()
            .ok_or_else(|| BridgeError::message("wallpaper_not_found", wallpaper_id.clone()))?;
        let path = wallpaper.source_path.clone();
        let output_name = output.clone().unwrap_or_else(|| "all".to_string());
        guard
            .engine
            .apply_wallpaper(output.clone(), path.clone())
            .map_err(|message| BridgeError::message("apply_failed", message))?;
        let monitors = guard.monitors.clone();
        save_wallpaper_state(&mut guard.settings, &monitors, output.as_deref(), &path);
        guard.settings.save().map_err(BridgeError::from)?;
        guard.events.push_back(ServiceEvent::WallpaperApplied {
            output: output_name,
            path: path.to_string_lossy().to_string(),
        });
        Ok(())
    }

    pub async fn clear_wallpaper(&self, output: Option<String>) -> Result<(), BridgeError> {
        let guard = self.lock()?;
        guard
            .engine
            .clear_wallpaper(output.clone())
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

    pub fn get_settings(&self) -> Result<SettingsDto, BridgeError> {
        Ok(settings_to_dto(&self.lock()?.settings))
    }

    pub fn update_settings(&self, patch: SettingsPatch) -> Result<SettingsDto, BridgeError> {
        let mut guard = self.lock()?;
        apply_settings_patch(&mut guard.settings, patch)?;
        guard.settings.save().map_err(BridgeError::from)?;
        Ok(settings_to_dto(&guard.settings))
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
                    });
                }
                wayvid_engine::EngineEvent::WallpaperCleared { output } => {
                    guard
                        .events
                        .push_back(ServiceEvent::WallpaperCleared { output });
                }
                wayvid_engine::EngineEvent::Error(message) => {
                    guard.events.push_back(ServiceEvent::Error {
                        code: "engine_error".to_string(),
                        message,
                    })
                }
                wayvid_engine::EngineEvent::OutputAdded(_)
                | wayvid_engine::EngineEvent::OutputRemoved(_)
                | wayvid_engine::EngineEvent::OutputsList(_)
                | wayvid_engine::EngineEvent::Status(_) => {}
            }
        }

        if guard.engine.check_show_window_request() {
            guard.events.push_back(ServiceEvent::ShowWindow);
        }
        let tray_actions = if let Some(tray) = &guard.tray {
            let mut actions = Vec::new();
            while let Some(action) = tray.try_recv_action() {
                actions.push(match action {
                    TrayAction::Show => "show",
                    TrayAction::Hide => "hide",
                    TrayAction::TogglePause => "toggle_pause",
                    TrayAction::Quit => "quit",
                });
            }
            actions
        } else {
            Vec::new()
        };
        for action in tray_actions {
            guard.events.push_back(ServiceEvent::TrayAction {
                action: action.to_string(),
            });
        }

        Ok(guard.events.drain(..).collect())
    }

    pub async fn open_url(&self, url: String) -> Result<(), BridgeError> {
        open::that(url).map_err(|error| BridgeError::message("open_url_failed", error.to_string()))
    }

    pub async fn shutdown(&self) -> Result<(), BridgeError> {
        let mut guard = self.lock()?;
        guard.engine.stop();
        if let Some(tray) = &guard.tray {
            tray.shutdown();
        }
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ServiceInner>, BridgeError> {
        self.inner
            .lock()
            .map_err(|_| BridgeError::message("service_poisoned", "Rust service lock poisoned"))
    }
}

fn settings_to_dto(settings: &AppSettings) -> SettingsDto {
    SettingsDto {
        gui: GuiSettingsDto {
            window_width: settings.gui.window_width,
            window_height: settings.gui.window_height,
            minimize_to_tray: settings.gui.minimize_to_tray,
            start_minimized: settings.gui.start_minimized,
            theme: settings.gui.theme.clone(),
            language: settings.gui.language.clone(),
            renderer: settings.gui.renderer.clone(),
            sidebar_collapsed: settings.gui.sidebar_collapsed,
            detail_panel_visible: settings.gui.detail_panel_visible,
        },
        playback: PlaybackSettingsDto {
            volume: settings.playback.volume,
            fps_limit: settings.playback.fps_limit,
            preferred_monitor: settings.playback.preferred_monitor.clone(),
            loop_mode: settings.playback.loop_mode,
            shuffle: settings.playback.shuffle,
        },
        autostart_enabled: AutostartManager::is_enabled(),
        restore_last_wallpaper: settings.autostart.restore_last_wallpaper,
        power: PowerSettingsDto {
            pause_on_battery: settings.power.pause_on_battery,
            pause_on_fullscreen: settings.power.pause_on_fullscreen,
            battery_fps_limit: settings.power.battery_fps_limit,
        },
        library_folders: settings
            .library
            .folders
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
    }
}

fn apply_settings_patch(
    settings: &mut AppSettings,
    patch: SettingsPatch,
) -> Result<(), BridgeError> {
    if let Some(value) = patch.window_width {
        settings.gui.window_width = value.max(800);
    }
    if let Some(value) = patch.window_height {
        settings.gui.window_height = value.max(600);
    }
    if let Some(value) = patch.minimize_to_tray {
        settings.gui.minimize_to_tray = value;
    }
    if let Some(value) = patch.start_minimized {
        settings.gui.start_minimized = value;
    }
    if let Some(value) = patch.theme {
        settings.gui.theme = value;
    }
    if let Some(value) = patch.language {
        settings.gui.language = value;
    }
    if let Some(value) = patch.renderer {
        settings.gui.renderer = value;
    }
    if let Some(value) = patch.sidebar_collapsed {
        settings.gui.sidebar_collapsed = value;
    }
    if let Some(value) = patch.detail_panel_visible {
        settings.gui.detail_panel_visible = value;
    }
    if let Some(value) = patch.volume {
        settings.playback.volume = value.clamp(0.0, 1.0);
    }
    if let Some(value) = patch.fps_limit {
        settings.playback.fps_limit = match value {
            FpsLimitPatch::Unlimited => None,
            FpsLimitPatch::Value(value) => Some(value),
        };
    }
    if let Some(value) = patch.pause_on_battery {
        settings.power.pause_on_battery = value;
    }
    if let Some(value) = patch.pause_on_fullscreen {
        settings.power.pause_on_fullscreen = value;
    }
    if let Some(value) = patch.autostart_enabled {
        AutostartManager::set_enabled(value)
            .map_err(|error| BridgeError::message("autostart_failed", error.to_string()))?;
        settings.autostart.enabled = value;
    }
    if let Some(value) = patch.restore_last_wallpaper {
        settings.autostart.restore_last_wallpaper = value;
    }
    if let Some(value) = patch.library_folders {
        settings.library.folders = value.into_iter().map(PathBuf::from).collect();
    }
    Ok(())
}

fn wallpaper_to_dto(item: &WallpaperItem) -> WallpaperDto {
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
        source_type: source_type_name(item.source_type).to_string(),
        wallpaper_type: wallpaper_type_name(item.wallpaper_type).to_string(),
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
        last_used: item.last_used.map(|date| date.to_rfc3339()),
    }
}

fn source_type_name(value: SourceType) -> &'static str {
    match value {
        SourceType::LocalFile => "local_file",
        SourceType::LocalDirectory => "local_directory",
        SourceType::SteamWorkshop => "steam_workshop",
    }
}

fn wallpaper_type_name(value: WallpaperType) -> &'static str {
    match value {
        WallpaperType::Video => "video",
        WallpaperType::Image => "image",
        WallpaperType::Gif => "gif",
        WallpaperType::Scene => "scene",
    }
}

fn merge_wallpapers(target: &mut Vec<WallpaperItem>, items: Vec<WallpaperItem>) {
    for item in items {
        if let Some(existing) = target.iter_mut().find(|existing| existing.id == item.id) {
            *existing = item;
        } else {
            target.push(item);
        }
    }
}

fn save_wallpaper_state(
    settings: &mut AppSettings,
    monitors: &[MonitorDto],
    output: Option<&str>,
    path: &Path,
) {
    let names = output
        .map(|name| vec![name.to_string()])
        .unwrap_or_else(|| {
            monitors
                .iter()
                .map(|monitor| monitor.name.clone())
                .collect()
        });
    for monitor in names {
        let state = crate::settings::MonitorState {
            monitor: monitor.clone(),
            wallpaper_path: Some(path.to_path_buf()),
        };
        if let Some(existing) = settings
            .autostart
            .monitor_states
            .iter_mut()
            .find(|item| item.monitor == monitor)
        {
            *existing = state;
        } else {
            settings.autostart.monitor_states.push(state);
        }
    }
}

fn thumbnail_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("wayvid")
        .join("thumbnails")
}

fn thumbnail_cache_path(cache_dir: &Path, id: &str) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    cache_dir.join(format!("{:x}.webp", hasher.finish()))
}

fn detect_monitors() -> Vec<MonitorDto> {
    let output = std::process::Command::new("wlr-randr").output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    let mut current: Option<MonitorDto> = None;
    for line in stdout.lines() {
        if !line.starts_with(' ') && !line.is_empty() {
            if let Some(monitor) = current.take() {
                monitors.push(monitor);
            }
            let name = line
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string();
            if !name.is_empty() {
                current = Some(MonitorDto {
                    name,
                    width: 0,
                    height: 0,
                    x: 0,
                    y: 0,
                    scale: 1.0,
                    primary: monitors.is_empty(),
                    current_wallpaper: None,
                });
            }
        } else if let Some(monitor) = current.as_mut() {
            let line = line.trim();
            if line.contains("current") && line.contains(" px") {
                if let Some(resolution) = line.split(" px").next() {
                    let parts: Vec<_> = resolution.split('x').collect();
                    if parts.len() == 2 {
                        monitor.width = parts[0].trim().parse().unwrap_or(0);
                        monitor.height = parts[1].trim().parse().unwrap_or(0);
                    }
                }
            } else if let Some(position) = line.strip_prefix("Position:") {
                let parts: Vec<_> = position.trim().split(',').collect();
                if parts.len() == 2 {
                    monitor.x = parts[0].trim().parse().unwrap_or(0);
                    monitor.y = parts[1].trim().parse().unwrap_or(0);
                }
            } else if let Some(scale) = line.strip_prefix("Scale:") {
                monitor.scale = scale.trim().parse().unwrap_or(1.0);
            }
        }
    }
    if let Some(monitor) = current {
        monitors.push(monitor);
    }
    monitors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_cache_path_is_stable() {
        let first = thumbnail_cache_path(Path::new("/tmp/cache"), "wallpaper");
        let second = thumbnail_cache_path(Path::new("/tmp/cache"), "wallpaper");
        assert_eq!(first, second);
        assert!(first
            .extension()
            .is_some_and(|extension| extension == "webp"));
    }

    #[test]
    fn settings_patch_clamps_values() {
        let mut settings = AppSettings::default();
        apply_settings_patch(
            &mut settings,
            SettingsPatch {
                volume: Some(2.0),
                window_width: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(settings.playback.volume, 1.0);
        assert_eq!(settings.gui.window_width, 800);
    }
}
