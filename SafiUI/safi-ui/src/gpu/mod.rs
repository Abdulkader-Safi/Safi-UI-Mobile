//! GPU rendering pipeline (PRD §8).
//!
//! - The [`Renderer`] trait is the public surface. `SDL_GPU` and the test
//!   [`MockRenderer`] both implement it.
//! - The [`Batcher`] folds a `&[Command]` into [`DrawBatch`]es per PRD §8.3.
//!   It is pure Rust and host-testable.
//! - [`SdlGpuRenderer`] lives behind the `gpu` Cargo feature so the default
//!   host build stays free of native dependencies.

pub mod batch;
pub mod mock;

#[cfg(feature = "gpu")]
pub mod sdl;

pub use batch::{BatchKind, Batcher, DrawBatch};
pub use mock::MockRenderer;

#[cfg(feature = "gpu")]
pub use sdl::SdlGpuRenderer;

use crate::commands::Command;

pub trait Renderer {
    fn begin_frame(&mut self, dpi_scale: f32);
    fn submit(&mut self, commands: &[Command]);
    fn end_frame(&mut self);
    fn release_resources(&mut self);
    fn recreate_resources(&mut self);
}
