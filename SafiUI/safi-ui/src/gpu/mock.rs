//! CPU-side `Renderer` implementation for tests. Records each `submit`'s
//! batches in order plus lifecycle counters.

use crate::commands::Command;

use super::batch::Batcher;
use super::{DrawBatch, Renderer};

#[derive(Default)]
pub struct MockRenderer {
    pub frames: u32,
    pub batches: Vec<DrawBatch>,
    pub dpi: f32,
    pub releases: u32,
    pub recreates: u32,
}

impl MockRenderer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Renderer for MockRenderer {
    fn begin_frame(&mut self, dpi_scale: f32) {
        self.dpi = dpi_scale;
        self.frames += 1;
    }

    fn submit(&mut self, commands: &[Command]) {
        self.batches.extend(Batcher::batch(commands));
    }

    fn end_frame(&mut self) {}

    fn release_resources(&mut self) {
        self.releases += 1;
    }

    fn recreate_resources(&mut self) {
        self.recreates += 1;
    }
}
