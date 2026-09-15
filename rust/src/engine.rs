//! Playback engine integration for wayvid-gui
//!
//! This module manages the embedded PlaybackEngine lifecycle and provides
//! a bridge between the Flutter service and the Wayland rendering engine.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use tracing::info;
use wayvid_engine::{spawn_engine, EngineCommand, EngineConfig, EngineEvent, EngineHandle};

use crate::ipc_server::{IpcServer, SharedStatusCache, ShowWindowFlag};

/// Wrapper around the engine handle with convenience methods
pub struct EngineController {
    /// The actual engine handle
    handle: Option<EngineHandle>,
    /// Event receiver
    events_rx: Option<Receiver<EngineEvent>>,
    /// IPC server for wayvid-ctl communication
    ipc_server: Option<IpcServer>,
    /// Shared status cache for IPC queries
    status_cache: Option<SharedStatusCache>,
    /// Flag to signal show window request from IPC
    show_window_flag: Option<ShowWindowFlag>,
    /// Whether the engine has finished enumerating at least one Wayland output
    outputs_ready: bool,
}

#[allow(dead_code)] // Methods reserved for future UI integration
impl EngineController {
    /// Create an uninitialized engine controller
    pub fn new() -> Self {
        Self {
            handle: None,
            events_rx: None,
            ipc_server: None,
            status_cache: None,
            show_window_flag: None,
            outputs_ready: false,
        }
    }

    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.is_running())
            .unwrap_or(false)
    }

    /// Start the engine
    pub fn start(&mut self, config: EngineConfig) -> Result<(), String> {
        if self.is_running() {
            return Err("Engine is already running".to_string());
        }

        info!("Starting integrated playback engine");

        let (handle, events_rx) = spawn_engine(config).map_err(|e| e.to_string())?;

        // Start IPC server for wayvid-ctl communication
        let mut ipc_server = IpcServer::new();
        if let Err(e) = ipc_server.start(handle.command_sender()) {
            // IPC server failure is non-fatal, just log it
            tracing::warn!("Failed to start IPC server: {}", e);
        } else {
            info!("IPC server started for wayvid-ctl integration");
            self.status_cache = Some(ipc_server.status_cache());
            self.show_window_flag = Some(ipc_server.show_window_flag());
            self.ipc_server = Some(ipc_server);
        }

        self.handle = Some(handle);
        self.events_rx = Some(events_rx);
        self.outputs_ready = false;

        info!("Playback engine started");
        Ok(())
    }

    /// Stop the engine
    pub fn stop(&mut self) {
        info!("Stopping playback engine");

        // Stop IPC server first
        if let Some(mut ipc_server) = self.ipc_server.take() {
            ipc_server.stop();
            info!("IPC server stopped");
        }
        self.status_cache = None;
        self.show_window_flag = None;

        if let Some(handle) = self.handle.take() {
            handle.request_shutdown();
            // Join with timeout (don't block forever)
            let _ = handle.join();
        }
        self.events_rx = None;

        info!("Playback engine stopped");
    }

    /// Check and consume the show window flag
    ///
    /// Returns true if a ShowWindow IPC request was received since last check
    pub fn check_show_window_request(&self) -> bool {
        use std::sync::atomic::Ordering;
        if let Some(ref flag) = self.show_window_flag {
            flag.swap(false, Ordering::SeqCst)
        } else {
            false
        }
    }

    /// Poll for engine events (non-blocking) and update IPC status cache
    pub fn poll_events(&mut self) -> Vec<EngineEvent> {
        let mut received = Vec::new();
        if let Some(ref rx) = self.events_rx {
            while let Ok(event) = rx.try_recv() {
                received.push(event);
            }
        }

        for event in &received {
            if matches!(event, EngineEvent::OutputAdded(_)) {
                self.outputs_ready = true;
            }
            // Update IPC status cache based on events
            self.update_status_cache(event);
        }

        received
    }

    /// Wait until Wayland outputs have been enumerated before accepting a
    /// wallpaper command. The engine thread starts asynchronously, so a
    /// command sent immediately after start could otherwise see an empty
    /// output list and be discarded without an error.
    pub fn wait_for_outputs(&mut self, timeout: Duration) -> Result<(), String> {
        if self.outputs_ready {
            return Ok(());
        }

        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let events = self.poll_events();
            if events.iter().any(|event| matches!(event, EngineEvent::Error(message) if message.contains("Wayland") || message.contains("output"))) {
                return Err("Wayland engine failed while enumerating outputs".to_string());
            }
            if self.outputs_ready {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Err("Timed out waiting for Wayland outputs".to_string())
    }

    /// Update the IPC status cache based on engine events
    fn update_status_cache(&self, event: &EngineEvent) {
        if let Some(ref cache) = self.status_cache {
            if let Ok(mut cache) = cache.write() {
                match event {
                    EngineEvent::Started => {
                        cache.running = true;
                    }
                    EngineEvent::Stopped => {
                        cache.running = false;
                        cache.active_wallpapers.clear();
                    }
                    EngineEvent::WallpaperApplied { output, path } => {
                        cache.active_wallpapers.insert(output.clone(), path.clone());
                    }
                    EngineEvent::WallpaperCleared { output } => {
                        cache.active_wallpapers.remove(output);
                    }
                    _ => {}
                }
            }
        }
    }
    /// Send a command to the engine
    pub fn send_command(&self, command: EngineCommand) -> Result<(), String> {
        if let Some(ref handle) = self.handle {
            handle.send(command).map_err(|e| e.to_string())
        } else {
            Err("Engine is not running".to_string())
        }
    }

    /// Apply wallpaper to an output
    pub fn apply_wallpaper(&self, output: Option<String>, path: PathBuf) -> Result<(), String> {
        self.send_command(EngineCommand::ApplyWallpaper { output, path })
    }

    /// Clear wallpaper from an output
    pub fn clear_wallpaper(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::ClearWallpaper { output })
    }

    /// Set volume for an output
    pub fn set_volume(&self, output: String, volume: f32) -> Result<(), String> {
        self.send_command(EngineCommand::SetVolume { output, volume })
    }

    /// Pause playback on an output
    pub fn pause(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::Pause { output })
    }

    /// Resume playback on an output
    pub fn resume(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::Resume { output })
    }

    /// Request engine shutdown
    pub fn shutdown(&self) -> Result<(), String> {
        self.send_command(EngineCommand::Shutdown)
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

/// Default engine configuration from app settings
pub fn default_engine_config(settings: &crate::settings::AppSettings) -> EngineConfig {
    use wayvid_engine::VideoConfig;

    EngineConfig {
        video: VideoConfig {
            volume: settings.playback.volume as f64,
            ..VideoConfig::default()
        },
        auto_play: true,
        fps_limit: settings.playback.fps_limit,
        pause_on_battery: settings.power.pause_on_battery,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_controller_creation() {
        let controller = EngineController::new();
        assert!(!controller.is_running());
    }
}
