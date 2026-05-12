//! Per-frame UI context (PRD §6.2).
//!
//! Threaded by `&mut` into every `Component::build()` call (todo 13+).
//! Aggregates the four core engine pieces (commands, dirty, focus, clips)
//! plus the device-context scalars (`dpi_scale`, `safe_area`).

use crate::arena::{WidgetArena, WidgetId};
use crate::clip::ClipStack;
use crate::commands::{Command, CommandBuffer, Rect, COMMAND_BUFFER_CAPACITY_DEFAULT};
use crate::dirty::DirtyTracker;
use crate::edge_insets::EdgeInsets;
use crate::focus::FocusSystem;

pub struct UIContext {
    pub commands: CommandBuffer,
    pub dirty: DirtyTracker,
    pub focus: FocusSystem,
    pub clips: ClipStack,
    pub dpi_scale: f32,
    pub safe_area: EdgeInsets,
}

impl UIContext {
    pub fn new(command_buffer_capacity: usize, dpi_scale: f32, safe_area: EdgeInsets) -> Self {
        Self {
            commands: CommandBuffer::with_capacity(command_buffer_capacity),
            dirty: DirtyTracker::new(),
            focus: FocusSystem::new(),
            clips: ClipStack::new(),
            dpi_scale,
            safe_area,
        }
    }

    /// Convenience constructor for tests: 8192-cap buffer, dpi=1.0, no safe area.
    pub fn test_default() -> Self {
        Self::new(COMMAND_BUFFER_CAPACITY_DEFAULT, 1.0, EdgeInsets::ZERO)
    }

    pub fn push_clip(&mut self, rect: Rect) {
        self.clips.push(rect);
        self.commands.push(Command::Clip { rect });
    }

    pub fn pop_clip(&mut self) {
        if self.clips.pop().is_some() {
            self.commands.push(Command::ClipPop);
        } else {
            debug_assert!(false, "UIContext::pop_clip with empty clip stack");
            eprintln!("safi-ui: UIContext::pop_clip with empty stack");
        }
    }

    /// Move focus to `new` (or clear it with `None`). Marks both the previous
    /// owner and the new owner dirty so the renderer redraws focus rings.
    pub fn request_focus(&mut self, arena: &WidgetArena, new: Option<WidgetId>) {
        let prev = self.focus.set_owner(new);
        if let Some(p) = prev {
            self.dirty.mark_dirty(arena, p);
        }
        if let Some(n) = new {
            self.dirty.mark_dirty(arena, n);
        }
    }

    pub fn clear_focus(&mut self, arena: &WidgetArena) {
        self.request_focus(arena, None);
    }
}
