use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    LocalFile,
    LocalDirectory,
    SteamWorkshop,
}

impl SourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalFile => "local_file",
            Self::LocalDirectory => "local_dir",
            Self::SteamWorkshop => "workshop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperType {
    Video,
    Scene,
    Gif,
    Image,
}

impl WallpaperType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Scene => "scene",
            Self::Gif => "gif",
            Self::Image => "image",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WallpaperMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub duration_secs: Option<f64>,
    pub resolution: Option<(u32, u32)>,
    pub file_size: Option<u64>,
    pub workshop_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperItem {
    pub id: String,
    pub name: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub wallpaper_type: WallpaperType,
    pub thumbnail_path: Option<PathBuf>,
    pub metadata: WallpaperMetadata,
    pub added_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl WallpaperItem {
    pub fn generate_id(path: &Path) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    pub fn new(
        source_path: PathBuf,
        name: String,
        source_type: SourceType,
        wallpaper_type: WallpaperType,
    ) -> Self {
        Self {
            id: Self::generate_id(&source_path),
            name,
            source_path,
            source_type,
            wallpaper_type,
            thumbnail_path: None,
            metadata: WallpaperMetadata::default(),
            added_at: Utc::now(),
            last_used: None,
        }
    }
}
