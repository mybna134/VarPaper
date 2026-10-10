//! Steam Workshop integration for Wallpaper Engine
//!
//! Features:
//! - Automatic Steam library detection
//! - Workshop item scanning
//! - project.json parsing
//! - Wallpaper metadata extraction

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{SourceType, WallpaperItem, WallpaperType};

/// Wallpaper Engine app ID on Steam
pub const WALLPAPER_ENGINE_APP_ID: u32 = 431960;

/// Steam library discovery and management
#[derive(Debug, Clone)]
pub struct SteamLibrary {
    /// Root Steam installation path
    pub root: PathBuf,
    /// Additional library folders
    pub libraries: Vec<PathBuf>,
}

impl SteamLibrary {
    /// Discover Steam installation automatically
    pub fn discover() -> Result<Self> {
        let root = Self::find_steam_root()?;
        let libraries = Self::parse_library_folders(&root)?;

        info!(
            "🎮 Found Steam installation with {} library folders",
            libraries.len()
        );
        Ok(Self { root, libraries })
    }

    /// Try to discover Steam, return None if not found
    pub fn try_discover() -> Option<Self> {
        Self::discover().ok()
    }

    /// Find Steam root directory
    fn find_steam_root() -> Result<PathBuf> {
        // Common Steam paths on Linux
        let candidates = [
            dirs::home_dir().map(|h| h.join(".steam/steam")),
            dirs::home_dir().map(|h| h.join(".local/share/Steam")),
            Some(PathBuf::from("/usr/share/steam")),
            // Flatpak Steam
            dirs::home_dir().map(|h| h.join(".var/app/com.valvesoftware.Steam/.steam/steam")),
            // Snap Steam
            dirs::home_dir().map(|h| h.join("snap/steam/common/.steam/steam")),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() && candidate.join("steamapps").exists() {
                debug!("Found Steam at: {:?}", candidate);
                return Ok(candidate);
            }
        }

        anyhow::bail!("Steam installation not found")
    }

    /// Parse libraryfolders.vdf to find additional libraries
    fn parse_library_folders(root: &Path) -> Result<Vec<PathBuf>> {
        let vdf_path = root.join("steamapps/libraryfolders.vdf");
        if !vdf_path.exists() {
            return Ok(vec![root.to_path_buf()]);
        }

        let content = fs::read_to_string(&vdf_path).context("Failed to read libraryfolders.vdf")?;

        let mut libraries = vec![root.to_path_buf()];
        libraries.extend(Self::parse_vdf_paths(&content));

        // Deduplicate
        let mut seen = HashSet::new();
        libraries.retain(|p| seen.insert(p.clone()));

        Ok(libraries)
    }

    /// Parse VDF file for library paths
    fn parse_vdf_paths(content: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            // Look for "path" key in VDF format: "path"		"/path/to/library"
            if line.starts_with("\"path\"") {
                if let Some(path_str) = Self::extract_vdf_value(line) {
                    let path = PathBuf::from(path_str);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
        }

        paths
    }

    /// Extract quoted value from VDF line
    fn extract_vdf_value(line: &str) -> Option<String> {
        // Format: "key"		"value"
        let parts: Vec<&str> = line.split('"').collect();
        if parts.len() >= 4 {
            Some(parts[3].to_string())
        } else {
            None
        }
    }

    /// Get all library paths (including root), without duplicates
    pub fn all_libraries(&self) -> Vec<&PathBuf> {
        let mut seen = HashSet::new();
        std::iter::once(&self.root)
            .chain(self.libraries.iter())
            .filter(|path| seen.insert(*path))
            .collect()
    }

    /// Find Workshop content path for an app
    pub fn workshop_content_path(&self, app_id: u32) -> Vec<PathBuf> {
        self.all_libraries()
            .into_iter()
            .map(|lib| {
                lib.join("steamapps/workshop/content")
                    .join(app_id.to_string())
            })
            .filter(|p| p.exists())
            .collect()
    }

    /// Check if Wallpaper Engine is installed
    pub fn has_wallpaper_engine(&self) -> bool {
        !self
            .workshop_content_path(WALLPAPER_ENGINE_APP_ID)
            .is_empty()
    }

    /// Existing shared asset directories from locally installed Wallpaper Engine.
    pub fn shared_asset_roots(&self) -> Vec<PathBuf> {
        let mut roots: Vec<_> = self
            .all_libraries()
            .into_iter()
            .map(|root| root.join("steamapps/common/wallpaper_engine/assets"))
            .filter_map(|path| path.canonicalize().ok())
            .filter(|path| path.is_dir())
            .collect();
        roots.sort();
        roots.dedup();
        roots
    }
}

/// Wallpaper Engine project metadata from project.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeProject {
    /// Project type: "video", "scene", "web", "application"
    #[serde(rename = "type")]
    pub project_type: String,

    /// Video/scene file path (relative to project directory)
    pub file: Option<String>,

    /// Project title
    #[serde(default)]
    pub title: Option<String>,

    /// Project description
    #[serde(default)]
    pub description: Option<String>,

    /// Preview image path
    #[serde(default)]
    pub preview: Option<String>,

    /// Steam Workshop ID
    #[serde(default)]
    pub workshopid: Option<String>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Content rating
    #[serde(default)]
    pub contentrating: Option<String>,
    /// Project properties and audio-processing declarations.
    #[serde(default)]
    pub general: serde_json::Value,
}

impl WeProject {
    /// Parse project.json from path
    pub fn load(project_dir: &Path) -> Result<Self> {
        crate::project::load_manifest(project_dir)
    }

    /// Check if this is a video type project
    pub fn is_video(&self) -> bool {
        self.project_type.to_lowercase() == "video"
    }

    /// Check if this is a scene type project
    pub fn is_scene(&self) -> bool {
        self.project_type.to_lowercase() == "scene"
    }

    /// Check if this project type is supported
    pub fn is_supported(&self) -> bool {
        self.wallpaper_type() != WallpaperType::Unsupported
    }

    /// Get the wallpaper type
    pub fn wallpaper_type(&self) -> WallpaperType {
        WallpaperType::from_project_type(&self.project_type)
    }

    /// Get the main file path
    pub fn main_file(&self, project_dir: &Path) -> Option<PathBuf> {
        self.file
            .as_ref()
            .and_then(|name| crate::project::resource_name(name).ok())
            .map(|name| project_dir.join(name))
    }

    /// Get the preview image path
    pub fn preview_image(&self, project_dir: &Path) -> Option<PathBuf> {
        self.preview.as_ref().and_then(|name| {
            crate::project::loose_resource(project_dir, name)
                .ok()
                .flatten()
        })
    }
}

/// Workshop scanner for discovering Wallpaper Engine wallpapers
#[derive(Debug)]
pub struct WorkshopScanner {
    steam: SteamLibrary,
    /// Cache of scanned workshop IDs
    scanned_ids: HashSet<u64>,
}

impl WorkshopScanner {
    /// Create a new workshop scanner
    pub fn new(steam: SteamLibrary) -> Self {
        Self {
            steam,
            scanned_ids: HashSet::new(),
        }
    }

    /// Create scanner with auto-discovery
    pub fn discover() -> Result<Self> {
        let steam = SteamLibrary::discover()?;
        Ok(Self::new(steam))
    }

    /// Try to create scanner, return None if Steam not found
    pub fn try_discover() -> Option<Self> {
        SteamLibrary::try_discover().map(Self::new)
    }

    /// Scan all Workshop items for Wallpaper Engine
    pub fn scan_all(&mut self) -> Result<Vec<WallpaperItem>> {
        let workshop_paths = self.steam.workshop_content_path(WALLPAPER_ENGINE_APP_ID);

        if workshop_paths.is_empty() {
            warn!("⚠️ No Wallpaper Engine Workshop content found");
            return Ok(Vec::new());
        }

        let mut items = Vec::new();

        for workshop_path in workshop_paths {
            info!("🔍 Scanning Workshop: {}", workshop_path.display());

            let scanned = self.scan_workshop_directory(&workshop_path)?;
            items.extend(scanned);
        }

        info!("✅ Found {} Workshop wallpapers", items.len());
        Ok(items)
    }

    /// Scan a specific Workshop directory
    fn scan_workshop_directory(&mut self, path: &Path) -> Result<Vec<WallpaperItem>> {
        let mut items = Vec::new();

        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let item_path = entry.path();

            if !item_path.is_dir() {
                continue;
            }

            // Extract workshop ID from directory name
            let workshop_id: u64 = match item_path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|s| s.parse().ok())
            {
                Some(id) => id,
                None => continue,
            };

            // Skip already scanned items
            if !self.scanned_ids.insert(workshop_id) {
                continue;
            }

            // Try to parse as WE project
            match self.parse_workshop_item(&item_path, workshop_id) {
                Ok(Some(item)) => {
                    debug!("  📄 {}", item.name);
                    items.push(item);
                }
                Ok(None) => {
                    // Unsupported type, skip
                }
                Err(e) => {
                    debug!("  ⚠️ Failed to parse {}: {}", item_path.display(), e);
                }
            }
        }

        Ok(items)
    }

    /// Parse a Workshop item directory
    fn parse_workshop_item(
        &self,
        item_path: &Path,
        workshop_id: u64,
    ) -> Result<Option<WallpaperItem>> {
        let project_file = item_path.join("project.json");

        if !project_file.exists() {
            return Ok(None);
        }

        crate::project::discover_project(
            item_path,
            SourceType::SteamWorkshop,
            Some(workshop_id),
            self.steam.shared_asset_roots(),
        )
        .map(Some)
    }

    /// Get workshop item by ID
    pub fn get_item(&self, workshop_id: u64) -> Result<Option<WallpaperItem>> {
        let workshop_paths = self.steam.workshop_content_path(WALLPAPER_ENGINE_APP_ID);

        for workshop_path in workshop_paths {
            let item_path = workshop_path.join(workshop_id.to_string());
            if item_path.exists() {
                return match self.parse_workshop_item(&item_path, workshop_id)? {
                    Some(item) => Ok(Some(item)),
                    None => continue,
                };
            }
        }

        Ok(None)
    }

    /// Clear scanned cache
    pub fn clear_cache(&mut self) {
        self.scanned_ids.clear();
    }

    /// Get Steam library reference
    pub fn steam(&self) -> &SteamLibrary {
        &self.steam
    }
}

/// Detect if a path is a Wallpaper Engine project
pub fn is_we_project(path: &Path) -> bool {
    path.join("project.json").exists()
}

/// Get project type from path
pub fn get_project_type(path: &Path) -> Result<String> {
    let project = WeProject::load(path)?;
    Ok(project.project_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_vdf_value_extraction() {
        let line = r#"		"path"		"/home/user/SteamLibrary""#;
        let value = SteamLibrary::extract_vdf_value(line);
        assert_eq!(value, Some("/home/user/SteamLibrary".to_string()));
    }

    #[test]
    fn test_vdf_parsing() {
        let content = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"/home/user/.local/share/Steam"
		"label"		""
	}
	"1"
	{
		"path"		"/mnt/games/SteamLibrary"
		"label"		"Games"
	}
}
"#;
        let paths = SteamLibrary::parse_vdf_paths(content);
        // Note: paths must exist to be returned
        assert!(paths.len() <= 2);
    }

    #[test]
    fn test_we_project_parse() {
        let temp_dir = TempDir::new().unwrap();
        let project_file = temp_dir.path().join("project.json");

        let content = r#"{
            "type": "video",
            "file": "video.mp4",
            "title": "Test Video",
            "description": "A test video wallpaper",
            "preview": "preview.jpg",
            "workshopid": "123456",
            "tags": ["nature", "landscape"]
        }"#;

        let mut file = File::create(&project_file).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let project = WeProject::load(temp_dir.path()).unwrap();

        assert_eq!(project.project_type, "video");
        assert_eq!(project.file.as_deref(), Some("video.mp4"));
        assert_eq!(project.title.as_deref(), Some("Test Video"));
        assert!(project.is_video());
        assert!(project.is_supported());
        assert_eq!(project.tags.len(), 2);
    }

    #[test]
    fn test_we_project_type_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_file = temp_dir.path().join("project.json");

        // Video type
        let content = r#"{"type": "video", "file": "test.mp4"}"#;
        fs::write(&project_file, content).unwrap();
        let project = WeProject::load(temp_dir.path()).unwrap();
        assert!(project.is_video());
        assert!(!project.is_scene());
        assert!(project.is_supported());

        // Scene type
        let content = r#"{"type": "scene", "file": "scene.json"}"#;
        fs::write(&project_file, content).unwrap();
        let project = WeProject::load(temp_dir.path()).unwrap();
        assert!(!project.is_video());
        assert!(project.is_scene());
        assert!(project.is_supported());

        // Web is a recognized project type, separate from renderer availability.
        let content = r#"{"type": "web", "file": "index.html"}"#;
        fs::write(&project_file, content).unwrap();
        let project = WeProject::load(temp_dir.path()).unwrap();
        assert!(!project.is_video());
        assert!(!project.is_scene());
        assert!(project.is_supported());
        assert_eq!(project.wallpaper_type(), WallpaperType::Web);
        fs::write(
            &project_file,
            r#"{"type":"application","file":"program.exe"}"#,
        )
        .unwrap();
        let project = WeProject::load(temp_dir.path()).unwrap();
        assert!(!project.is_supported());
        assert_eq!(project.wallpaper_type(), WallpaperType::Unsupported);
    }

    #[test]
    fn test_is_we_project() {
        let temp_dir = TempDir::new().unwrap();

        // No project.json
        assert!(!is_we_project(temp_dir.path()));

        // With project.json
        File::create(temp_dir.path().join("project.json")).unwrap();
        assert!(is_we_project(temp_dir.path()));
    }

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// Builds `<root>/steamapps/workshop/content/431960/<id>/...` fixtures.
    fn fake_steam() -> (TempDir, SteamLibrary) {
        let root = TempDir::new().unwrap();
        let content = root
            .path()
            .join("steamapps/workshop/content")
            .join(WALLPAPER_ENGINE_APP_ID.to_string());

        write(
            &content.join("100/project.json"),
            r#"{"type":"video","file":"v.mp4","title":"Video","preview":"p.jpg","tags":["a"]}"#,
        );
        write(&content.join("100/v.mp4"), "");
        write(&content.join("100/p.jpg"), "");
        // Scene without a main file falls back to the project directory.
        write(&content.join("200/project.json"), r#"{"type":"scene"}"#);
        // Unsupported, missing main file, broken JSON, no project, non-numeric.
        write(
            &content.join("300/project.json"),
            r#"{"type":"web","file":"index.html"}"#,
        );
        write(
            &content.join("400/project.json"),
            r#"{"type":"video","file":"gone.mp4"}"#,
        );
        write(&content.join("500/project.json"), "{not json");
        fs::create_dir_all(content.join("600")).unwrap();
        write(&content.join("notanid/project.json"), r#"{"type":"scene"}"#);
        write(&content.join("700"), "a file, not a directory");
        // Video without a file entry is unusable.
        write(&content.join("800/project.json"), r#"{"type":"video"}"#);

        let steam = SteamLibrary {
            root: root.path().to_path_buf(),
            libraries: vec![root.path().to_path_buf()],
        };
        (root, steam)
    }

    #[test]
    fn test_library_folders_are_deduplicated() {
        let root = TempDir::new().unwrap();
        let extra = TempDir::new().unwrap();
        let vdf = format!(
            "\"libraryfolders\"\n{{\n\t\"0\" {{ \"path\"\t\"{}\" }}\n\t\"path\"\t\"{}\"\n\t\"path\"\t\"/does/not/exist\"\n}}",
            root.path().display(),
            extra.path().display()
        );
        write(&root.path().join("steamapps/libraryfolders.vdf"), &vdf);

        let libraries = SteamLibrary::parse_library_folders(root.path()).unwrap();
        assert_eq!(
            libraries,
            vec![root.path().to_path_buf(), extra.path().to_path_buf()]
        );

        let steam = SteamLibrary {
            root: root.path().to_path_buf(),
            libraries,
        };
        assert_eq!(steam.all_libraries().len(), 2);
        assert!(!steam.has_wallpaper_engine());
    }

    #[test]
    fn test_library_folders_without_vdf() {
        let root = TempDir::new().unwrap();
        let libraries = SteamLibrary::parse_library_folders(root.path()).unwrap();
        assert_eq!(libraries, vec![root.path().to_path_buf()]);
        assert_eq!(SteamLibrary::extract_vdf_value("\"path\""), None);
    }

    #[test]
    fn test_scan_all_workshop_items() {
        let (_root, steam) = fake_steam();
        assert!(steam.has_wallpaper_engine());
        assert_eq!(
            steam.workshop_content_path(WALLPAPER_ENGINE_APP_ID).len(),
            1
        );

        let mut scanner = WorkshopScanner::new(steam);
        let mut items = scanner.scan_all().unwrap();
        items.sort_by_key(|item| item.metadata.workshop_id);
        let ids: Vec<_> = items
            .iter()
            .map(|item| item.metadata.workshop_id.unwrap())
            .collect();
        assert_eq!(ids, vec![100, 200, 300, 400, 500, 800]);

        let video = &items[0];
        assert_eq!(video.name, "Video");
        assert_eq!(video.wallpaper_type, WallpaperType::Video);
        assert_eq!(video.source_type, SourceType::SteamWorkshop);
        assert!(video.source_path.ends_with("100/v.mp4"));
        assert!(video
            .thumbnail_path
            .as_ref()
            .unwrap()
            .ends_with("100/p.jpg"));
        assert_eq!(video.metadata.tags, vec!["a".to_string()]);

        let scene = &items[1];
        assert_eq!(scene.name, "Workshop #200");
        assert_eq!(scene.wallpaper_type, WallpaperType::Scene);
        assert!(scene.source_path.ends_with("200/project.json"));
        assert_eq!(scene.compatibility, crate::CompatibilityStatus::Invalid);
        assert_eq!(scene.thumbnail_path, None);
        assert_eq!(items[2].wallpaper_type, WallpaperType::Web);
        assert!(items[2]
            .compatibility_reason
            .as_ref()
            .unwrap()
            .contains("missing"));
        assert_eq!(items[4].compatibility, crate::CompatibilityStatus::Invalid);

        // Already scanned items are skipped until the cache is cleared.
        assert!(scanner.scan_all().unwrap().is_empty());
        scanner.clear_cache();
        assert_eq!(scanner.scan_all().unwrap().len(), 6);
    }

    #[test]
    fn test_get_item() {
        let (_root, steam) = fake_steam();
        let scanner = WorkshopScanner::new(steam);
        assert_eq!(scanner.get_item(100).unwrap().unwrap().name, "Video");
        assert_eq!(
            scanner.get_item(300).unwrap().unwrap().wallpaper_type,
            WallpaperType::Web
        );
        assert!(scanner.get_item(999).unwrap().is_none());
        assert_eq!(
            scanner.get_item(500).unwrap().unwrap().compatibility,
            crate::CompatibilityStatus::Invalid
        );
        assert_eq!(scanner.steam().all_libraries().len(), 1);
    }

    #[test]
    fn test_scan_all_without_workshop_content() {
        let root = TempDir::new().unwrap();
        let mut scanner = WorkshopScanner::new(SteamLibrary {
            root: root.path().to_path_buf(),
            libraries: Vec::new(),
        });
        assert!(scanner.scan_all().unwrap().is_empty());
    }

    #[test]
    fn test_get_project_type() {
        let dir = TempDir::new().unwrap();
        assert!(get_project_type(dir.path()).is_err());
        write(&dir.path().join("project.json"), r#"{"type":"Scene"}"#);
        assert_eq!(get_project_type(dir.path()).unwrap(), "Scene");
        let project = WeProject::load(dir.path()).unwrap();
        assert!(project.is_scene());
        assert_eq!(project.wallpaper_type(), WallpaperType::Scene);
        assert_eq!(project.main_file(dir.path()), None);
        assert_eq!(project.preview_image(dir.path()), None);
    }
}
