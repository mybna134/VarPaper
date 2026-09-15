mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
// Rust service and Flutter bridge for wayvid-gui.
//
// The presentation layer lives in Flutter. This crate deliberately owns the
// playback engine, Wayland integration, settings, tray, single-instance
// handling, and the legacy Unix IPC server.

rust_i18n::i18n!("locales");

mod bridge;
mod engine;
mod ipc_server;
mod settings;
mod single_instance;
mod tray;

pub use bridge::*;
