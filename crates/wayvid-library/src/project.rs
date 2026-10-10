//! Bounded Wallpaper Engine project and PKGV resource loading.
//!
//! Loading a project validates its resources, but does not claim that a Scene
//! or Web renderer has initialized. Packages are read in place, never extracted.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};

use crate::workshop::WeProject;
use crate::{CompatibilityStatus, ProjectSource, SourceType, WallpaperItem, WallpaperType};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_RESOURCE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PACKAGE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_ENTRIES: u32 = 100_000;
const MAX_NAME_BYTES: u32 = 4096;

/// Validate a relative resource name and normalize its path components.
/// Backslashes are rejected so a name cannot acquire a different meaning in
/// a native library. Paths are not URL-decoded by this interface.
pub fn resource_name(name: &str) -> Result<PathBuf> {
    ensure!(
        !name.is_empty() && !name.contains(['\\', '\0', ':']),
        "Invalid resource path: {name}"
    );
    let path = Path::new(name);
    ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir)),
        "Resource path escapes its root: {name}"
    );
    let normalized: PathBuf = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value),
            _ => None,
        })
        .collect();
    ensure!(
        !normalized.as_os_str().is_empty(),
        "Empty resource path: {name}"
    );
    Ok(normalized)
}

/// Locate an existing loose file, rejecting symlinks outside its root.
pub fn loose_resource(root: &Path, name: &str) -> Result<Option<PathBuf>> {
    let name = resource_name(name)?;
    let root = root
        .canonicalize()
        .with_context(|| format!("Invalid asset root {}", root.display()))?;
    let candidate = root.join(name);
    let resolved = match candidate.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    ensure!(
        resolved.starts_with(&root),
        "Resource symlink escapes root: {}",
        candidate.display()
    );
    ensure!(
        resolved.is_file(),
        "Resource is not a file: {}",
        resolved.display()
    );
    Ok(Some(resolved))
}

#[derive(Debug, Clone)]
struct PackageEntry {
    offset: u64,
    length: u64,
}

/// Validated PKGV directory. Offsets are relative to the end of the index.
#[derive(Debug)]
pub struct PackageIndex {
    path: PathBuf,
    entries: HashMap<PathBuf, PackageEntry>,
}

impl PackageIndex {
    /// Read an index with explicit bounds for filenames, entries and offsets.
    pub fn open(path: &Path) -> Result<Self> {
        let mut file =
            File::open(path).with_context(|| format!("Cannot open package {}", path.display()))?;
        let size = file.metadata()?.len();
        ensure!(size <= MAX_PACKAGE_BYTES, "Package exceeds size budget");
        let header = sized_string(&mut file)?;
        ensure!(
            header.starts_with("PKGV"),
            "Invalid package header: {header}"
        );
        let count = uint32(&mut file)?;
        ensure!(count <= MAX_ENTRIES, "Package exceeds entry budget");
        let mut entries = HashMap::new();
        for _ in 0..count {
            let name = resource_name(&sized_string(&mut file)?)?;
            let offset = u64::from(uint32(&mut file)?);
            let length = u64::from(uint32(&mut file)?);
            ensure!(
                length <= MAX_RESOURCE_BYTES,
                "Package resource exceeds size budget"
            );
            ensure!(
                entries
                    .insert(name.clone(), PackageEntry { offset, length })
                    .is_none(),
                "Duplicate package resource: {}",
                name.display()
            );
        }
        let base = file.stream_position()?;
        for (name, entry) in &mut entries {
            entry.offset = base
                .checked_add(entry.offset)
                .context("Package offset overflow")?;
            ensure!(
                entry
                    .offset
                    .checked_add(entry.length)
                    .is_some_and(|end| end <= size),
                "Package resource out of bounds: {}",
                name.display()
            );
        }
        Ok(Self {
            path: path.to_path_buf(),
            entries,
        })
    }

    pub fn contains(&self, name: &str) -> Result<bool> {
        Ok(self.entries.contains_key(&resource_name(name)?))
    }

    /// Read one bounded entry, with an independent file cursor for each read.
    pub fn read(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let Some(entry) = self.entries.get(&resource_name(name)?) else {
            return Ok(None);
        };
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(entry.offset))?;
        let mut bytes = vec![0; usize::try_from(entry.length)?];
        file.read_exact(&mut bytes)?;
        Ok(Some(bytes))
    }
}

fn uint32(reader: &mut impl Read) -> Result<u32> {
    let mut bytes = [0; 4];
    reader
        .read_exact(&mut bytes)
        .context("Truncated package index")?;
    Ok(u32::from_le_bytes(bytes))
}

fn sized_string(reader: &mut impl Read) -> Result<String> {
    let length = uint32(reader)?;
    ensure!(
        length <= MAX_NAME_BYTES,
        "Package string exceeds size budget"
    );
    let mut bytes = vec![0; length as usize];
    reader
        .read_exact(&mut bytes)
        .context("Truncated package string")?;
    String::from_utf8(bytes).context("Invalid package string encoding")
}

/// Project-scoped lookup with validated scene.pkg and gifscene.pkg indexes.
pub struct ProjectResources {
    roots: Vec<PathBuf>,
    packages: Vec<PackageIndex>,
}

impl ProjectResources {
    pub fn new(project: &ProjectSource) -> Result<Self> {
        let mut roots = vec![project.root.canonicalize()?];
        for root in &project.shared_asset_roots {
            let root = root.canonicalize()?;
            ensure!(root.is_dir(), "Asset root is not a directory");
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
        let mut packages = Vec::new();
        for name in ["scene.pkg", "gifscene.pkg"] {
            if let Some(path) = loose_resource(&roots[0], name)? {
                packages.push(PackageIndex::open(&path)?);
            }
        }
        Ok(Self { roots, packages })
    }

    pub fn exists(&self, name: &str) -> Result<bool> {
        for root in &self.roots {
            if let Some(path) = loose_resource(root, name)? {
                ensure!(
                    fs::metadata(path)?.len() <= MAX_RESOURCE_BYTES,
                    "Resource exceeds size budget: {name}"
                );
                return Ok(true);
            }
            if root == &self.roots[0] {
                for package in &self.packages {
                    if package.contains(name)? {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>> {
        for (index, root) in self.roots.iter().enumerate() {
            if let Some(path) = loose_resource(root, name)? {
                return read_bounded(&path, MAX_RESOURCE_BYTES);
            }
            if index == 0 {
                for package in &self.packages {
                    if let Some(bytes) = package.read(name)? {
                        return Ok(bytes);
                    }
                }
            }
        }
        bail!("Missing project/shared asset: {name}")
    }
}

pub(crate) fn read_bounded(path: &Path, budget: u64) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    ensure!(
        file.metadata()?.len() <= budget,
        "Resource exceeds size budget: {}",
        path.display()
    );
    let mut bytes = Vec::new();
    (&mut file).take(budget + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= budget,
        "Resource exceeds size budget: {}",
        path.display()
    );
    Ok(bytes)
}

/// Load a bounded, project-scoped manifest.
pub fn load_manifest(root: &Path) -> Result<WeProject> {
    let path = loose_resource(root, "project.json")?.context("Project manifest missing")?;
    serde_json::from_slice(&read_bounded(&path, MAX_MANIFEST_BYTES)?)
        .with_context(|| format!("Invalid project manifest {}", path.display()))
}

/// Discover a project while retaining invalid/unsupported items for diagnosis.
/// Scene/Web remain `requires_renderer` until their renderers are integrated.
pub fn discover_project(
    root: &Path,
    source_type: SourceType,
    workshop_id: Option<u64>,
    shared_asset_roots: Vec<PathBuf>,
) -> Result<WallpaperItem> {
    let root = root.canonicalize()?;
    let manifest = root.join("project.json");
    let mut item = WallpaperItem::new(
        manifest.clone(),
        workshop_id
            .map(|id| format!("Workshop #{id}"))
            .unwrap_or_else(|| {
                root.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            }),
        source_type,
        WallpaperType::Unsupported,
    );
    item.metadata.workshop_id = workshop_id;
    let project = match load_manifest(&root) {
        Ok(project) => project,
        Err(error) => {
            item.compatibility = CompatibilityStatus::Invalid;
            item.compatibility_reason = Some(error.to_string());
            return Ok(item);
        }
    };
    item.wallpaper_type = project.wallpaper_type();
    if let Some(title) = &project.title {
        item.name = title.clone();
    }
    item.metadata.title = project.title.clone();
    item.metadata.description = project.description.clone();
    item.metadata.tags = project.tags.clone();
    item.project = Some(ProjectSource {
        version: 1,
        manifest,
        root: root.clone(),
        declared_type: project.project_type.clone(),
        entry: project.file.clone(),
        shared_asset_roots,
        properties: project
            .general
            .get("properties")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        property_overrides: Default::default(),
    });
    if item.wallpaper_type == WallpaperType::Unsupported {
        item.compatibility = CompatibilityStatus::Unsupported;
        item.compatibility_reason = Some(format!(
            "Unsupported project type: {}",
            project.project_type
        ));
    } else {
        let validation = (|| -> Result<()> {
            let source = item.project.as_ref().unwrap();
            let entry = source
                .entry
                .as_deref()
                .context("Project entry file missing")?;
            let resources = ProjectResources::new(source)?;
            ensure!(
                resources.exists(entry)?,
                "Project entry/shared asset missing: {entry}"
            );
            // Media still passes a physical file to mpv. Scene/Web keep the
            // manifest path so callers cannot discard the project context.
            if matches!(
                item.wallpaper_type,
                WallpaperType::Video | WallpaperType::Image | WallpaperType::Gif
            ) {
                item.source_path =
                    loose_resource(&root, entry)?.context("Media entry must be a loose file")?;
            }
            Ok(())
        })();
        match validation {
            Err(error) => {
                item.compatibility = CompatibilityStatus::Invalid;
                item.compatibility_reason = Some(error.to_string());
            }
            Ok(())
                if matches!(
                    item.wallpaper_type,
                    WallpaperType::Scene | WallpaperType::Web
                ) =>
            {
                item.compatibility = CompatibilityStatus::RequiresRenderer;
                item.compatibility_reason = Some(format!(
                    "{} renderer is not available yet",
                    item.wallpaper_type.as_str()
                ));
            }
            Ok(()) => {}
        }
    }
    if let Some(preview) = project.preview.as_deref() {
        // Invalid previews do not prevent a valid wallpaper from loading.
        item.thumbnail_path = loose_resource(&root, preview).ok().flatten();
    }
    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn package(path: &Path, name: &str, data: &[u8]) {
        let mut bytes = Vec::new();
        for value in ["PKGV0001", name] {
            if value == name {
                bytes.extend(1u32.to_le_bytes());
            }
            bytes.extend((value.len() as u32).to_le_bytes());
            bytes.extend(value.as_bytes());
        }
        bytes.extend(0u32.to_le_bytes());
        bytes.extend((data.len() as u32).to_le_bytes());
        bytes.extend(data);
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn animated_scene_packages_are_available_after_the_main_package() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("project.json"),
            r#"{"type":"scene","file":"scene.json"}"#,
        )
        .unwrap();
        package(&dir.path().join("scene.pkg"), "scene.json", b"{}");
        package(
            &dir.path().join("gifscene.pkg"),
            "materials/animated.tex",
            b"texture",
        );
        let item = discover_project(dir.path(), SourceType::LocalDirectory, None, vec![]).unwrap();
        let resources = ProjectResources::new(&item.project.unwrap()).unwrap();
        assert!(resources.exists("materials/animated.tex").unwrap());
        assert_eq!(
            resources.read("materials/animated.tex").unwrap(),
            b"texture"
        );
        assert_eq!(resources.read("scene.json").unwrap(), b"{}");
    }

    #[test]
    fn packed_scene_keeps_context_and_properties_without_loose_entry() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("project.json"), r#"{"type":"Scene","file":"scene.json","general":{"properties":{"color":{"type":"color","value":"1 0 0"}}}}"#).unwrap();
        package(&dir.path().join("scene.pkg"), "scene.json", b"{}");
        let item = discover_project(dir.path(), SourceType::LocalDirectory, None, vec![]).unwrap();
        assert_eq!(item.wallpaper_type, WallpaperType::Scene);
        assert_eq!(item.compatibility, CompatibilityStatus::RequiresRenderer);
        let source = item.project.unwrap();
        assert_eq!(source.declared_type, "Scene");
        assert!(source.properties.get("color").is_some());
        let restored: ProjectSource =
            serde_json::from_str(&serde_json::to_string(&source).unwrap()).unwrap();
        assert_eq!(restored, source);
        assert_eq!(
            ProjectResources::new(&source)
                .unwrap()
                .read("scene.json")
                .unwrap(),
            b"{}"
        );
    }

    #[test]
    fn rejects_unsafe_names_and_symlink_escapes() {
        for name in [
            "../secret",
            "/etc/passwd",
            "C:\\secret",
            "a/../../b",
            "a\\b",
            "",
            "a\0b",
        ] {
            assert!(resource_name(name).is_err(), "{name:?}");
        }
        let dir = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        fs::write(outside.path().join("secret"), "secret").unwrap();
        std::os::unix::fs::symlink(outside.path().join("secret"), dir.path().join("entry"))
            .unwrap();
        assert!(loose_resource(dir.path(), "entry").is_err());
        std::os::unix::fs::symlink(
            outside.path().join("secret"),
            dir.path().join("project.json"),
        )
        .unwrap();
        assert!(load_manifest(dir.path()).is_err());
    }

    #[test]
    fn rejects_truncated_out_of_bounds_and_unsafe_packages() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("scene.pkg");
        package(&path, "scene.json", b"{}");
        let mut bytes = fs::read(&path).unwrap();
        bytes.pop();
        fs::write(&path, bytes).unwrap();
        assert!(PackageIndex::open(&path).is_err());
        package(&path, "../secret", b"{}");
        assert!(PackageIndex::open(&path).is_err());
        fs::write(&path, [0xff; 4]).unwrap();
        assert!(PackageIndex::open(&path).is_err());
    }

    #[test]
    fn shared_assets_are_bounded_and_explicit() {
        let dir = TempDir::new().unwrap();
        let shared = TempDir::new().unwrap();
        fs::write(
            dir.path().join("project.json"),
            r#"{"type":"web","file":"index.html"}"#,
        )
        .unwrap();
        fs::write(dir.path().join("index.html"), "<html></html>").unwrap();
        fs::write(shared.path().join("common.js"), "shared").unwrap();
        let item = discover_project(
            dir.path(),
            SourceType::LocalDirectory,
            None,
            vec![shared.path().into()],
        )
        .unwrap();
        let resources = ProjectResources::new(item.project.as_ref().unwrap()).unwrap();
        assert_eq!(resources.read("common.js").unwrap(), b"shared");
        assert!(resources.read("../secret").is_err());
        assert!(resources.read("missing.js").is_err());
    }
}
