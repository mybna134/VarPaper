//! Folder scanner for discovering wallpapers
//!
//! Features:
//! - Parallel scanning using rayon
//! - Incremental scanning with change detection
//! - File system watching with debounced events
//! - Automatic file type detection

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind, Debouncer};
use parking_lot::RwLock;
use rayon::prelude::*;
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::{SourceType, SteamLibrary, WallpaperItem, WallpaperType};

/// File scanner for discovering wallpapers
#[derive(Debug, Clone)]
pub struct FolderScanner {
    /// Supported video extensions
    video_extensions: HashSet<String>,
    /// Supported image extensions
    image_extensions: HashSet<String>,
}

impl FolderScanner {
    pub fn new() -> Self {
        Self {
            video_extensions: ["mp4", "mkv", "webm", "avi", "mov", "m4v", "wmv", "flv"]
                .iter()
                .map(|s| s.to_string())
                .collect(),

            image_extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tiff", "tif", "gif"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// Scan a folder for wallpapers (single-threaded)
    pub fn scan_folder(&self, path: &Path, recursive: bool) -> Result<Vec<WallpaperItem>> {
        info!("🔍 Scanning folder: {}", path.display());

        if !path.exists() {
            warn!("  ⚠️ Folder does not exist: {}", path.display());
            return Ok(Vec::new());
        }

        let items = self
            .wallpaper_paths(path, recursive)
            .iter()
            .filter_map(|path| self.process_file(path))
            .collect::<Vec<_>>();

        info!("  ✓ Found {} wallpapers", items.len());
        Ok(items)
    }

    /// Scan a folder in parallel using rayon
    pub fn scan_folder_parallel(&self, path: &Path, recursive: bool) -> Result<Vec<WallpaperItem>> {
        info!("🔍 Parallel scanning folder: {}", path.display());

        if !path.exists() {
            warn!("  ⚠️ Folder does not exist: {}", path.display());
            return Ok(Vec::new());
        }

        let start = Instant::now();

        let items = self
            .wallpaper_paths(path, recursive)
            .par_iter()
            .filter_map(|path| self.process_file(path))
            .collect::<Vec<_>>();

        let duration = start.elapsed();
        info!(
            "  ✓ Found {} wallpapers in {:?} (parallel)",
            items.len(),
            duration
        );

        Ok(items)
    }

    /// Scan multiple folders in parallel
    pub fn scan_folders_parallel(
        &self,
        folders: &[(PathBuf, bool)],
    ) -> Result<HashMap<PathBuf, Vec<WallpaperItem>>> {
        info!("🔍 Scanning {} folders in parallel", folders.len());
        let start = Instant::now();

        let results: HashMap<PathBuf, Vec<WallpaperItem>> = folders
            .par_iter()
            .map(|(path, recursive)| {
                let items = self
                    .scan_folder_parallel(path, *recursive)
                    .unwrap_or_default();
                (path.clone(), items)
            })
            .collect();

        let total_items: usize = results.values().map(|v| v.len()).sum();
        let duration = start.elapsed();
        info!(
            "  ✓ Found {} total wallpapers across {} folders in {:?}",
            total_items,
            folders.len(),
            duration
        );

        Ok(results)
    }

    /// Process a single file and create WallpaperItem if valid
    pub fn process_file(&self, path: &Path) -> Option<WallpaperItem> {
        if path.file_name().is_some_and(|name| name == "project.json") {
            let root = path.parent()?;
            let shared = SteamLibrary::try_discover()
                .map(|steam| steam.shared_asset_roots())
                .unwrap_or_default();
            return crate::project::discover_project(
                root,
                SourceType::LocalDirectory,
                None,
                shared,
            )
            .ok();
        }
        let extension = path.extension()?.to_string_lossy().to_lowercase();

        let wallpaper_type = if self.video_extensions.contains(&extension) {
            WallpaperType::Video
        } else if extension == "gif" {
            WallpaperType::Gif
        } else if self.image_extensions.contains(&extension) {
            WallpaperType::Image
        } else {
            return None;
        };

        let name = path.file_stem()?.to_string_lossy().to_string();

        // Get file metadata
        let file_size = fs::metadata(path).ok().map(|m| m.len());

        let mut item =
            WallpaperItem::new(path.to_owned(), name, SourceType::LocalFile, wallpaper_type);

        // Update metadata with file size
        item.metadata.file_size = file_size;

        Some(item)
    }

    /// Check if a file is a supported wallpaper
    pub fn is_wallpaper_file(&self, path: &Path) -> bool {
        if path.file_name().is_some_and(|name| name == "project.json") {
            return true;
        }
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            self.video_extensions.contains(&ext) || self.image_extensions.contains(&ext)
        } else {
            false
        }
    }

    fn wallpaper_paths(&self, path: &Path, recursive: bool) -> Vec<PathBuf> {
        let walker = if recursive {
            WalkDir::new(path)
        } else {
            WalkDir::new(path).max_depth(1)
        };
        let mut walker = walker.into_iter();
        let mut paths = Vec::new();
        let mut seen = HashSet::new();
        while let Some(entry) = walker.next() {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if entry.file_type().is_dir() && path.join("project.json").exists() {
                // Normalize project identities even for overlapping folder aliases.
                // Keep a symlinked manifest in place so loading can reject escape.
                let identity = path
                    .canonicalize()
                    .unwrap_or_else(|_| path.into())
                    .join("project.json");
                if seen.insert(identity.clone()) {
                    paths.push(identity);
                }
                walker.skip_current_dir();
            } else if entry.file_type().is_file()
                && self.is_wallpaper_file(path)
                && seen.insert(path.to_path_buf())
            {
                paths.push(path.to_path_buf());
            }
        }
        paths
    }

    /// Get the wallpaper type for a file
    pub fn get_wallpaper_type(&self, path: &Path) -> Option<WallpaperType> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();

        if self.video_extensions.contains(&ext) {
            Some(WallpaperType::Video)
        } else if ext == "gif" {
            Some(WallpaperType::Gif)
        } else if self.image_extensions.contains(&ext) {
            Some(WallpaperType::Image)
        } else {
            None
        }
    }

    /// Add custom video extension (a leading `.` is ignored)
    pub fn add_video_extension(&mut self, ext: &str) {
        self.video_extensions
            .insert(ext.trim_start_matches('.').to_lowercase());
    }

    /// Add custom image extension (a leading `.` is ignored)
    pub fn add_image_extension(&mut self, ext: &str) {
        self.image_extensions
            .insert(ext.trim_start_matches('.').to_lowercase());
    }
}

impl Default for FolderScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a scan operation
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// New wallpapers found
    pub added: Vec<WallpaperItem>,
    /// Wallpapers that no longer exist (IDs)
    pub removed: Vec<String>,
    /// Wallpapers that were updated
    pub updated: Vec<WallpaperItem>,
    /// Total files scanned
    pub files_scanned: usize,
    /// Scan duration in milliseconds
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.updated.is_empty()
    }

    pub fn merge(&mut self, other: ScanResult) {
        self.added.extend(other.added);
        self.removed.extend(other.removed);
        self.updated.extend(other.updated);
        self.files_scanned += other.files_scanned;
        self.duration_ms += other.duration_ms;
    }
}

/// Incremental scanner that tracks changes
pub struct IncrementalScanner {
    scanner: FolderScanner,
    /// Known files and their modification times
    known_files: HashMap<PathBuf, SystemTime>,
}

impl IncrementalScanner {
    pub fn new() -> Self {
        Self {
            scanner: FolderScanner::new(),
            known_files: HashMap::new(),
        }
    }

    /// Perform incremental scan, only processing changed files
    pub fn scan_incremental(&mut self, path: &Path, recursive: bool) -> Result<ScanResult> {
        let start = Instant::now();
        let mut result = ScanResult::default();

        if !path.exists() {
            return Ok(result);
        }

        let mut current_files: HashSet<PathBuf> = HashSet::new();
        let paths = self.scanner.wallpaper_paths(path, recursive);
        for file_path in &paths {
            result.files_scanned += 1;
            current_files.insert(file_path.to_owned());

            let modified = fs::metadata(file_path).ok().and_then(|m| m.modified().ok());

            if let Some(modified) = modified {
                if let Some(known_modified) = self.known_files.get(file_path) {
                    // Check if file was modified
                    if &modified != known_modified {
                        if let Some(item) = self.scanner.process_file(file_path) {
                            result.updated.push(item);
                        }
                    }
                } else {
                    // New file
                    if let Some(item) = self.scanner.process_file(file_path) {
                        result.added.push(item);
                    }
                }
                self.known_files.insert(file_path.to_owned(), modified);
            }
        }

        result.removed = self.forget_removed(path, &current_files);

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Perform parallel incremental scan
    pub fn scan_incremental_parallel(
        &mut self,
        path: &Path,
        recursive: bool,
    ) -> Result<ScanResult> {
        let start = Instant::now();
        let mut result = ScanResult::default();

        if !path.exists() {
            return Ok(result);
        }

        let file_paths = self.scanner.wallpaper_paths(path, recursive);

        result.files_scanned = file_paths.len();
        let current_files: HashSet<PathBuf> = file_paths.iter().cloned().collect();

        // Process files in parallel, checking modification times
        let known_files_ref = &self.known_files;
        let scanner_ref = &self.scanner;

        let changes: Vec<(PathBuf, Option<WallpaperItem>, bool)> = file_paths
            .par_iter()
            .filter_map(|file_path| {
                let modified = fs::metadata(file_path)
                    .ok()
                    .and_then(|m| m.modified().ok())?;

                if let Some(known_modified) = known_files_ref.get(file_path) {
                    if &modified != known_modified {
                        // Updated
                        let item = scanner_ref.process_file(file_path);
                        Some((file_path.clone(), item, false))
                    } else {
                        None
                    }
                } else {
                    // New
                    let item = scanner_ref.process_file(file_path);
                    Some((file_path.clone(), item, true))
                }
            })
            .collect();

        // Update known_files and collect results
        for (file_path, item_opt, is_new) in changes {
            if let Some(item) = item_opt {
                if is_new {
                    result.added.push(item);
                } else {
                    result.updated.push(item);
                }
            }
            if let Ok(modified) = fs::metadata(&file_path).and_then(|m| m.modified()) {
                self.known_files.insert(file_path, modified);
            }
        }

        result.removed = self.forget_removed(path, &current_files);

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Drop known files under `path` that were not seen in this scan and
    /// return their wallpaper IDs.
    fn forget_removed(&mut self, path: &Path, current_files: &HashSet<PathBuf>) -> Vec<String> {
        let mut removed = Vec::new();
        self.known_files.retain(|known, _| {
            let keep = !known.starts_with(path) || current_files.contains(known);
            if !keep {
                removed.push(WallpaperItem::generate_id(known));
            }
            keep
        });
        removed
    }

    /// Clear known files cache
    pub fn clear_cache(&mut self) {
        self.known_files.clear();
    }

    /// Get number of known files
    pub fn known_file_count(&self) -> usize {
        self.known_files.len()
    }
}

impl Default for IncrementalScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// File system event type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEvent {
    /// New file created
    Created(PathBuf),
    /// File modified
    Modified(PathBuf),
    /// File deleted
    Deleted(PathBuf),
}

/// File system watcher for real-time change detection
pub struct FileWatcher {
    scanner: FolderScanner,
    #[allow(dead_code)]
    debouncer: Debouncer<notify::RecommendedWatcher>,
    event_rx: Receiver<FileEvent>,
}

impl FileWatcher {
    /// Create a new file watcher for the given paths
    pub fn new(paths: Vec<(PathBuf, bool)>, debounce_duration: Duration) -> Result<Self> {
        let scanner = FolderScanner::new();
        let (event_tx, event_rx) = channel();

        // Create scanner clone for the debouncer callback
        let scanner_clone = scanner.clone();

        // Create debounced watcher
        let debouncer = new_debouncer(debounce_duration, move |res: DebounceEventResult| {
            if let Ok(events) = res {
                for event in events {
                    let path = event.path;

                    // Only process wallpaper files
                    if path.is_file() && scanner_clone.is_wallpaper_file(&path) {
                        let file_event = match event.kind {
                            DebouncedEventKind::Any => {
                                if path.exists() {
                                    FileEvent::Modified(path)
                                } else {
                                    FileEvent::Deleted(path)
                                }
                            }
                            DebouncedEventKind::AnyContinuous => FileEvent::Modified(path),
                            _ => FileEvent::Modified(path), // Handle future variants
                        };
                        let _ = event_tx.send(file_event);
                    } else if !path.exists() && scanner_clone.is_wallpaper_file(&path) {
                        let _ = event_tx.send(FileEvent::Deleted(path));
                    }
                }
            }
        })
        .context("Failed to create file watcher")?;

        let mut watcher = Self {
            scanner,
            debouncer,
            event_rx,
        };

        // Watch all paths
        for (path, recursive) in &paths {
            let mode = if *recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            watcher
                .debouncer
                .watcher()
                .watch(path, mode)
                .with_context(|| format!("Failed to watch path: {}", path.display()))?;
        }

        info!("👁️ Watching {} paths for changes", paths.len());
        Ok(watcher)
    }

    /// Try to receive a file event (non-blocking)
    pub fn try_recv(&self) -> Option<FileEvent> {
        self.event_rx.try_recv().ok()
    }

    /// Receive file event with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Option<FileEvent> {
        self.event_rx.recv_timeout(timeout).ok()
    }

    /// Process event and return WallpaperItem for created/modified files
    pub fn process_event(&self, event: &FileEvent) -> Option<WallpaperItem> {
        match event {
            FileEvent::Created(path) | FileEvent::Modified(path) => self.scanner.process_file(path),
            FileEvent::Deleted(_) => None,
        }
    }
}

/// Async file watcher using tokio channels
pub struct AsyncFileWatcher {
    scanner: Arc<FolderScanner>,
    watched_paths: Arc<RwLock<Vec<(PathBuf, bool)>>>,
    event_tx: tokio::sync::mpsc::Sender<FileEvent>,
    event_rx: tokio::sync::mpsc::Receiver<FileEvent>,
}

impl AsyncFileWatcher {
    /// Create a new async file watcher
    pub fn new(buffer_size: usize) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(buffer_size);

        Self {
            scanner: Arc::new(FolderScanner::new()),
            watched_paths: Arc::new(RwLock::new(Vec::new())),
            event_tx,
            event_rx,
        }
    }

    /// Add path to watch
    pub fn add_path(&self, path: PathBuf, recursive: bool) {
        self.watched_paths.write().push((path, recursive));
    }

    /// Remove path from watch
    pub fn remove_path(&self, path: &Path) {
        self.watched_paths.write().retain(|(p, _)| p != path);
    }

    /// Receive next event (async)
    pub async fn recv(&mut self) -> Option<FileEvent> {
        self.event_rx.recv().await
    }

    /// Get event sender for external use
    pub fn event_sender(&self) -> tokio::sync::mpsc::Sender<FileEvent> {
        self.event_tx.clone()
    }

    /// Get scanner reference
    pub fn scanner(&self) -> &FolderScanner {
        &self.scanner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn project_scans_prune_assets_and_preserve_types_and_identity() {
        let dir = TempDir::new().unwrap();
        for (name, kind, entry) in [
            ("scene", "SCENE", "scene.json"),
            ("web", "Web", "index.html"),
            ("app", "application", "program.exe"),
            ("unknown", "future", "main.bin"),
        ] {
            let root = dir.path().join(name);
            fs::create_dir_all(root.join("assets")).unwrap();
            fs::write(
                root.join("project.json"),
                serde_json::json!({"type":kind,"file":entry}).to_string(),
            )
            .unwrap();
            fs::write(root.join(entry), "{}").unwrap();
            fs::write(root.join("assets/hidden.mp4"), "asset").unwrap();
            fs::write(root.join("preview.jpg"), "preview").unwrap();
        }
        fs::create_dir(dir.path().join("broken")).unwrap();
        fs::write(dir.path().join("broken/project.json"), "{broken").unwrap();
        fs::write(dir.path().join("broken/hidden.mp4"), "asset").unwrap();
        fs::write(dir.path().join("still.png"), "image").unwrap();
        fs::write(dir.path().join("animation.gif"), "gif").unwrap();
        let scanner = FolderScanner::new();
        let mut single = scanner.scan_folder(dir.path(), true).unwrap();
        let mut parallel = scanner.scan_folder_parallel(dir.path(), true).unwrap();
        single.sort_by(|a, b| a.id.cmp(&b.id));
        parallel.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(single.len(), 7);
        assert_eq!(
            single.iter().map(|item| &item.id).collect::<Vec<_>>(),
            parallel.iter().map(|item| &item.id).collect::<Vec<_>>()
        );
        assert_eq!(
            single
                .iter()
                .filter(|item| item.wallpaper_type == WallpaperType::Unsupported)
                .count(),
            3
        );
        let web = single
            .iter()
            .find(|item| item.wallpaper_type == WallpaperType::Web)
            .unwrap();
        let by_root = scanner.scan_folder(&dir.path().join("web"), false).unwrap();
        assert_eq!(by_root.len(), 1);
        assert_eq!(by_root[0].id, web.id);
        let workshop = crate::project::discover_project(
            &dir.path().join("web"),
            SourceType::SteamWorkshop,
            Some(42),
            vec![],
        )
        .unwrap();
        assert_eq!(workshop.id, web.id);
        assert_eq!(
            single
                .iter()
                .find(|item| item.wallpaper_type == WallpaperType::Image)
                .unwrap()
                .id,
            WallpaperItem::generate_id(&dir.path().join("still.png"))
        );
        let mut incremental = IncrementalScanner::new();
        assert_eq!(
            incremental
                .scan_incremental(dir.path(), true)
                .unwrap()
                .added
                .len(),
            7
        );
        assert!(incremental
            .scan_incremental_parallel(dir.path(), true)
            .unwrap()
            .added
            .is_empty());
    }

    fn create_test_files(dir: &Path) {
        File::create(dir.join("video1.mp4")).unwrap();
        File::create(dir.join("video2.mkv")).unwrap();
        File::create(dir.join("image1.png")).unwrap();
        File::create(dir.join("document.pdf")).unwrap(); // Should be ignored
    }

    #[test]
    fn test_scan_folder() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(temp_dir.path());

        let scanner = FolderScanner::new();
        let items = scanner.scan_folder(temp_dir.path(), false).unwrap();

        assert_eq!(items.len(), 3); // 2 videos + 1 image

        let video_count = items
            .iter()
            .filter(|i| i.wallpaper_type == WallpaperType::Video)
            .count();
        let image_count = items
            .iter()
            .filter(|i| i.wallpaper_type == WallpaperType::Image)
            .count();

        assert_eq!(video_count, 2);
        assert_eq!(image_count, 1);
    }

    #[test]
    fn test_scan_folder_parallel() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(temp_dir.path());

        let scanner = FolderScanner::new();
        let items = scanner
            .scan_folder_parallel(temp_dir.path(), false)
            .unwrap();

        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_is_wallpaper_file() {
        let scanner = FolderScanner::new();

        assert!(scanner.is_wallpaper_file(Path::new("test.mp4")));
        assert!(scanner.is_wallpaper_file(Path::new("test.PNG"))); // Case insensitive
        assert!(scanner.is_wallpaper_file(Path::new("test.webm")));
        assert!(!scanner.is_wallpaper_file(Path::new("test.txt")));
        assert!(!scanner.is_wallpaper_file(Path::new("test.pdf")));
    }

    #[test]
    fn test_get_wallpaper_type() {
        let scanner = FolderScanner::new();

        assert_eq!(
            scanner.get_wallpaper_type(Path::new("test.mp4")),
            Some(WallpaperType::Video)
        );
        assert_eq!(
            scanner.get_wallpaper_type(Path::new("test.png")),
            Some(WallpaperType::Image)
        );
        assert_eq!(
            scanner.get_wallpaper_type(Path::new("test.gif")),
            Some(WallpaperType::Gif)
        );
        assert_eq!(scanner.get_wallpaper_type(Path::new("test.txt")), None);
    }

    #[test]
    fn test_incremental_scanner() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(temp_dir.path());

        let mut scanner = IncrementalScanner::new();

        // First scan should find all files as new
        let result1 = scanner.scan_incremental(temp_dir.path(), false).unwrap();
        assert_eq!(result1.added.len(), 3);
        assert!(result1.removed.is_empty());
        assert!(result1.updated.is_empty());

        // Second scan should find no changes
        let result2 = scanner.scan_incremental(temp_dir.path(), false).unwrap();
        assert!(result2.added.is_empty());
        assert!(result2.removed.is_empty());
        assert!(result2.updated.is_empty());

        // Delete a file
        std::fs::remove_file(temp_dir.path().join("video1.mp4")).unwrap();

        // Third scan should detect deletion
        let result3 = scanner.scan_incremental(temp_dir.path(), false).unwrap();
        assert!(result3.added.is_empty());
        assert_eq!(result3.removed.len(), 1);
        assert!(result3.updated.is_empty());
    }

    #[test]
    fn test_scan_result_merge() {
        let mut result1 = ScanResult {
            added: vec![WallpaperItem::new(
                PathBuf::from("/a.mp4"),
                "a".to_string(),
                SourceType::LocalFile,
                WallpaperType::Video,
            )],
            removed: vec!["id1".to_string()],
            updated: vec![],
            files_scanned: 10,
            duration_ms: 100,
        };

        let result2 = ScanResult {
            added: vec![WallpaperItem::new(
                PathBuf::from("/b.mp4"),
                "b".to_string(),
                SourceType::LocalFile,
                WallpaperType::Video,
            )],
            removed: vec!["id2".to_string()],
            updated: vec![],
            files_scanned: 20,
            duration_ms: 200,
        };

        result1.merge(result2);

        assert_eq!(result1.added.len(), 2);
        assert_eq!(result1.removed.len(), 2);
        assert_eq!(result1.files_scanned, 30);
        assert_eq!(result1.duration_ms, 300);
    }

    #[test]
    fn test_incremental_parallel_scanner_tracks_changes() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(temp_dir.path());
        let mut scanner = IncrementalScanner::default();

        let first = scanner
            .scan_incremental_parallel(temp_dir.path(), false)
            .unwrap();
        assert_eq!(first.added.len(), 3);
        assert_eq!(first.files_scanned, 3);
        assert_eq!(scanner.known_file_count(), 3);

        let unchanged = scanner
            .scan_incremental_parallel(temp_dir.path(), false)
            .unwrap();
        assert!(!unchanged.has_changes());

        // Modify one file (with an mtime clearly in the past) and remove another.
        let video = temp_dir.path().join("video1.mp4");
        File::options()
            .write(true)
            .open(&video)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH)
            .unwrap();
        fs::remove_file(temp_dir.path().join("image1.png")).unwrap();

        let changed = scanner
            .scan_incremental_parallel(temp_dir.path(), false)
            .unwrap();
        assert_eq!(changed.updated.len(), 1);
        assert_eq!(changed.updated[0].source_path, video);
        assert_eq!(
            changed.removed,
            vec![WallpaperItem::generate_id(
                &temp_dir.path().join("image1.png")
            )]
        );
        assert_eq!(scanner.known_file_count(), 2);

        scanner.clear_cache();
        assert_eq!(scanner.known_file_count(), 0);
        assert!(!scanner
            .scan_incremental_parallel(&temp_dir.path().join("missing"), true)
            .unwrap()
            .has_changes());
    }

    #[test]
    fn test_incremental_scanner_detects_update_and_removal() {
        let temp_dir = TempDir::new().unwrap();
        create_test_files(temp_dir.path());
        let mut scanner = IncrementalScanner::new();
        scanner.scan_incremental(temp_dir.path(), true).unwrap();

        File::options()
            .write(true)
            .open(temp_dir.path().join("video2.mkv"))
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH)
            .unwrap();
        fs::remove_file(temp_dir.path().join("video1.mp4")).unwrap();

        let result = scanner.scan_incremental(temp_dir.path(), true).unwrap();
        assert_eq!(result.updated.len(), 1);
        assert_eq!(result.removed.len(), 1);
        assert!(result.added.is_empty());
    }

    #[test]
    fn test_scan_folders_parallel_and_custom_extensions() {
        let first = TempDir::new().unwrap();
        let second = TempDir::new().unwrap();
        create_test_files(first.path());
        File::create(second.path().join("clip.xyz")).unwrap();
        File::create(second.path().join("pic.abc")).unwrap();

        let mut scanner = FolderScanner::new();
        scanner.add_video_extension(".XYZ");
        scanner.add_image_extension("abc");
        assert_eq!(
            scanner.get_wallpaper_type(&second.path().join("clip.xyz")),
            Some(WallpaperType::Video)
        );

        let results = scanner
            .scan_folders_parallel(&[
                (first.path().to_path_buf(), false),
                (second.path().to_path_buf(), true),
                (PathBuf::from("/definitely/missing"), false),
            ])
            .unwrap();
        assert_eq!(results[first.path()].len(), 3);
        assert_eq!(results[second.path()].len(), 2);
        assert!(results[Path::new("/definitely/missing")].is_empty());
    }
}
