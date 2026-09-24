//! In-process lifecycle wrapper for the Linux playback engine.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use tracing::info;
use wayvid_engine::{spawn_engine, EngineCommand, EngineConfig, EngineEvent, EngineHandle};

pub struct EngineController {
    handle: Option<EngineHandle>,
    events_rx: Option<Receiver<EngineEvent>>,
    outputs_ready: bool,
    outputs: HashMap<String, wayvid_engine::OutputInfo>,
}

impl EngineController {
    pub fn new() -> Self {
        Self {
            handle: None,
            events_rx: None,
            outputs_ready: false,
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
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.request_shutdown();
            let _ = handle.join();
        }
        self.events_rx = None;
        self.outputs_ready = false;
        self.outputs.clear();
    }

    pub fn poll_events(&mut self) -> Vec<EngineEvent> {
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

    pub fn apply_wallpaper(&self, output: Option<String>, path: PathBuf) -> Result<(), String> {
        self.send_command(EngineCommand::ApplyWallpaper { output, path })
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
