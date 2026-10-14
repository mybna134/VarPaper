//! Explicit opt-in integration test. Every runtime/cache/assignment is temporary.
use super::*;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;

struct RuntimeOverride(&'static str, Option<OsString>);
impl Drop for RuntimeOverride {
    fn drop(&mut self) {
        match self.1.take() {
            Some(previous) => std::env::set_var(self.0, previous),
            None => std::env::remove_var(self.0),
        }
    }
}
fn service(root: PathBuf) -> WayvidService {
    WayvidService {
        web_component: Arc::new(WebComponent::new(root)),
        inner: Arc::new(Mutex::new(ServiceInner {
            engine: EngineController::new(),
            events: VecDeque::new(),
        })),
    }
}
async fn scan(service: &WayvidService, project: &Path) -> WallpaperDto {
    service
        .scan_folder(project.to_string_lossy().into_owned())
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.wallpaper_type == "web")
        .unwrap()
}
fn descendants(root: u32) -> std::collections::HashSet<u32> {
    let mut parents = std::collections::HashMap::new();
    for entry in fs::read_dir("/proc").unwrap().flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        if let Ok(stat) = fs::read_to_string(entry.path().join("stat")) {
            if let Some(parent) = stat
                .rsplit_once(')')
                .and_then(|(_, tail)| tail.split_whitespace().nth(1))
                .and_then(|value| value.parse::<u32>().ok())
            {
                parents.insert(pid, parent);
            }
        }
    }
    let mut owned = std::collections::HashSet::from([root]);
    loop {
        let previous = owned.len();
        for (&pid, parent) in &parents {
            if owned.contains(parent) {
                owned.insert(pid);
            }
        }
        if owned.len() == previous {
            return owned;
        }
    }
}

#[tokio::test]
async fn live_uninstall_failure_retry_reinstall_and_restart_preserve_user_sources() {
    let Some(sdk) = std::env::var_os("VARPAPER_TEST_CEF_SDK") else {
        return;
    };
    assert_eq!(std::env::var("VARPAPER_TEST_X11").as_deref(), Ok("1"));
    assert_eq!(std::env::var("XDG_SESSION_TYPE").as_deref(), Ok("x11"));
    assert!(std::env::var_os("WAYLAND_DISPLAY").is_none());
    assert_eq!(std::env::var("SDL_AUDIODRIVER").as_deref(), Ok("dummy"));
    // Hard links stay on the SDK filesystem, avoiding another 1 GiB fixture copy.
    let sdk = PathBuf::from(sdk);
    let temp = tempfile::tempdir_in(sdk.parent().unwrap()).unwrap();
    let root = temp.path().join("component");
    let project = temp.path().join("user-project");
    fs::create_dir(&project).unwrap();
    let manifest = br#"{"type":"web","file":"index.html","general":{"properties":{"caption":{"type":"textinput","value":"saved"}}}}"#;
    fs::write(project.join("project.json"), manifest).unwrap();
    fs::write(project.join("index.html"), "<body style='background:#264a78'><script>window.wallpaperPropertyListener={applyUserProperties:p=>document.title=p.caption.value};setInterval(()=>document.body.style.opacity='.99',20)</script>").unwrap();
    let assignment = temp.path().join("saved-assignment.json");
    let saved = serde_json::to_vec(
        &serde_json::json!({"sourcePath":project,"properties":{"caption":{"value":"saved"}}}),
    )
    .unwrap();
    fs::write(&assignment, &saved).unwrap();
    let _override = RuntimeOverride(
        "VARPAPER_CEF_RUNTIME",
        std::env::var_os("VARPAPER_CEF_RUNTIME"),
    );
    std::env::set_var("VARPAPER_CEF_RUNTIME", root.join("runtime"));
    let counts = wayvid_engine::audio::live_counts();
    let absent = service(root.clone());
    assert_eq!(
        scan(&absent, &project).await.compatibility,
        "requires_renderer"
    );
    assert_eq!(
        absent
            .apply_wallpaper(project.to_string_lossy().into_owned(), None)
            .await
            .unwrap_err()
            .code,
        "requires_renderer"
    );
    assert!(!root.exists());
    drop(absent);

    crate::web_component::install_sdk_fixture(&sdk, &root).unwrap();
    let marker = root.join("runtime/installed.json");
    let record = fs::read(&marker).unwrap();
    for key in ["version", "archive_sha256"] {
        let mut bad: serde_json::Value = serde_json::from_slice(&record).unwrap();
        bad[key] = serde_json::json!("incompatible");
        fs::write(&marker, serde_json::to_vec(&bad).unwrap()).unwrap();
        let incompatible = service(root.clone());
        assert_eq!(incompatible.web_support_status().await.state, "error");
        assert_eq!(
            scan(&incompatible, &project).await.compatibility,
            "requires_renderer"
        );
        assert_eq!(
            incompatible
                .apply_wallpaper(project.to_string_lossy().into_owned(), None)
                .await
                .unwrap_err()
                .code,
            "requires_renderer"
        );
    }
    fs::write(&marker, &record).unwrap();
    let resource = root.join("runtime/icudtl.dat");
    let missing = temp.path().join("missing-icudtl.dat");
    fs::rename(&resource, &missing).unwrap();
    let damaged = service(root.clone());
    assert_eq!(damaged.web_support_status().await.state, "error");
    assert_eq!(
        scan(&damaged, &project).await.compatibility,
        "requires_renderer"
    );
    assert_eq!(
        damaged
            .apply_wallpaper(project.to_string_lossy().into_owned(), None)
            .await
            .unwrap_err()
            .code,
        "requires_renderer"
    );
    drop(damaged);
    fs::rename(&missing, &resource).unwrap();
    let installed = service(root.clone());
    assert_eq!(installed.web_support_status().await.state, "installed");
    assert!(installed.web_support_status().await.disk_bytes > 100_000_000.0);
    assert_eq!(scan(&installed, &project).await.compatibility, "ready");
    installed
        .lock()
        .unwrap()
        .engine
        .start(wayvid_engine::EngineConfig::default())
        .unwrap();
    let image = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packaging/varpaper.png");
    installed
        .apply_wallpaper(image.to_string_lossy().into_owned(), None)
        .await
        .unwrap();
    assert!(installed
        .poll_events()
        .unwrap()
        .iter()
        .any(|event| matches!(event,
            ServiceEvent::WallpaperApplied { path, .. } if path == &image.to_string_lossy()
        )));
    {
        use std::os::unix::fs::PermissionsExt;
        let helper = temp.path().join("incompatible-helper");
        fs::create_dir(&helper).unwrap();
        let binary = helper.join("varpaper-web-host");
        fs::write(&binary, b"must never execute").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let mut metadata: serde_json::Value =
            serde_json::from_str(include_str!("../../native/web/runtime.json")).unwrap();
        metadata["ipc_version"] = serde_json::json!(1);
        fs::write(
            helper.join("runtime.json"),
            serde_json::to_vec(&metadata).unwrap(),
        )
        .unwrap();
        let _host_override =
            RuntimeOverride("VARPAPER_WEB_HOST", std::env::var_os("VARPAPER_WEB_HOST"));
        std::env::set_var("VARPAPER_WEB_HOST", &binary);
        let item = scan(&installed, &project).await;
        assert_eq!(item.compatibility, "requires_renderer");
        assert!(item.compatibility_reason.unwrap().contains("ABI mismatch"));
        let failure = installed
            .apply_wallpaper(project.to_string_lossy().into_owned(), None)
            .await
            .unwrap_err();
        assert_eq!(failure.code, "requires_renderer");
        assert!(failure.message.contains("ABI mismatch"));
        // A stale executable can also lie through a current manifest. Reject its
        // actual protocol before first-frame commit and keep the running image.
        fs::write(
            helper.join("runtime.json"),
            include_str!("../../native/web/runtime.json"),
        )
        .unwrap();
        fs::write(
            &binary,
            br#"#!/usr/bin/python3
import json,socket,struct,sys,time
connection=socket.socket(fileno=int(sys.argv[1].split('=')[1]))
def exact(size):
    result=b''
    while len(result)<size:
        part=connection.recv(size-len(result))
        if not part: raise SystemExit()
        result+=part
    return result
command=json.loads(exact(struct.unpack('<I',exact(4))[0]))
command.update(ipc_version=1,ok=True,bytes=0)
response=json.dumps(command).encode()
connection.sendall(struct.pack('<I',len(response))+response)
time.sleep(10)
"#,
        )
        .unwrap();
        assert_eq!(scan(&installed, &project).await.compatibility, "ready");
        let failure = installed
            .apply_wallpaper(project.to_string_lossy().into_owned(), None)
            .await
            .unwrap_err();
        assert_eq!(failure.code, "apply_failed");
        assert!(failure.message.contains("IPC version mismatch"));
        assert!(installed
            .poll_events()
            .unwrap()
            .iter()
            .all(|event| !matches!(
                event,
                ServiceEvent::WallpaperApplied { .. } | ServiceEvent::WallpaperCleared { .. }
            )));
        assert_eq!(wayvid_engine::audio::live_counts(), counts);
        assert_eq!(fs::read(&assignment).unwrap(), saved);
    }
    installed
        .apply_wallpaper(project.to_string_lossy().into_owned(), None)
        .await
        .unwrap();
    assert!(wayvid_engine::audio::live_counts().0 > counts.0);
    let hosts: Vec<_> = fs::read_dir(root.join("browser-cache"))
        .unwrap()
        .map(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        })
        .collect();
    assert_eq!(hosts.len(), 1);
    let processes = descendants(hosts[0]);
    assert!(processes.len() > 1);

    // Own fixture symlink makes the deletion guard fail, after StopWeb has acked.
    let backup = temp.path().join("component-backup");
    fs::rename(&root, &backup).unwrap();
    symlink(&backup, &root).unwrap();
    assert_eq!(
        installed.uninstall_web_support().await.unwrap_err().code,
        "web_uninstall_failed"
    );
    assert_eq!(installed.web_support_status().await.state, "error");
    assert_eq!(wayvid_engine::audio::live_counts(), counts);
    for pid in processes {
        assert!(
            !Path::new("/proc").join(pid.to_string()).exists(),
            "CEF process {pid} survived StopWeb"
        );
    }
    assert!(backup.join("runtime/libcef.so").exists());
    assert_eq!(fs::read(&assignment).unwrap(), saved);
    assert_eq!(fs::read(project.join("project.json")).unwrap(), manifest);
    fs::remove_file(&root).unwrap();
    fs::rename(&backup, &root).unwrap();
    installed.uninstall_web_support().await.unwrap();
    assert_eq!(installed.web_support_status().await.state, "not_installed");
    assert_eq!(installed.web_support_status().await.disk_bytes, 0.0);
    assert!(!root.exists());
    installed.shutdown().await.unwrap();
    drop(installed);

    let restarted = service(root.clone());
    assert_eq!(
        scan(&restarted, &project).await.compatibility,
        "requires_renderer"
    );
    assert_eq!(
        restarted
            .apply_wallpaper(project.to_string_lossy().into_owned(), None)
            .await
            .unwrap_err()
            .code,
        "requires_renderer"
    );
    assert!(!root.exists());
    drop(restarted);
    crate::web_component::install_sdk_fixture(&sdk, &root).unwrap();
    let reinstalled = service(root.clone());
    assert_eq!(scan(&reinstalled, &project).await.compatibility, "ready");
    reinstalled
        .lock()
        .unwrap()
        .engine
        .start(wayvid_engine::EngineConfig::default())
        .unwrap();
    reinstalled
        .apply_wallpaper(project.to_string_lossy().into_owned(), None)
        .await
        .unwrap();
    reinstalled.uninstall_web_support().await.unwrap();
    reinstalled.shutdown().await.unwrap();
    assert_eq!(wayvid_engine::audio::live_counts(), counts);
    assert_eq!(fs::read(&assignment).unwrap(), saved);
    assert_eq!(fs::read(project.join("project.json")).unwrap(), manifest);
    assert!(project.join("index.html").exists());
    assert!(!root.exists());
}
