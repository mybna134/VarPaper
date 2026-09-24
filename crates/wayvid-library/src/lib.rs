//! wayvid-library: Wallpaper library management for wayvid
//!
//! This crate provides stateless wallpaper discovery and preview extraction.
//! The Flutter application owns persistence; this crate only returns in-memory
//! scan results and never talks to the playback engine.
//!
mod model;
pub mod scanner;
pub mod thumbnail;
pub mod workshop;

pub use model::{SourceType, WallpaperItem, WallpaperMetadata, WallpaperType};
pub use scanner::{
    AsyncFileWatcher, FileEvent, FileWatcher, FolderScanner, IncrementalScanner, ScanResult,
};
pub use thumbnail::{
    get_video_dimensions, get_video_duration, CacheStats, ThumbnailFormat, ThumbnailGenerator,
    ThumbnailPriority, ThumbnailRequest, ThumbnailResponse, ThumbnailResult, ThumbnailService,
};

// Workshop exports
pub use workshop::{
    get_project_type, is_we_project, SteamLibrary, WeProject, WorkshopScanner,
    WALLPAPER_ENGINE_APP_ID,
};
