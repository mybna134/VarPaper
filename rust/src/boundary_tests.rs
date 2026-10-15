//! Service boundary tests with owned sources, component state and preview files.
use super::*;

fn service(root: &Path) -> WayvidService {
    WayvidService {
        web_component: Arc::new(WebComponent::new(root.join("component"))),
        inner: Arc::new(Mutex::new(ServiceInner {
            engine: EngineController::new(),
            events: VecDeque::new(),
        })),
    }
}

fn image_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packaging/varpaper.png")
}

#[tokio::test]
async fn folder_scan_returns_stable_serializable_dtos_without_an_engine() {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join("wallpapers");
    std::fs::create_dir(&folder).unwrap();
    let image = folder.join("image.png");
    std::fs::copy(image_fixture(), &image).unwrap();
    let project = folder.join("scene");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("scene.json"), "{}").unwrap();
    std::fs::copy(image_fixture(), project.join("preview.png")).unwrap();
    std::fs::write(project.join("project.json"), r#"{"type":"scene","file":"scene.json","title":"Boundary Scene","description":"fixture description","preview":"preview.png"}"#).unwrap();
    std::fs::write(folder.join("ignored.txt"), "not wallpaper").unwrap();
    let service = service(root.path());
    let scan = || service.scan_folder(folder.to_string_lossy().into_owned());
    let items = scan().await.unwrap();
    assert_eq!(
        items.len(),
        2,
        "project preview must not become a library item"
    );
    let scene = items
        .iter()
        .find(|item| item.wallpaper_type == "scene")
        .unwrap();
    assert_eq!(scene.metadata.title.as_deref(), Some("Boundary Scene"));
    assert_eq!(
        scene.metadata.description.as_deref(),
        Some("fixture description")
    );
    assert_eq!(scene.source_type, "local_dir");
    assert_eq!(scene.compatibility, "ready");
    assert!(Path::new(scene.thumbnail_path.as_ref().unwrap()).is_file());
    let context: wayvid_library::ProjectSource =
        serde_json::from_str(scene.project_source_json.as_ref().unwrap()).unwrap();
    assert_eq!(context.root, project.canonicalize().unwrap());
    assert_eq!(context.entry.as_deref(), Some("scene.json"));
    for item in &items {
        let encoded = serde_json::to_value(item).unwrap();
        let decoded: WallpaperDto = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    }
    let mut first_ids: Vec<_> = items.iter().map(|item| item.id.clone()).collect();
    let mut next_ids: Vec<_> = scan()
        .await
        .unwrap()
        .into_iter()
        .map(|item| item.id)
        .collect();
    first_ids.sort();
    next_ids.sort();
    assert_eq!(first_ids, next_ids);
    std::fs::remove_file(image).unwrap();
    let remaining = scan().await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, scene.id);
    assert!(!service.lock().unwrap().engine.is_running());
    assert!(!root.path().join("component").exists());
    assert!(service.poll_events().unwrap().is_empty());
}

#[tokio::test]
async fn image_preview_returns_path_without_starting_engine() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("image.PNG");
    std::fs::copy(image_fixture(), &path).unwrap();
    let service = service(root.path());
    let preview = service
        .load_preview(
            "direct-image".into(),
            path.to_string_lossy().into_owned(),
            "image".into(),
            64,
            32,
        )
        .await
        .unwrap();
    assert_eq!(preview.image_path.as_deref(), path.to_str());
    assert!(preview.bytes.is_empty());
    let decoded: PreviewDto =
        serde_json::from_slice(&serde_json::to_vec(&preview).unwrap()).unwrap();
    assert_eq!(decoded.image_path, preview.image_path);
    assert_eq!(decoded.bytes, preview.bytes);
    assert!(!service.lock().unwrap().engine.is_running());
}

struct PreviewCacheFile(PathBuf);
impl Drop for PreviewCacheFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[tokio::test]
async fn video_preview_returns_encoded_frame_and_reuses_owned_cache() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("video.mp4");
    assert!(std::process::Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=64x32:r=10",
            "-t",
            "0.2",
            "-c:v",
            "mpeg4"
        ])
        .arg(&path)
        .status()
        .expect("ffmpeg is required for video preview acceptance")
        .success());
    let id = format!("boundary-preview-{}", root.path().display());
    let cache = PreviewCacheFile(preview_cache_path(&id));
    assert!(!cache.0.exists());
    let service = service(root.path());
    let preview = || {
        service.load_preview(
            id.clone(),
            path.to_string_lossy().into_owned(),
            "video".into(),
            64,
            32,
        )
    };
    let first = preview().await.unwrap();
    assert!(first.image_path.is_none());
    assert!(first.bytes.starts_with(b"RIFF"));
    assert_eq!(&first.bytes[8..12], b"WEBP");
    assert_eq!(std::fs::read(&cache.0).unwrap(), first.bytes);
    std::fs::remove_file(&path).unwrap();
    let cached = preview().await.unwrap();
    assert_eq!(cached.bytes, first.bytes);
    assert!(cached.image_path.is_none());
    assert!(!service.lock().unwrap().engine.is_running());
}

async fn expect_service_event(service: &WayvidService, expected: impl Fn(&ServiceEvent) -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        let mut found = false;
        for event in service.poll_events().unwrap() {
            match &event {
                ServiceEvent::Error { message, .. }
                | ServiceEvent::WallpaperFailed { message, .. } => panic!("{message}"),
                _ => found |= expected(&event),
            }
        }
        if found {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Wayland service event timed out"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn expect_assignment(service: &WayvidService, output: &str, path: Option<&Path>) {
    use wayvid_engine::{EngineCommand, EngineEvent};
    service
        .lock()
        .unwrap()
        .engine
        .send_command(EngineCommand::GetStatus)
        .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let events = service.lock().unwrap().engine.poll_events();
        for event in events {
            match event {
                EngineEvent::Status(status) => {
                    assert!(status.running);
                    assert_eq!(
                        status
                            .active_wallpapers
                            .get(output)
                            .and_then(|path| path.as_deref()),
                        path
                    );
                    return;
                }
                EngineEvent::Error(error) | EngineEvent::WallpaperFailed { error, .. } => {
                    panic!("{error}")
                }
                _ => {}
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "Wayland status barrier timed out"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn wayland_service_scan_preview_and_playback_lifecycle() {
    if std::env::var_os("VARPAPER_TEST_NESTED_WAYLAND").is_none() {
        return;
    }
    assert_eq!(std::env::var("XDG_SESSION_TYPE").as_deref(), Ok("wayland"));
    assert_eq!(std::env::var("SDL_AUDIODRIVER").as_deref(), Ok("dummy"));
    let runtime = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap());
    assert!(PathBuf::from(std::env::var_os("WAYLAND_DISPLAY").unwrap()).starts_with(runtime));
    let root = tempfile::tempdir().unwrap();
    let image = root.path().join("wallpaper.png");
    std::fs::copy(image_fixture(), &image).unwrap();
    let video = root.path().join("wallpaper.mp4");
    assert!(std::process::Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=64x32:r=20",
            "-t",
            "1",
            "-c:v",
            "mpeg4"
        ])
        .arg(&video)
        .status()
        .unwrap()
        .success());
    let service = service(root.path());
    assert!(!service.initialize().unwrap().engine_running);
    let items = service
        .scan_folder(root.path().to_string_lossy().into_owned())
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    for item in &items {
        let _cache = PreviewCacheFile(preview_cache_path(&item.id));
        let preview = service
            .load_preview(
                item.id.clone(),
                item.source_path.clone(),
                item.wallpaper_type.clone(),
                64,
                32,
            )
            .await
            .unwrap();
        if item.wallpaper_type == "image" {
            assert_eq!(preview.image_path.as_deref(), image.to_str());
        } else {
            assert_eq!(item.wallpaper_type, "video");
            assert!(preview.bytes.starts_with(b"RIFF"));
        }
    }
    eprintln!("Wayland service smoke: scan and image/video previews passed");
    let mut config = EngineConfigDto {
        volume: 0.0,
        fps_limit: Some(24),
        loop_playback: true,
        layout: "contain".into(),
        hwdec: "no".into(),
        mute: true,
        start_time: 0.0,
        playback_rate: 1.0,
        hdr_mode: "disabled".into(),
        tone_mapping_algorithm: "hable".into(),
        tone_mapping_param: 1.0,
        tone_mapping_mode: "hybrid".into(),
        tone_mapping_compute_peak: true,
        auto_play: true,
        pause_on_battery: false,
    };
    service.create_engine(config.clone()).await.unwrap();
    expect_service_event(&service, |event| {
        matches!(event, ServiceEvent::EngineStarted)
    })
    .await;
    let outputs = service.refresh_monitors().await.unwrap();
    assert!(!outputs.is_empty());
    let output = outputs[0].name.clone();
    assert!(outputs[0].width > 0 && outputs[0].height > 0);
    for path in [&image, &video] {
        let source = service
            .describe_wallpaper(path.to_string_lossy().into_owned())
            .unwrap();
        service
            .apply_wallpaper_source(source, Some(output.clone()))
            .await
            .unwrap();
        expect_service_event(&service, |event| matches!(event, ServiceEvent::WallpaperApplied { output: name, path: applied } if name == &output && Path::new(applied) == path)).await;
        config.fps_limit = Some(30);
        service.update_engine_config(config.clone()).await.unwrap();
        expect_assignment(&service, &output, Some(path)).await;
        service.pause(Some(output.clone())).await.unwrap();
        expect_assignment(&service, &output, Some(path)).await;
        service.resume(Some(output.clone())).await.unwrap();
        expect_assignment(&service, &output, Some(path)).await;
        service.clear_wallpaper(Some(output.clone())).await.unwrap();
        expect_service_event(&service, |event| matches!(event, ServiceEvent::WallpaperCleared { output: name } if name == &output)).await;
        expect_assignment(&service, &output, None).await;
        eprintln!(
            "Wayland service smoke: apply/update/pause/resume/clear passed for {}",
            path.display()
        );
    }
    service.stop_engine().await.unwrap();
    expect_service_event(&service, |event| {
        matches!(event, ServiceEvent::EngineStopped)
    })
    .await;
    assert!(!service.lock().unwrap().engine.is_running());
    service.shutdown().await.unwrap();
    eprintln!("Wayland service smoke: stop and shutdown passed");
}
