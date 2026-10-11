//! In-process lifecycle wrapper for the Linux playback engine.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use tracing::info;
use wayvid_engine::{spawn_engine, EngineCommand, EngineConfig, EngineEvent, EngineHandle};

pub struct EngineController {
    handle: Option<EngineHandle>,
    events_rx: Option<Receiver<EngineEvent>>,
    outputs_ready: bool,
    pending_events: VecDeque<EngineEvent>,
    outputs: HashMap<String, wayvid_engine::OutputInfo>,
}

impl EngineController {
    pub fn new() -> Self {
        Self {
            handle: None,
            events_rx: None,
            outputs_ready: false,
            pending_events: VecDeque::new(),
            outputs: HashMap::new(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle
            .as_ref()
            .map(|handle| handle.is_running())
            .unwrap_or(false)
    }

    pub fn start(&mut self, config: EngineConfig) -> Result<(), String> {
        if self.is_running() {
            return Err("Engine is already running".to_string());
        }
        info!("Starting integrated playback engine");
        let (handle, events_rx) = spawn_engine(config).map_err(|error| error.to_string())?;
        self.handle = Some(handle);
        self.events_rx = Some(events_rx);
        self.outputs_ready = false;
        self.outputs.clear();
        self.pending_events.clear();
        Ok(())
    }

    pub fn stop_web(&self) -> Result<(), String> {
        if !self.is_running() {
            return Ok(());
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        self.send_command(EngineCommand::StopWeb(sender))?;
        receiver
            .recv_timeout(Duration::from_secs(30))
            .map_err(|error| format!("Web shutdown acknowledgement failed: {error}"))?
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.request_shutdown();
            let _ = handle.join();
        }
        self.events_rx = None;
        self.outputs_ready = false;
        self.outputs.clear();
        self.pending_events.clear();
    }

    pub fn poll_events(&mut self) -> Vec<EngineEvent> {
        let mut received: Vec<_> = self.pending_events.drain(..).collect();
        received.extend(self.drain_incoming());
        received
    }

    fn drain_incoming(&mut self) -> Vec<EngineEvent> {
        let mut received = Vec::new();
        if let Some(receiver) = &self.events_rx {
            while let Ok(event) = receiver.try_recv() {
                match &event {
                    EngineEvent::OutputAdded(info) => {
                        self.outputs.insert(info.name.clone(), info.clone());
                    }
                    EngineEvent::OutputRemoved(name) => {
                        self.outputs.remove(name);
                    }
                    EngineEvent::OutputsList(outputs) => {
                        self.outputs = outputs
                            .iter()
                            .map(|info| (info.name.clone(), info.clone()))
                            .collect();
                    }
                    _ => {}
                }
                self.outputs_ready = !self.outputs.is_empty();
                received.push(event);
            }
        }
        received
    }

    pub fn outputs(&self) -> Vec<wayvid_engine::OutputInfo> {
        self.outputs.values().cloned().collect()
    }

    pub fn wait_for_outputs(&mut self, timeout: Duration) -> Result<(), String> {
        if self.outputs_ready {
            return Ok(());
        }
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let events = self.poll_events();
            if let Some(message) = events.iter().find_map(|event| match event {
                EngineEvent::Error(message) => Some(message.clone()),
                _ => None,
            }) {
                return Err(message);
            }
            if self.outputs_ready {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("Timed out waiting for display outputs".to_string())
    }

    pub fn send_command(&self, command: EngineCommand) -> Result<(), String> {
        self.handle
            .as_ref()
            .ok_or_else(|| "Engine is not running".to_string())?
            .send(command)
            .map_err(|error| error.to_string())
    }

    pub fn update_config(&self, config: EngineConfig) -> Result<(), String> {
        self.send_command(EngineCommand::UpdateConfig(config))
    }

    pub fn apply_wallpaper(&mut self, output: Option<String>, path: PathBuf) -> Result<(), String> {
        if !self.is_running() {
            return Err("Engine is not running".into());
        }
        let before = self.drain_incoming();
        self.pending_events.extend(before);
        let mut waiting: HashSet<String> = output
            .clone()
            .map(|name| HashSet::from([name]))
            .unwrap_or_else(|| self.outputs.keys().cloned().collect());
        if waiting.is_empty() {
            return Err("No display outputs available".into());
        }
        self.send_command(EngineCommand::ApplyWallpaper {
            output,
            path: path.clone(),
        })?;
        let deadline = Instant::now() + Duration::from_secs(12);
        while Instant::now() < deadline {
            let events = self.drain_incoming();
            let mut failure = None;
            for event in &events {
                match event {
                    EngineEvent::WallpaperApplied {
                        output,
                        path: applied,
                    } if applied == &path => {
                        waiting.remove(output);
                    }
                    EngineEvent::WallpaperFailed {
                        output,
                        path: failed,
                        error,
                    } if failed == &path && waiting.contains(output) => {
                        failure = Some(format!("{output}: {error}"));
                    }
                    EngineEvent::OutputRemoved(output) if waiting.contains(output) => {
                        failure = Some(format!(
                            "Output {output} disconnected during wallpaper initialization"
                        ));
                    }
                    EngineEvent::Error(error) => {
                        failure = Some(error.clone());
                    }
                    _ => {}
                }
            }
            self.pending_events.extend(events);
            if let Some(error) = failure {
                return Err(error);
            }
            if waiting.is_empty() {
                return Ok(());
            }
            if !self.is_running() {
                return Err("Engine stopped before wallpaper first frame".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Err("Timed out waiting for wallpaper first frame".into())
    }

    pub fn clear_wallpaper(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::ClearWallpaper { output })
    }

    pub fn pause(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::Pause { output })
    }

    pub fn resume(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::Resume { output })
    }
}

impl Default for EngineController {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EngineController {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_returns_after_first_frame_and_keeps_correlated_events() {
        if std::env::var_os("VARPAPER_TEST_X11").is_none() {
            return;
        }
        let mut controller = EngineController::new();
        controller.start(EngineConfig::default()).unwrap();
        controller.wait_for_outputs(Duration::from_secs(5)).unwrap();
        let output = controller.outputs()[0].name.clone();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packaging/varpaper.png");
        controller
            .apply_wallpaper(Some(output.clone()), path.clone())
            .unwrap();
        assert!(controller.poll_events().iter().any(|event| matches!(event,
            EngineEvent::WallpaperApplied { output: applied, path: source } if applied == &output && source == &path)));
        let missing = path.with_file_name("missing-candidate.png");
        assert!(controller
            .apply_wallpaper(Some(output.clone()), missing.clone())
            .is_err());
        assert!(controller.poll_events().iter().any(|event| matches!(event,
            EngineEvent::WallpaperFailed { output: failed, path: source, .. } if failed == &output && source == &missing)));
        controller.stop();
    }

    #[test]
    fn commands_fail_when_engine_is_not_running() {
        let mut controller = EngineController::default();
        assert!(!controller.is_running());
        assert!(controller.outputs().is_empty());
        assert!(controller.poll_events().is_empty());

        let not_running = Err("Engine is not running".to_string());
        assert_eq!(controller.pause(None), not_running);
        assert_eq!(controller.resume(Some("DP-1".into())), not_running);
        assert_eq!(controller.clear_wallpaper(None), not_running);
        assert_eq!(
            controller.apply_wallpaper(None, PathBuf::from("/tmp/a.mp4")),
            not_running
        );
        assert_eq!(
            controller.update_config(EngineConfig::default()),
            not_running
        );

        // Stopping an idle controller is a no-op.
        controller.stop();
        assert!(!controller.is_running());
    }

    #[test]
    fn wait_for_outputs_times_out_without_engine() {
        let mut controller = EngineController::new();
        assert_eq!(
            controller.wait_for_outputs(Duration::from_millis(20)),
            Err("Timed out waiting for display outputs".to_string())
        );
    }
}
