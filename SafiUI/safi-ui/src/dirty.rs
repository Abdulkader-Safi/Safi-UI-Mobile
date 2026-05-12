//! Per-subtree dirty tracking (PRD §5.1 Pillar 2, §5.3, §6.4).
//!
//! `mark_dirty` and `invalidate_key` take an `arena: &WidgetArena` because the
//! §5.3 dirty-cascade rule (v2.2 addition) requires walking the parent chain.
//! This deviates from the illustrative §6.4 sample signatures — Pillar-2
//! semantics win when they conflict.
//!
//! `SizingMode` is stored in a tracker-owned side-table. Todo `10` (Taffy
//! integration) populates it after layout; until then everything defaults to
//! `Auto` (cascade-through), the conservative invalidation choice.

use std::collections::{HashMap, HashSet};

use crate::arena::{WidgetArena, WidgetId};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SizingMode {
    /// Fully resolved bounds (fixed `width`+`height`, or `flex` constrained
    /// by a sized parent). Cascade stops here.
    Resolved,
    /// Sizing depends on child content (`auto`, or unconstrained `flex`).
    /// Cascade propagates through this widget to its parent.
    #[default]
    Auto,
}

#[derive(Default)]
pub struct DirtyTracker {
    dirty: HashSet<WidgetId>,
    state_subs: HashMap<String, Vec<WidgetId>>,
    sizing: HashMap<WidgetId, SizingMode>,
}

impl DirtyTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_dirty(&mut self, arena: &WidgetArena, id: WidgetId) {
        if !arena.contains(id) {
            return;
        }
        self.dirty.insert(id);

        let mut cur = arena.parent_of(id);
        while let Some(p) = cur {
            if self.sizing(p) == SizingMode::Resolved {
                break;
            }
            if !self.dirty.insert(p) {
                break;
            }
            cur = arena.parent_of(p);
        }
    }

    pub fn is_dirty(&self, id: WidgetId) -> bool {
        self.dirty.contains(&id)
    }

    pub fn dirty_widgets(&self) -> impl Iterator<Item = WidgetId> + '_ {
        self.dirty.iter().copied()
    }

    pub fn needs_redraw(&self) -> bool {
        !self.dirty.is_empty()
    }

    pub fn on_frame_complete(&mut self) {
        self.dirty.clear();
    }

    pub fn subscribe(&mut self, key: &str, id: WidgetId) {
        self.state_subs.entry(key.to_string()).or_default().push(id);
    }

    pub fn unsubscribe_widget(&mut self, id: WidgetId) {
        for subs in self.state_subs.values_mut() {
            subs.retain(|&s| s != id);
        }
    }

    pub fn invalidate_key(&mut self, arena: &WidgetArena, key: &str) {
        let Some(subs) = self.state_subs.get(key) else {
            return;
        };
        let ids: Vec<WidgetId> = subs.clone();
        for id in ids {
            self.mark_dirty(arena, id);
        }
    }

    pub fn set_sizing(&mut self, id: WidgetId, mode: SizingMode) {
        self.sizing.insert(id, mode);
    }

    pub fn sizing(&self, id: WidgetId) -> SizingMode {
        self.sizing.get(&id).copied().unwrap_or_default()
    }
}
