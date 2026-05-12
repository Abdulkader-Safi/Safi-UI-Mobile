//! Clip-region stack (PRD §6.2).
//!
//! Pure depth/bounds tracker. `Command::Clip` / `Command::ClipPop` emission is
//! the responsibility of [`crate::context::UIContext::push_clip`] /
//! [`crate::context::UIContext::pop_clip`] so the pairing has a single owner.

use crate::commands::Rect;

#[derive(Default)]
pub struct ClipStack {
    stack: Vec<Rect>,
}

impl ClipStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, rect: Rect) {
        self.stack.push(rect);
    }

    pub fn pop(&mut self) -> Option<Rect> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<Rect> {
        self.stack.last().copied()
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}
