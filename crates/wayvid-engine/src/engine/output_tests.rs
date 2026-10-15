//! Opt-in native fixtures. The runner must own three outputs and the compositor.
use super::*;
use session::{PlaybackState, SessionSnapshot};
use std::path::Path;
use std::time::{Duration, Instant};

struct RunningEngine(Option<EngineHandle>);
impl std::ops::Deref for RunningEngine {
    type Target = EngineHandle;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}
impl Drop for RunningEngine {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.request_shutdown();
            handle.join().unwrap();
        }
    }
}

#[track_caller]
fn wait(events: &StdReceiver<EngineEvent>, matches: impl Fn(&EngineEvent) -> bool) -> EngineEvent {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(Instant::now() < deadline, "Output event timed out");
        match events.recv_timeout(Duration::from_millis(100)) {
            Ok(event) if matches(&event) => return event,
            Ok(EngineEvent::Error(error) | EngineEvent::WallpaperFailed { error, .. }) => {
                panic!("{error}")
            }
            _ => {}
        }
    }
}

fn snapshots(handle: &EngineHandle) -> HashMap<String, SessionSnapshot> {
    let (tx, rx) = mpsc::channel();
    handle.send(EngineCommand::InspectSessions(tx)).unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap()
}

fn counts(handle: &EngineHandle) -> command::LifecycleCounts {
    let (tx, rx) = mpsc::channel();
    handle.send(EngineCommand::LifecycleCounts(tx)).unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap()
}

fn apply(handle: &EngineHandle, events: &StdReceiver<EngineEvent>, path: &Path, output: &str) {
    handle
        .send(EngineCommand::ApplyWallpaper {
            path: path.into(),
            output: Some(output.into()),
        })
        .unwrap();
    wait(
        events,
        |event| matches!(event, EngineEvent::WallpaperApplied { path: actual, output: name } if actual == path && name == output),
    );
}

#[test]
fn mixed_outputs_first_frames_scale_resize_pause_and_frame_limit() {
    if std::env::var_os("VARPAPER_TEST_MIXED_OUTPUTS").is_none() {
        return;
    }
    assert!(std::env::var_os("VARPAPER_WEB_HOST").is_some());
    let control =
        std::env::var("VARPAPER_TEST_OUTPUT_CONTROL").expect("Owned output control required");
    let assets = tempfile::tempdir().unwrap();
    let scene = assets.path().join("scene");
    crate::scene::fixtures::export(&crate::scene::fixtures::image_resources(), &scene).unwrap();
    let web = assets.path().join("web");
    std::fs::create_dir(&web).unwrap();
    std::fs::write(
        web.join("project.json"),
        r#"{"type":"web","file":"index.html"}"#,
    )
    .unwrap();
    std::fs::write(web.join("index.html"), "<body style='margin:0'><canvas id='c' width='64' height='32'></canvas><script>let n=0;setInterval(()=>{let x=c.getContext('2d');x.fillStyle=`rgb(${++n%255},80,160)`;x.fillRect(0,0,64,32)},30)</script>").unwrap();
    let video = assets.path().join("video.mp4");
    assert!(std::process::Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=64x32:r=30",
            "-t",
            "3",
            "-c:v",
            "mpeg4"
        ])
        .arg(&video)
        .status()
        .unwrap()
        .success());
    let gif = assets.path().join("animation.gif");
    assert!(std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&video)
        .args(["-t", "1"])
        .arg(&gif)
        .status()
        .unwrap()
        .success());
    let image = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/varpaper.png");
    let mut config = EngineConfig {
        fps_limit: Some(20),
        ..Default::default()
    };
    let (handle, events) = spawn_engine(config.clone()).unwrap();
    let handle = RunningEngine(Some(handle));
    wait(&events, |event| matches!(event, EngineEvent::Started));
    handle.send(EngineCommand::GetOutputs).unwrap();
    let EngineEvent::OutputsList(mut outputs) = wait(&events, |event| {
        matches!(event, EngineEvent::OutputsList(_))
    }) else {
        unreachable!()
    };
    outputs.sort_by(|a, b| a.name.cmp(&b.name));
    if outputs
        .iter()
        .any(|output| output.name.starts_with("Fixture"))
    {
        outputs.retain(|output| output.name.starts_with("Fixture"));
    }
    assert_eq!(
        outputs.len(),
        3,
        "Runner must provide exactly three owned outputs"
    );
    let baseline = counts(&handle);
    for (path, output) in [&video, &scene, &web].into_iter().zip(&outputs) {
        apply(&handle, &events, path, &output.name);
    }
    let initial = snapshots(&handle);
    assert_eq!(initial.len(), 3);
    for output in &outputs {
        assert_eq!(
            initial[&output.name].buffer_size,
            Some((output.width, output.height))
        );
        assert!(initial[&output.name].presented_frames > 0);
    }
    let name = &outputs[1].name;
    handle
        .send(EngineCommand::Pause {
            output: Some(name.clone()),
        })
        .unwrap();
    let paused = snapshots(&handle);
    assert_eq!(paused[name].state, PlaybackState::Paused);
    assert_eq!(paused[&outputs[0].name].state, PlaybackState::Playing);
    assert_eq!(paused[&outputs[2].name].state, PlaybackState::Playing);
    let started = Instant::now();
    // Event traffic must not bypass the render deadline.
    for _ in 0..30 {
        handle.send(EngineCommand::GetStatus).unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
    let after = snapshots(&handle);
    assert_eq!(paused[name].presented_frames, after[name].presented_frames);
    let elapsed = started.elapsed().as_secs_f64();
    for output in [&outputs[0], &outputs[2]] {
        let frames = after[&output.name].presented_frames - paused[&output.name].presented_frames;
        assert!(frames > 0, "{} did not continue", output.name);
        assert!(
            frames as f64 <= elapsed * 20.0 + 2.0,
            "{} exceeded frame limit: {frames}/{elapsed}",
            output.name
        );
    }
    handle
        .send(EngineCommand::Pause {
            output: Some(outputs[2].name.clone()),
        })
        .unwrap();
    // Resize every type, including paused Scene/Web, through real output changes.
    for output in &outputs {
        assert!(std::process::Command::new(&control)
            .args([&output.name, "800", "600", "2"])
            .status()
            .unwrap()
            .success());
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let current = snapshots(&handle);
        if outputs
            .iter()
            .all(|output| current[&output.name].buffer_size == Some((800, 600)))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Mixed resize timed out: {current:?}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    let resized = snapshots(&handle);
    assert_eq!(resized[name].state, PlaybackState::Paused);
    assert_eq!(resized[name].path.as_deref(), Some(scene.as_path()));
    assert_eq!(
        resized[&outputs[0].name].path.as_deref(),
        Some(video.as_path())
    );
    assert_eq!(
        resized[&outputs[2].name].path.as_deref(),
        Some(web.as_path())
    );
    assert!(resized[name].presented_frames > paused[name].presented_frames);
    assert_eq!(resized[&outputs[2].name].state, PlaybackState::Paused);
    assert!(resized[&outputs[2].name].presented_frames > after[&outputs[2].name].presented_frames);
    std::thread::sleep(Duration::from_millis(150));
    let still_paused = snapshots(&handle);
    for output in [&outputs[1], &outputs[2]] {
        assert_eq!(
            resized[&output.name].presented_frames,
            still_paused[&output.name].presented_frames
        );
    }
    handle
        .send(EngineCommand::Resume {
            output: Some(name.clone()),
        })
        .unwrap();
    handle
        .send(EngineCommand::Resume {
            output: Some(outputs[2].name.clone()),
        })
        .unwrap();
    // Media types also share the same surface and source transaction path.
    for path in [&image, &gif] {
        apply(&handle, &events, path, &outputs[0].name);
        let current = snapshots(&handle);
        assert_eq!(current[name].path.as_deref(), Some(scene.as_path()));
        assert_eq!(
            current[&outputs[2].name].path.as_deref(),
            Some(web.as_path())
        );
        // Allow the initial config notification to settle, then distinguish
        // static Image from animated GIF without relying on render attempts.
        std::thread::sleep(Duration::from_millis(300));
        let before = snapshots(&handle)[&outputs[0].name].presented_frames;
        std::thread::sleep(Duration::from_millis(200));
        let after = snapshots(&handle)[&outputs[0].name].presented_frames;
        if path == &image {
            assert_eq!(before, after, "Static Image kept swapping buffers");
        } else {
            assert!(after > before, "GIF did not animate");
        }
    }
    // Zero is accepted consistently by both scheduling paths.
    config.fps_limit = Some(0);
    handle.send(EngineCommand::UpdateConfig(config)).unwrap();
    snapshots(&handle);
    // Pausing a fresh Web session must still allow its first static frame.
    let output = &outputs[2].name;
    handle
        .send(EngineCommand::ClearWallpaper {
            output: Some(output.clone()),
        })
        .unwrap();
    wait(
        &events,
        |event| matches!(event, EngineEvent::WallpaperCleared { output: name } if name == output),
    );
    handle
        .send(EngineCommand::ApplyWallpaper {
            path: web.clone(),
            output: Some(output.clone()),
        })
        .unwrap();
    handle
        .send(EngineCommand::Pause {
            output: Some(output.clone()),
        })
        .unwrap();
    wait(
        &events,
        |event| matches!(event, EngineEvent::WallpaperApplied { output: name, .. } if name == output),
    );
    let initial = snapshots(&handle)[output].clone();
    assert_eq!(initial.state, PlaybackState::Paused);
    assert!(initial.presented_frames > 0);
    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(
        snapshots(&handle)[output].presented_frames,
        initial.presented_frames
    );

    // Cleanup must be independent of renderer type and of the reason for
    // removal. The GUI owns saved descriptions and reapplies on OutputAdded.
    handle
        .send(EngineCommand::ClearWallpaper { output: None })
        .unwrap();
    assert!(snapshots(&handle).is_empty());
    assert_eq!(counts(&handle), baseline);
    let control_output = |name: &str, action: &str| {
        assert!(std::process::Command::new(&control)
            .args([name, action])
            .status()
            .unwrap()
            .success());
    };
    for path in [&video, &scene, &web, &image, &gif] {
        eprintln!("Lifecycle: {}", path.display());
        for output in &outputs {
            apply(&handle, &events, path, &output.name);
        }
        let before = counts(&handle);
        let output = &outputs[1].name;
        control_output(output, "disconnect");
        wait(
            &events,
            |event| matches!(event, EngineEvent::OutputRemoved(name) if name == output),
        );
        assert_eq!(snapshots(&handle).len(), 2);
        let disconnected = counts(&handle);
        assert_eq!(disconnected.native, (before.native.0, before.native.1 - 1));
        if path == &web {
            assert_eq!(disconnected.web[1], before.web[1] - 1);
            assert_eq!(disconnected.audio.0, before.audio.0 - 1);
        }
        control_output(output, "reconnect");
        wait(
            &events,
            |event| matches!(event, EngineEvent::OutputAdded(info) if &info.name == output),
        );
        assert!(!snapshots(&handle).contains_key(output));
        // Reapply the saved description after receiving the reconnect event.
        apply(&handle, &events, path, output);
        assert_eq!(counts(&handle), before);
        handle
            .send(EngineCommand::ClearWallpaper {
                output: Some(output.clone()),
            })
            .unwrap();
        wait(
            &events,
            |event| matches!(event, EngineEvent::WallpaperCleared { output: name } if name == output),
        );
        assert_eq!(snapshots(&handle).len(), 2);
        handle
            .send(EngineCommand::ClearWallpaper { output: None })
            .unwrap();
        assert!(snapshots(&handle).is_empty());
        assert_eq!(counts(&handle), baseline, "cleanup for {}", path.display());
    }
    // Shutdown while all three renderer families are still active. Backend
    // teardown asserts native, Scene and Web owner/GPU counts on its thread.
    for (path, output) in [&video, &scene, &web].into_iter().zip(&outputs) {
        apply(&handle, &events, path, &output.name);
    }
    drop(handle);
    assert_eq!(crate::audio::live_counts(), baseline.audio);
}
