//! Wayland output tracking shared by the engine.
//!
//! The engine module owns layer-shell protocol dispatch and native surfaces.

pub mod output;

pub use output::OutputManager;
