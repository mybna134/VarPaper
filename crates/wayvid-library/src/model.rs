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
    Web,
    Gif,
    Image,
    Unsupported,
}

impl WallpaperType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Scene => "scene",
            Self::Web => "web",
            Self::Gif => "gif",
            Self::Image => "image",
            Self::Unsupported => "unsupported",
        }
    }

    /// Parse a project declaration without treating unknown content as video.
    pub fn from_project_type(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "video" => Self::Video,
            "scene" => Self::Scene,
            "web" => Self::Web,
            "image" => Self::Image,
            "gif" => Self::Gif,
            _ => Self::Unsupported,
        }
    }
}

/// Loading status, separate from successful renderer initialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    #[default]
    Ready,
    RequiresRenderer,
    Unsupported,
    Invalid,
}

impl CompatibilityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::RequiresRenderer => "requires_renderer",
            Self::Unsupported => "unsupported",
            Self::Invalid => "invalid",
        }
    }
}

/// Project context needed to resolve assets and restore an assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSource {
    pub version: u32,
    pub manifest: PathBuf,
    pub root: PathBuf,
    pub declared_type: String,
    pub entry: Option<String>,
    #[serde(default)]
    pub shared_asset_roots: Vec<PathBuf>,
    #[serde(default)]
    pub properties: serde_json::Value,
    #[serde(default)]
    pub property_overrides: serde_json::Map<String, serde_json::Value>,
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
    #[serde(default)]
    pub project: Option<ProjectSource>,
    #[serde(default)]
    pub compatibility: CompatibilityStatus,
    #[serde(default)]
    pub compatibility_reason: Option<String>,
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
            project: None,
            compatibility: CompatibilityStatus::Ready,
            compatibility_reason: None,
            thumbnail_path: None,
            metadata: WallpaperMetadata::default(),
            added_at: Utc::now(),
            last_used: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_items_remain_readable_and_media_ids_remain_stable() {
        let item = WallpaperItem::new(
            "/media/a.gif".into(),
            "a".into(),
            SourceType::LocalFile,
            WallpaperType::Gif,
        );
        let mut value = serde_json::to_value(&item).unwrap();
        for field in ["project", "compatibility", "compatibility_reason"] {
            value.as_object_mut().unwrap().remove(field);
        }
        let restored: WallpaperItem = serde_json::from_value(value).unwrap();
        assert_eq!(
            restored.id,
            WallpaperItem::generate_id(Path::new("/media/a.gif"))
        );
        assert_eq!(restored.compatibility, CompatibilityStatus::Ready);
        assert!(restored.project.is_none());
    }

    #[test]
    fn project_types_never_fall_back_to_video() {
        for (declaration, expected) in [
            ("Video", WallpaperType::Video),
            ("SCENE", WallpaperType::Scene),
            ("Web", WallpaperType::Web),
            ("image", WallpaperType::Image),
            ("gif", WallpaperType::Gif),
            ("application", WallpaperType::Unsupported),
            ("future", WallpaperType::Unsupported),
        ] {
            assert_eq!(WallpaperType::from_project_type(declaration), expected);
            let json = serde_json::to_string(&expected).unwrap();
            assert_eq!(
                serde_json::from_str::<WallpaperType>(&json).unwrap(),
                expected
            );
        }
    }
}
