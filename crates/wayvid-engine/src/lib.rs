//! wayvid-engine: Video rendering engine for wayvid
//!
//! This crate provides the core rendering functionality:
//! - Wayland layer-shell surface management
//! - MPV video playback integration  
//! - EGL/OpenGL rendering
//! - Vulkan rendering (optional)
//!
//! # Architecture
//!
//! ```text
//! EngineHandle (commands and lifecycle)
//!     └── EngineState / X11 backend
//!         ├── Native surfaces (Wayland layer-shell / X11 windows)
//!         ├── EglContext (shared rendering context)
//!         ├── WallpaperSession (per-output)
//!         │   └── WallpaperRenderer (MPV / Rust Scene / Web)
//!         └── Output tracking (OutputManager / XRandR)
//! ```

pub mod audio;
pub mod egl;
pub mod engine;
pub mod frame_timing;
pub mod mpv;
pub mod properties;
pub mod renderer;
pub mod scene;
pub mod types;
pub mod wayland;
pub mod web;

// Re-exports - Engine API
pub use engine::{
    discover_x11_outputs, spawn_engine, EngineCommand, EngineConfig, EngineEvent, EngineHandle,
    EngineStatus, WallpaperSession,
};

// Re-export the command sender for the in-process Flutter service.
pub use calloop::channel::Sender as CommandSender;

// Re-exports - Low-level components
pub use egl::{EglContext, EglWindow};
pub use frame_timing::FrameTiming;
pub use mpv::{MpvPlayer, VideoConfig};
pub use wayland::OutputManager;

pub use types::{
    calculate_layout, HdrMetadata, HdrMode, HwdecMode, LayoutMode, LayoutTransform,
    OutputHdrCapabilities, OutputInfo, RenderBackend, ToneMappingConfig, TransferFunction,
};
