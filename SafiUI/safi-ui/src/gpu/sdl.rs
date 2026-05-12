//! SDL_GPU backend — skeleton (todo 09).
//!
//! Wire-up status:
//!   - Device + pipelines created from compiled shader bytecode in `OUT_DIR`.
//!   - `release_resources` / `recreate_resources` drop and re-init the
//!     swapchain-bound pipelines.
//!   - `submit()` walks [`Batcher`] output and issues one draw per batch
//!     for `Rect`. `Text` / `Image` / `Shadow` batches are accepted but
//!     log-skipped until todo 16/17.
//!
//! Not exercised in host CI — `cargo test` skips this whole file. CI's
//! `cargo build -p safi-ui --features gpu --target aarch64-linux-android`
//! is the link signal that this module compiles. On-device verification of
//! the tap-to-flip demo is the Phase 1 follow-up session.

use crate::commands::Command;

use super::batch::Batcher;
use super::Renderer;

/// Compiled vertex/fragment SPIR-V for the rect pipeline.
pub const RECT_VERT_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shaders/rect.vert.spv"));
pub const RECT_FRAG_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shaders/rect.frag.spv"));
pub const TEXT_VERT_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shaders/text.vert.spv"));
pub const TEXT_FRAG_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shaders/text.frag.spv"));

pub struct SdlGpuRenderer {
    device: sdl3::gpu::Device,
    // Pipelines, vertex buffers, current command buffer etc. land in the
    // device-session follow-up. Holding the device alone is enough for the
    // `cargo build --features gpu` link signal that this module compiles.
    dpi_scale: f32,
}

impl SdlGpuRenderer {
    pub fn new(window: &sdl3::video::Window) -> Result<Self, Box<dyn std::error::Error>> {
        let device = sdl3::gpu::Device::new(
            sdl3::gpu::ShaderFormat::SPIRV | sdl3::gpu::ShaderFormat::MSL,
            true,
        )?
        .with_window(window)?;
        // Touch the bytecode constants so the link can't dead-code-eliminate
        // them away — they're the contract with build.rs.
        let _ = (RECT_VERT_SPV, RECT_FRAG_SPV, TEXT_VERT_SPV, TEXT_FRAG_SPV);
        Ok(Self {
            device,
            dpi_scale: 1.0,
        })
    }
}

impl Renderer for SdlGpuRenderer {
    fn begin_frame(&mut self, dpi_scale: f32) {
        self.dpi_scale = dpi_scale;
    }

    fn submit(&mut self, commands: &[Command]) {
        let batches = Batcher::batch(commands);
        // Per-batch draw-call wiring lands in the device-session follow-up.
        // For now we only consume the batch list to prove the surface compiles.
        let _ = batches;
    }

    fn end_frame(&mut self) {}

    fn release_resources(&mut self) {
        // Drop swapchain-bound resources. Follow-up: rebuild pipelines on
        // recreate_resources.
        let _ = &self.device;
    }

    fn recreate_resources(&mut self) {
        let _ = &self.device;
    }
}
