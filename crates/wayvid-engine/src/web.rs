//! Shared out-of-process CEF owner and per-output OpenGL presentation.
use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

mod native_host;
mod renderer;
pub use renderer::WebRenderer;

const MAX_JSON: usize = 65536;
const MAX_FRAME: usize = 64 * 1024 * 1024;
const IPC_VERSION: u64 = 2;
const RUNTIME_MANIFEST: &str = include_str!("../../../native/web/runtime.json");
thread_local! { static HOST: RefCell<Weak<RefCell<Host>>> = const { RefCell::new(Weak::new()) }; }

#[cfg(test)]
thread_local! {
    // Host processes, renderer/browser owners, programs, textures, vertex arrays.
    static LIVE: std::cell::Cell<[usize; 5]> = const { std::cell::Cell::new([0; 5]) };
}
#[cfg(test)]
pub(crate) fn live_counts() -> [usize; 5] {
    LIVE.with(|counts| counts.get())
}
#[cfg(test)]
fn track(index: usize, acquired: bool) {
    LIVE.with(|counts| {
        let mut live = counts.get();
        if acquired {
            live[index] += 1;
        } else {
            live[index] -= 1;
        }
        counts.set(live);
    });
}

#[cfg(test)]
pub(crate) fn host_is_alive() -> bool {
    HOST.with(|host| host.borrow().strong_count() != 0)
}

pub fn runtime_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VARPAPER_CEF_RUNTIME") {
        return Ok(path.into());
    }
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .context("Missing application data directory")?;
    Ok(root.join("varpaper/components/web/runtime"))
}
pub fn host_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VARPAPER_WEB_HOST") {
        return Ok(path.into());
    }
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .context("Missing executable directory")?
        .join("libexec/varpaper-web-host"))
}
pub fn available() -> bool {
    runtime_path()
        .is_ok_and(|root| root.join("installed.json").is_file() && root.join("libcef.so").is_file())
        && validated_host_path().is_ok()
}
pub fn validate_host_installation() -> Result<()> {
    validated_host_path().map(|_| ())
}
fn validate_host(host: &Path, manifest: &Path) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let executable = std::fs::metadata(host).context("Web host is missing")?;
    ensure!(
        executable.is_file() && executable.permissions().mode() & 0o111 != 0,
        "Web host is not executable"
    );
    let file = std::fs::File::open(manifest).context("Web host manifest is missing")?;
    let mut bytes = Vec::new();
    file.take(MAX_JSON as u64 + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= MAX_JSON,
        "Web host manifest exceeds size budget"
    );
    let actual: Value = serde_json::from_slice(&bytes).context("Invalid Web host manifest")?;
    let expected: Value = serde_json::from_str(RUNTIME_MANIFEST)?;
    ensure!(
        expected["ipc_version"] == IPC_VERSION && actual == expected,
        "Web host version or ABI mismatch; reinstall the application"
    );
    // Zypak re-launches this executable in a child sandbox. A temporary
    // alias may only exist in the parent, so launch the verified real path.
    std::fs::canonicalize(host).context("Cannot resolve Web host executable")
}
fn validated_host_path() -> Result<PathBuf> {
    let host = host_path()?;
    let parent = host.parent().context("Missing Web host directory")?;
    let manifest = if std::env::var_os("VARPAPER_WEB_HOST").is_some() {
        parent.join("runtime.json")
    } else {
        parent
            .parent()
            .context("Missing application directory")?
            .join("data/runtime.json")
    };
    validate_host(&host, &manifest)
}
struct Host {
    socket: UnixStream,
    child: Child,
    request: u64,
    next_id: u64,
    dead: bool,
}
impl Host {
    fn call(&mut self, mut message: Value) -> Result<(Value, Vec<u8>)> {
        ensure!(
            !self.dead,
            "Web host exited; reapply the wallpaper to retry"
        );
        self.request += 1;
        message["request"] = self.request.into();
        let result = self.exchange(&message);
        if result.is_err() {
            self.dead = true;
            self.terminate();
        }
        let (response, pixels) = result?;
        if response["ok"] != true {
            bail!(
                "{}",
                response["error"]
                    .as_str()
                    .unwrap_or("Web wallpaper host failed")
            );
        }
        Ok((response, pixels))
    }
    fn exchange(&mut self, message: &Value) -> Result<(Value, Vec<u8>)> {
        let data = serde_json::to_vec(message)?;
        ensure!(data.len() <= MAX_JSON, "Web command exceeds size budget");
        self.socket.write_all(&(data.len() as u32).to_le_bytes())?;
        self.socket.write_all(&data)?;
        let mut header = [0; 4];
        self.socket.read_exact(&mut header)?;
        let size = u32::from_le_bytes(header) as usize;
        ensure!(
            size > 0 && size <= MAX_JSON,
            "Invalid Web host response size"
        );
        let mut bytes = vec![0; size];
        self.socket.read_exact(&mut bytes)?;
        let response: Value = serde_json::from_slice(&bytes)?;
        ensure!(
            response["request"] == message["request"]
                && response["id"] == message["id"]
                && response["generation"] == message["generation"],
            "Stale Web host response"
        );
        ensure!(
            response["ipc_version"] == IPC_VERSION,
            "Web host IPC version mismatch"
        );
        let count = response["bytes"]
            .as_u64()
            .context("Missing Web frame size")?;
        ensure!(count <= MAX_FRAME as u64, "Web frame exceeds size budget");
        let mut pixels = vec![0; count as usize];
        self.socket.read_exact(&mut pixels)?;
        Ok((response, pixels))
    }
    fn terminate(&mut self) {
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        let _ = self.child.wait();
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        if !self.dead {
            let _ = self.call(json!({"op":"shutdown", "id":0,"generation":0}));
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        self.terminate();
        #[cfg(test)]
        track(0, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn helper_metadata_rejects_missing_corrupt_and_incompatible_installations() {
        let directory = tempfile::tempdir().unwrap();
        let host = directory.path().join("varpaper-web-host");
        let manifest = directory.path().join("runtime.json");
        std::fs::write(&host, b"fixture").unwrap();
        std::fs::set_permissions(&host, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("missing"));
        std::fs::write(&manifest, b"broken").unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("Invalid"));
        let mut incompatible: Value = serde_json::from_str(RUNTIME_MANIFEST).unwrap();
        for key in ["ipc_version", "version", "architecture", "sha256"] {
            let original = incompatible[key].clone();
            incompatible[key] = json!("incompatible");
            std::fs::write(&manifest, serde_json::to_vec(&incompatible).unwrap()).unwrap();
            assert!(validate_host(&host, &manifest)
                .unwrap_err()
                .to_string()
                .contains("mismatch"));
            incompatible[key] = original;
        }
        std::fs::write(&manifest, vec![b' '; MAX_JSON + 1]).unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("budget"));
        std::fs::write(&manifest, RUNTIME_MANIFEST).unwrap();
        assert_eq!(validate_host(&host, &manifest).unwrap(), host);
        let alias = directory.path().join("host-alias");
        std::os::unix::fs::symlink(&host, &alias).unwrap();
        assert_eq!(validate_host(&alias, &manifest).unwrap(), host);
        std::fs::set_permissions(&host, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("executable"));
    }

    #[test]
    fn response_abi_mismatch_terminates_the_helper_without_consuming_pixels() {
        let (socket, mut peer) = UnixStream::pair().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let child = unsafe {
            Command::new("sleep")
                .arg("10")
                .pre_exec(|| {
                    if libc::setsid() < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                })
                .spawn()
                .unwrap()
        };
        let thread = std::thread::spawn(move || {
            let mut header = [0; 4];
            peer.read_exact(&mut header).unwrap();
            let mut bytes = vec![0; u32::from_le_bytes(header) as usize];
            peer.read_exact(&mut bytes).unwrap();
            let mut response: Value = serde_json::from_slice(&bytes).unwrap();
            response["ipc_version"] = json!(IPC_VERSION - 1);
            response["bytes"] = json!(4);
            response["ok"] = json!(true);
            let bytes = serde_json::to_vec(&response).unwrap();
            peer.write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
            peer.write_all(&bytes).unwrap();
            // No pixels: validation must reject the protocol before reading them.
        });
        let mut host = Host {
            socket,
            child,
            request: 0,
            next_id: 0,
            dead: false,
        };
        track(0, true);
        assert!(host
            .call(json!({"id":0,"generation":0}))
            .unwrap_err()
            .to_string()
            .contains("IPC version mismatch"));
        assert!(host.dead);
        assert!(host.child.try_wait().unwrap().is_some());
        thread.join().unwrap();
    }
}
