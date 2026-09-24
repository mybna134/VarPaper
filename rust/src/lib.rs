mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
// Rust service and Flutter bridge for wayvid-gui.
//
// Flutter owns presentation, persistence, tray, and process lifecycle. Rust
// is an in-process service for scanning and Wayland/MPV playback.

mod bridge;
mod engine;

pub use bridge::*;
