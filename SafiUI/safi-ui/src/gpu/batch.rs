//! Batching: fold a flat `&[Command]` into a `Vec<DrawBatch>` per PRD §8.3.
//!
//! A `DrawBatch` is a contiguous run over the source command slice that the
//! GPU renderer can submit with one draw call (plus state changes). Batches
//! split when the shader pipeline changes, when an `Image` switches texture,
//! when a `Text` switches font, or when a clip-state command is encountered.

use std::ops::Range;

use crate::commands::{Command, FontHandle, TextureHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchKind {
    Rect,
    Border,
    Text { font: FontHandle },
    Image { texture: TextureHandle },
    Shadow,
    ClipPush,
    ClipPop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawBatch {
    pub kind: BatchKind,
    pub range: Range<usize>,
}

pub struct Batcher;

impl Batcher {
    pub fn batch(commands: &[Command]) -> Vec<DrawBatch> {
        let mut out = Vec::new();
        let mut cursor = 0usize;

        while cursor < commands.len() {
            let kind = kind_of(&commands[cursor]);

            // State-change commands are always single-command batches.
            if matches!(kind, BatchKind::ClipPush | BatchKind::ClipPop) {
                out.push(DrawBatch {
                    kind,
                    range: cursor..cursor + 1,
                });
                cursor += 1;
                continue;
            }

            // Extend the run while the next command produces the same kind.
            let start = cursor;
            cursor += 1;
            while cursor < commands.len() && kind_of(&commands[cursor]) == kind {
                cursor += 1;
            }
            out.push(DrawBatch {
                kind,
                range: start..cursor,
            });
        }

        out
    }
}

fn kind_of(cmd: &Command) -> BatchKind {
    match cmd {
        Command::Rect { .. } => BatchKind::Rect,
        Command::Border { .. } => BatchKind::Border,
        Command::Text { font, .. } => BatchKind::Text { font: *font },
        Command::Image { texture, .. } => BatchKind::Image { texture: *texture },
        Command::Shadow { .. } => BatchKind::Shadow,
        Command::Clip { .. } => BatchKind::ClipPush,
        Command::ClipPop => BatchKind::ClipPop,
    }
}
