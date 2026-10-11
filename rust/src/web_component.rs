//! Optional Chromium runtime. Nothing here downloads until `install` is called.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::bridge::WebSupportDto;

const VERSION: &str = "135.0.17+gcbc1c5b+chromium-135.0.7049.52";
const ARCHIVE_SHA256: &str = "24ac1980e62be7b1aea2c337d6bd4bc770dccd8b37e47c6db94b397f13ec4dc6";
const DISTRIBUTION: &str = "cef_binary_135.0.17+gcbc1c5b+chromium-135.0.7049.52_linux64_minimal";
const URL: &str = "https://cef-builds.spotifycdn.com/cef_binary_135.0.17%2Bgcbc1c5b%2Bchromium-135.0.7049.52_linux64_minimal.tar.bz2";
const MAX_DOWNLOAD: u64 = 1024 * 1024 * 1024;
const MAX_EXPANDED: u64 = 3 * 1024 * 1024 * 1024;
const REQUIRED: &[&str] = &[
    "libcef.so",
    "libEGL.so",
    "libGLESv2.so",
    "v8_context_snapshot.bin",
    "icudtl.dat",
    "chrome_100_percent.pak",
    "chrome_200_percent.pak",
    "resources.pak",
    "locales/en-US.pak",
    "LICENSE.txt",
    "CREDITS.html",
];

#[derive(Serialize, Deserialize)]
struct Installed {
    version: String,
    archive_sha256: String,
    files: BTreeMap<String, (u64, String)>,
}

pub(crate) struct WebComponent {
    root: PathBuf,
    status: Mutex<WebSupportDto>,
    busy: AtomicBool,
    cancel: AtomicBool,
    blocked: AtomicBool,
    operation: Mutex<()>,
}
impl WebComponent {
    pub(crate) fn shared(root: PathBuf) -> Arc<Self> {
        static COMPONENTS: OnceLock<Mutex<HashMap<PathBuf, Weak<WebComponent>>>> = OnceLock::new();
        let mut components = COMPONENTS.get_or_init(Default::default).lock().unwrap();
        if let Some(component) = components.get(&root).and_then(Weak::upgrade) {
            return component;
        }
        let component = Arc::new(Self::new(root.clone()));
        components.insert(root, Arc::downgrade(&component));
        component
    }
    pub(crate) fn new(root: PathBuf) -> Self {
        // A previous process cannot still own this staging directory: running jobs
        // retain the shared Arc, and GTK enforces the application's single instance.
        if root.join("staging").exists() {
            let _ = remove_owned(&root.join("staging"));
        }
        let status = match inspect(&root) {
            Ok((installed, bytes)) => dto(
                if installed {
                    "installed"
                } else {
                    "not_installed"
                },
                bytes,
                None,
            ),
            Err(error) => dto(
                "error",
                disk_usage(&root).unwrap_or(0),
                Some(error.to_string()),
            ),
        };
        Self {
            root,
            status: Mutex::new(status),
            busy: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            blocked: AtomicBool::new(false),
            operation: Mutex::new(()),
        }
    }
    pub(crate) fn status(&self) -> WebSupportDto {
        let mut status = self.status.lock().unwrap().clone();
        if status.state == "installed" || status.state == "error" {
            status.disk_bytes = disk_usage(&self.root).unwrap_or(status.disk_bytes as u64) as f64;
        }
        if self.blocked.load(Ordering::Acquire) {
            status.state = "uninstalling".into();
        }
        status
    }
    pub(crate) fn playback_allowed(&self) -> bool {
        !self.blocked.load(Ordering::Acquire) && self.status.lock().unwrap().state == "installed"
    }
    pub(crate) fn prepare_uninstall(&self) -> Result<()> {
        let _operation = self.operation.lock().unwrap();
        if self
            .blocked
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            bail!("Web component uninstallation is already running");
        }
        self.cancel();
        Ok(())
    }
    pub(crate) fn finish_uninstall(&self) {
        self.blocked.store(false, Ordering::Release);
    }
    fn update(&self, state: &str, downloaded: u64, total: Option<u64>) {
        let mut status = self.status.lock().unwrap();
        status.state = state.into();
        status.downloaded_bytes = downloaded as f64;
        status.total_bytes = total.map(|v| v as f64);
        status.error = None;
    }
    pub(crate) fn install(self: &Arc<Self>) -> Result<()> {
        let _operation = self.operation.lock().unwrap();
        if self.blocked.load(Ordering::Acquire) {
            bail!("Web component uninstallation is in progress");
        }
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            bail!("No verified Chromium runtime is available for this architecture");
        }
        if self
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            bail!("A Web component operation is already running");
        }
        if self.status().state == "installed" {
            self.busy.store(false, Ordering::Release);
            return Ok(());
        }
        self.cancel.store(false, Ordering::Release);
        self.update("downloading", 0, None);
        let this = Arc::clone(self);
        match std::thread::Builder::new()
            .name("web-component-install".into())
            .spawn(move || {
                let result = this.download_and_install();
                let cleanup = remove_owned(&this.root.join("staging"));
                let cleanup_failed = cleanup.is_err();
                let result = cleanup.and(result);
                let final_status = match result {
                    Ok(_) => dto("installed", disk_usage(&this.root).unwrap_or(0), None),
                    Err(_) if this.cancel.load(Ordering::Acquire) && !cleanup_failed => {
                        match inspect(&this.root) {
                            Ok((installed, _)) => dto(
                                if installed {
                                    "installed"
                                } else {
                                    "not_installed"
                                },
                                disk_usage(&this.root).unwrap_or(0),
                                None,
                            ),
                            Err(error) => dto(
                                "error",
                                disk_usage(&this.root).unwrap_or(0),
                                Some(format!("{error:#}")),
                            ),
                        }
                    }
                    Err(error) => dto(
                        "error",
                        disk_usage(&this.root).unwrap_or(0),
                        Some(format!("{error:#}")),
                    ),
                };
                *this.status.lock().unwrap() = final_status;
                this.busy.store(false, Ordering::Release);
            }) {
            Ok(_) => Ok(()),
            Err(error) => {
                *self.status.lock().unwrap() = dto(
                    "error",
                    disk_usage(&self.root).unwrap_or(0),
                    Some(error.to_string()),
                );
                self.busy.store(false, Ordering::Release);
                Err(error.into())
            }
        }
    }
    pub(crate) fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    pub(crate) fn busy(&self) -> bool {
        self.busy.load(Ordering::Acquire)
    }
    /// The caller must stop Web sessions/host processes before entering this method.
    pub(crate) fn uninstall(&self) -> Result<()> {
        let _operation = self.operation.lock().unwrap();
        if self
            .busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            self.cancel();
            bail!("Web component cancellation is in progress; retry uninstall after it finishes");
        }
        self.update("uninstalling", 0, None);
        let result = (|| {
            ensure_no_links(&self.root)?;
            remove_owned(&self.root)?;
            Ok(())
        })();
        *self.status.lock().unwrap() = match &result {
            Ok(()) => dto("not_installed", 0, None),
            Err(error) => dto(
                "error",
                disk_usage(&self.root).unwrap_or(0),
                Some(format!("{error:#}")),
            ),
        };
        self.busy.store(false, Ordering::Release);
        result
    }
    fn download_and_install(&self) -> Result<u64> {
        ensure_no_links(&self.root)?;
        fs::create_dir_all(&self.root)?;
        let stage = self.root.join("staging");
        remove_owned(&stage)?;
        fs::create_dir(&stage)?;
        let archive = stage.join("runtime.tar.bz2");
        // Reject all redirects: the compiled URL already identifies the official artifact.
        let (downloaded, total) = tokio::runtime::Runtime::new()?.block_on(async {
            let client = reqwest::Client::builder().https_only(true)
                .redirect(reqwest::redirect::Policy::none()).connect_timeout(Duration::from_secs(15))
                .read_timeout(Duration::from_secs(15)).timeout(Duration::from_secs(600)).build()?;
            let request = client.get(URL).send();
            tokio::pin!(request);
            let mut response = loop {
                tokio::select! {
                    result = &mut request => break result?.error_for_status()?,
                    _ = tokio::time::sleep(Duration::from_millis(100)) => cancelled(&self.cancel)?,
                }
            };
            if !response.status().is_success() { bail!("Official Chromium source returned HTTP {}", response.status()); }
            let total = response.content_length();
            if total.is_some_and(|v| v > MAX_DOWNLOAD) { bail!("Chromium download exceeds size budget"); }
            let mut output = File::create(&archive)?;
            let mut downloaded = 0_u64;
            loop {
                cancelled(&self.cancel)?;
                tokio::select! {
                    result = response.chunk() => {
                        let Some(bytes) = result? else { break; };
                        downloaded += bytes.len() as u64;
                        if downloaded > MAX_DOWNLOAD { bail!("Chromium download exceeds size budget"); }
                        output.write_all(&bytes).context("Cannot write Chromium download; check disk space")?;
                        self.update("downloading", downloaded, total);
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => cancelled(&self.cancel)?,
                }
            }
            output.sync_all()?;
            if total.is_some_and(|v| v != downloaded) { bail!("Chromium download was truncated"); }
            Ok::<_, anyhow::Error>((downloaded, total))
        })?;
        self.install_archive(&stage, downloaded, total)
    }
    fn install_archive(&self, stage: &Path, downloaded: u64, total: Option<u64>) -> Result<u64> {
        let archive = stage.join("runtime.tar.bz2");
        self.update("verifying", downloaded, total);
        if hash_file(&archive, &self.cancel)? != ARCHIVE_SHA256 {
            bail!("Chromium archive checksum mismatch");
        }
        self.update("installing", downloaded, total);
        let runtime = stage.join("runtime");
        extract(&archive, &runtime, &self.cancel)?;
        let installed = manifest(&runtime, &self.cancel)?;
        let bytes = installed.files.values().map(|(bytes, _)| bytes).sum();
        let marker = File::create(runtime.join("installed.json"))?;
        serde_json::to_writer(&marker, &installed)?;
        marker.sync_all()?;
        cancelled(&self.cancel)?;
        let target = self.root.join("runtime");
        if target.exists() {
            remove_owned(&target)?;
        }
        fs::rename(runtime, target).context("Cannot commit Chromium runtime installation")?;
        Ok(bytes)
    }
}
fn dto(state: &str, bytes: u64, error: Option<String>) -> WebSupportDto {
    WebSupportDto {
        state: state.into(),
        version: VERSION.into(),
        disk_bytes: bytes as f64,
        downloaded_bytes: 0.0,
        total_bytes: None,
        error,
        source: "https://cef-builds.spotifycdn.com/".into(),
        supported: cfg!(all(target_os = "linux", target_arch = "x86_64")),
    }
}
fn cancelled(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        bail!("Web component installation cancelled");
    }
    Ok(())
}
fn ensure_no_links(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(info) if info.file_type().is_symlink() => {
                bail!("Web component path contains a symbolic link")
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
fn remove_owned(path: &Path) -> Result<()> {
    ensure_no_links(path)?;
    match fs::symlink_metadata(path) {
        Ok(info) if info.is_dir() => {
            fs::remove_dir_all(path)?;
        }
        Ok(_) => bail!("Web component directory is not a directory"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}
fn hash_file(path: &Path, cancel: &AtomicBool) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        cancelled(cancel)?;
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn extract(archive: &Path, target: &Path, cancel: &AtomicBool) -> Result<()> {
    fs::create_dir(target)?;
    let decoded = bzip2::read::BzDecoder::new(File::open(archive)?);
    let mut archive = tar::Archive::new(decoded);
    let mut seen = HashSet::new();
    let mut expanded = 0_u64;
    for (index, entry) in archive.entries()?.enumerate() {
        cancelled(cancel)?;
        if index >= 100_000 {
            bail!("Chromium archive exceeds entry budget");
        }
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if path.to_str().is_none()
            || path.as_os_str().len() > 4096
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            bail!("Unsafe Chromium archive path");
        }
        if !seen.insert(path.clone()) {
            bail!("Duplicate Chromium archive entry");
        }
        let relative = path
            .strip_prefix(DISTRIBUTION)
            .context("Unexpected Chromium archive root")?;
        if !entry.header().entry_type().is_file() && !entry.header().entry_type().is_dir() {
            bail!("Chromium archive links/special files are forbidden");
        }
        expanded = expanded
            .checked_add(entry.size())
            .context("Chromium archive size overflow")?;
        if expanded > MAX_EXPANDED || entry.size() > 2 * 1024 * 1024 * 1024 {
            bail!("Chromium archive exceeds unpack budget");
        }
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let selected = relative
            .strip_prefix("Release")
            .ok()
            .or_else(|| relative.strip_prefix("Resources").ok());
        let destination = if let Some(relative) = selected {
            target.join(relative)
        } else if relative == Path::new("LICENSE.txt") || relative == Path::new("CREDITS.html") {
            target.join(relative)
        } else {
            continue;
        };
        let parent = destination
            .parent()
            .context("Missing Chromium resource parent")?;
        fs::create_dir_all(parent)?;
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(destination)?;
        let mut buffer = [0; 64 * 1024];
        loop {
            cancelled(cancel)?;
            let n = entry.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])?;
        }
        file.sync_all()?;
    }
    Ok(())
}
fn walk_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let info = fs::symlink_metadata(entry.path())?;
        if info.file_type().is_symlink() {
            bail!("Web runtime contains symbolic links");
        }
        if info.is_dir() {
            walk_files(root, &entry.path(), files)?;
        } else if info.is_file() {
            files.push(entry.path().strip_prefix(root)?.to_path_buf());
        } else {
            bail!("Web runtime contains special files");
        }
    }
    Ok(())
}
fn manifest(runtime: &Path, cancel: &AtomicBool) -> Result<Installed> {
    for required in REQUIRED {
        if !runtime.join(required).is_file() {
            bail!("Missing Chromium runtime resource: {required}");
        }
    }
    let mut paths = vec![];
    walk_files(runtime, runtime, &mut paths)?;
    let mut files = BTreeMap::new();
    for path in paths {
        files.insert(
            path.to_string_lossy().into_owned(),
            (
                fs::metadata(runtime.join(&path))?.len(),
                hash_file(&runtime.join(path), cancel)?,
            ),
        );
    }
    Ok(Installed {
        version: VERSION.into(),
        archive_sha256: ARCHIVE_SHA256.into(),
        files,
    })
}
fn inspect(root: &Path) -> Result<(bool, u64)> {
    ensure_no_links(root)?;
    let runtime = root.join("runtime");
    if !runtime.exists() {
        return Ok((false, 0));
    }
    let marker = runtime.join("installed.json");
    ensure_no_links(&marker)?;
    if fs::metadata(&marker)?.len() > 1024 * 1024 {
        bail!("Invalid Web component installation record");
    }
    let installed: Installed = serde_json::from_reader(File::open(marker)?)?;
    if installed.version != VERSION || installed.archive_sha256 != ARCHIVE_SHA256 {
        bail!("Web component version mismatch; reinstall support");
    }
    for required in REQUIRED {
        if !installed.files.contains_key(*required) {
            bail!("Incomplete Web component installation");
        }
    }
    let cancel = AtomicBool::new(false);
    let mut bytes = 0;
    for (name, (size, hash)) in installed.files {
        let path = Path::new(&name);
        if path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            bail!("Unsafe Web component installation record");
        }
        let path = runtime.join(path);
        ensure_no_links(&path)?;
        if fs::metadata(&path)?.len() != size || hash_file(&path, &cancel)? != hash {
            bail!("Damaged Web runtime; reinstall support");
        }
        bytes += size;
    }
    Ok((true, bytes))
}
fn disk_usage(root: &Path) -> Result<u64> {
    if !root.exists() {
        return Ok(0);
    }
    fn directory_bytes(directory: &Path) -> Result<u64> {
        let mut total = 0_u64;
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            let bytes = if metadata.is_dir() {
                directory_bytes(&entry.path())?
            } else {
                metadata.len()
            };
            total = total
                .checked_add(bytes)
                .context("Component disk usage overflow")?;
        }
        Ok(total)
    }
    ensure_no_links(root)?;
    directory_bytes(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bzip2::{write::BzEncoder, Compression};
    use std::io::Cursor;

    fn archive(root: &Path, files: Vec<(String, Vec<u8>)>) -> PathBuf {
        let path = root.join("fixture.tar.bz2");
        let encoder = BzEncoder::new(File::create(&path).unwrap(), Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        for (name, bytes) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_entry_type(tar::EntryType::Regular);
            if name.starts_with('/') || name.contains("..") {
                // Raw header permits malicious paths rejected by append_data.
                let destination = &mut header.as_mut_bytes()[..100];
                assert!(name.len() < destination.len());
                destination[..name.len()].copy_from_slice(name.as_bytes());
                header.set_cksum();
                builder.append(&header, Cursor::new(bytes)).unwrap();
            } else {
                builder
                    .append_data(&mut header, &name, Cursor::new(bytes))
                    .unwrap();
            }
        }
        builder.into_inner().unwrap().finish().unwrap();
        path
    }
    fn resources() -> Vec<(String, Vec<u8>)> {
        REQUIRED
            .iter()
            .map(|name| {
                let prefix = if *name == "LICENSE.txt" || *name == "CREDITS.html" {
                    ""
                } else if name.ends_with(".pak") || *name == "icudtl.dat" {
                    "Resources/"
                } else {
                    "Release/"
                };
                (
                    format!("{DISTRIBUTION}/{prefix}{name}"),
                    format!("fixture: {name}").into_bytes(),
                )
            })
            .collect()
    }
    fn commit(runtime: &Path) {
        let installed = manifest(runtime, &AtomicBool::new(false)).unwrap();
        serde_json::to_writer(
            File::create(runtime.join("installed.json")).unwrap(),
            &installed,
        )
        .unwrap();
    }
    #[test]
    fn installer_and_host_build_share_the_pinned_runtime_manifest() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../native/web/runtime.json")).unwrap();
        assert_eq!(manifest["version"], VERSION);
        assert_eq!(manifest["url"], URL);
        assert_eq!(manifest["sha256"], ARCHIVE_SHA256);
        assert_eq!(manifest["distribution"], DISTRIBUTION);
        assert_eq!(manifest["ipc_version"], 1);
    }
    #[test]
    fn default_status_does_not_create_or_download_anything() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        let manager = WebComponent::new(root.clone());
        assert_eq!(manager.status().state, "not_installed");
        assert!(!root.exists());
        assert!(!manager.busy());
        let a = WebComponent::shared(root.clone());
        let b = WebComponent::shared(root);
        assert!(Arc::ptr_eq(&a, &b));
    }
    #[test]
    fn valid_component_restart_verifies_files_and_uninstall_preserves_wallpaper() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        fs::create_dir(&root).unwrap();
        let runtime = root.join("runtime");
        let archive = archive(temp.path(), resources());
        extract(&archive, &runtime, &AtomicBool::new(false)).unwrap();
        commit(&runtime);
        let wallpaper = temp.path().join("wallpaper.html");
        fs::write(&wallpaper, "user project").unwrap();
        let manager = WebComponent::new(root.clone());
        assert_eq!(manager.status().state, "installed");
        assert!(manager.status().disk_bytes > 0.0);
        manager.uninstall().unwrap();
        assert!(!root.exists());
        assert_eq!(fs::read_to_string(wallpaper).unwrap(), "user project");
        assert_eq!(WebComponent::new(root).status().state, "not_installed");
    }
    #[test]
    fn same_size_corruption_and_missing_resources_never_enable_component() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        fs::create_dir(&root).unwrap();
        let runtime = root.join("runtime");
        let archive = archive(temp.path(), resources());
        extract(&archive, &runtime, &AtomicBool::new(false)).unwrap();
        commit(&runtime);
        let library = runtime.join("libcef.so");
        let size = fs::metadata(&library).unwrap().len();
        fs::write(&library, vec![b'x'; size as usize]).unwrap();
        assert_eq!(WebComponent::new(root.clone()).status().state, "error");
        fs::remove_file(runtime.join("resources.pak")).unwrap();
        assert!(inspect(&root).is_err());
    }
    #[test]
    fn unsafe_paths_duplicates_bad_root_and_truncation_are_rejected() {
        for entries in [
            vec![(format!("{DISTRIBUTION}/../x"), vec![0])],
            vec![("/absolute".into(), vec![0])],
            vec![("other/Release/libcef.so".into(), vec![0])],
            vec![
                (format!("{DISTRIBUTION}/Release/libcef.so"), vec![0]),
                (format!("{DISTRIBUTION}/Release/libcef.so"), vec![1]),
            ],
        ] {
            let temp = tempfile::tempdir().unwrap();
            let input = archive(temp.path(), entries);
            assert!(extract(
                &input,
                &temp.path().join("runtime"),
                &AtomicBool::new(false)
            )
            .is_err());
            assert!(!temp.path().join("escape").exists());
        }
        let temp = tempfile::tempdir().unwrap();
        let input = archive(temp.path(), resources());
        let file = File::options().write(true).open(&input).unwrap();
        file.set_len(32).unwrap();
        assert!(extract(
            &input,
            &temp.path().join("runtime"),
            &AtomicBool::new(false)
        )
        .is_err());
    }
    #[test]
    fn cancelled_extract_and_abandoned_stage_never_enable_component() {
        let temp = tempfile::tempdir().unwrap();
        let input = archive(temp.path(), resources());
        let root = temp.path().join("component");
        fs::create_dir(&root).unwrap();
        assert!(extract(&input, &root.join("runtime"), &AtomicBool::new(true)).is_err());
        assert_ne!(WebComponent::new(root.clone()).status().state, "installed");
        fs::create_dir(root.join("staging")).unwrap();
        fs::write(root.join("staging/partial"), b"partial").unwrap();
        let manager = WebComponent::new(root.clone());
        assert!(!root.join("staging").exists());
        assert_ne!(manager.status().state, "installed");
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_do_not_redirect_uninstall_or_inspection() {
        let temp = tempfile::tempdir().unwrap();
        let outside = temp.path().join("user");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), b"keep").unwrap();
        let root = temp.path().join("component");
        std::os::unix::fs::symlink(&outside, &root).unwrap();
        let manager = WebComponent::new(root);
        assert_eq!(manager.status().state, "error");
        assert!(manager.uninstall().is_err());
        assert_eq!(fs::read(outside.join("keep")).unwrap(), b"keep");
    }
    #[test]
    fn uninstall_blocks_new_installation_until_shutdown_acknowledgement() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        let manager = Arc::new(WebComponent::new(root.clone()));
        manager.prepare_uninstall().unwrap();
        assert_eq!(manager.status().state, "uninstalling");
        assert!(!manager.playback_allowed());
        assert!(manager.install().is_err());
        assert!(manager.prepare_uninstall().is_err());
        assert!(!root.exists());
        manager.uninstall().unwrap();
        manager.finish_uninstall();
        assert_eq!(manager.status().state, "not_installed");
    }
    #[test]
    fn failed_archive_verification_preserves_existing_runtime() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        fs::create_dir(&root).unwrap();
        let runtime = root.join("runtime");
        let input = archive(temp.path(), resources());
        extract(&input, &runtime, &AtomicBool::new(false)).unwrap();
        commit(&runtime);
        let manager = WebComponent::new(root.clone());
        let stage = root.join("staging");
        fs::create_dir(&stage).unwrap();
        fs::copy(input, stage.join("runtime.tar.bz2")).unwrap();
        assert!(manager.install_archive(&stage, 0, None).is_err());
        assert!(inspect(&root).unwrap().0);
        remove_owned(&stage).unwrap();
        manager.uninstall().unwrap();
    }
    #[test]
    fn missing_resources_and_wrong_version_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        fs::create_dir(&root).unwrap();
        let runtime = root.join("runtime");
        let input = archive(temp.path(), resources());
        extract(&input, &runtime, &AtomicBool::new(false)).unwrap();
        let mut installed = manifest(&runtime, &AtomicBool::new(false)).unwrap();
        installed.version = "different ABI".into();
        serde_json::to_writer(
            File::create(runtime.join("installed.json")).unwrap(),
            &installed,
        )
        .unwrap();
        assert!(inspect(&root).is_err());
        fs::remove_file(runtime.join("libcef.so")).unwrap();
        assert!(manifest(&runtime, &AtomicBool::new(false)).is_err());
    }
    #[test]
    fn archive_links_are_rejected_without_touching_their_target() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("link.tar.bz2");
        let encoder = BzEncoder::new(File::create(&path).unwrap(), Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o644);
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_link_name("../../outside").unwrap();
        builder
            .append_data(
                &mut header,
                format!("{DISTRIBUTION}/Release/libcef.so"),
                std::io::empty(),
            )
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap();
        fs::write(temp.path().join("outside"), b"keep").unwrap();
        assert!(extract(&path, &temp.path().join("runtime"), &AtomicBool::new(false)).is_err());
        assert_eq!(fs::read(temp.path().join("outside")).unwrap(), b"keep");
    }
    #[test]
    fn official_runtime_fixture_if_requested() {
        let Some(input) = std::env::var_os("VARPAPER_TEST_CEF_ARCHIVE") else {
            return;
        };
        let input = Path::new(&input);
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("component");
        fs::create_dir(&root).unwrap();
        let stage = root.join("staging");
        fs::create_dir(&stage).unwrap();
        fs::hard_link(input, stage.join("runtime.tar.bz2")).unwrap();
        let manager = WebComponent::new(root.clone());
        // Constructor cleans abandoned staging; recreate it for the explicit fixture action.
        fs::create_dir(&stage).unwrap();
        fs::hard_link(input, stage.join("runtime.tar.bz2")).unwrap();
        manager
            .install_archive(&stage, fs::metadata(input).unwrap().len(), None)
            .unwrap();
        assert!(root.join("runtime/installed.json").is_file());
        remove_owned(&stage).unwrap();
        assert_eq!(WebComponent::new(root.clone()).status().state, "installed");
        manager.uninstall().unwrap();
        assert!(!root.exists());
    }
}
