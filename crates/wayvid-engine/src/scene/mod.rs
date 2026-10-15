//! Rust Scene rendering and assignment-local resource owners.

pub mod assets;
pub mod clock;
mod draw;
#[cfg(test)]
pub(crate) mod fixtures;
mod geometry;
mod gpu;
pub mod model;
pub mod particle;
mod particle_draw;
mod program;
mod resources;
mod script;
mod script_timers;
pub mod shader;
mod sound;
mod text;
pub mod texture;
pub use resources::SceneResources;
pub mod runtime;

/// Assignment-owned resources, sampled on the render thread by native fixtures.
#[cfg(test)]
pub(crate) fn live_counts() -> ((usize, usize), (usize, usize), usize, usize, usize) {
    (
        gpu::counts(),
        program::counts(),
        particle_draw::counts(),
        script::counts(),
        sound::counts(),
    )
}
