//! Keyboard / tab focus tracking (PRD §6.2).
//!
//! Pure state — does not touch the dirty tracker. Focus moves with side
//! effects (mark prev + new dirty) live on [`crate::context::UIContext`].

use crate::arena::WidgetId;

#[derive(Default)]
pub struct FocusSystem {
    owner: Option<WidgetId>,
    tab_order: Vec<WidgetId>,
}

impl FocusSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn owner(&self) -> Option<WidgetId> {
        self.owner
    }

    /// Set focus owner. Returns the previous owner.
    pub fn set_owner(&mut self, new: Option<WidgetId>) -> Option<WidgetId> {
        std::mem::replace(&mut self.owner, new)
    }

    pub fn tab_order(&self) -> &[WidgetId] {
        &self.tab_order
    }

    pub fn register_tabbable(&mut self, id: WidgetId) {
        if !self.tab_order.contains(&id) {
            self.tab_order.push(id);
        }
    }

    pub fn unregister_tabbable(&mut self, id: WidgetId) {
        self.tab_order.retain(|&t| t != id);
    }

    /// Next tabbable after the current owner (wraps). `None` if list empty.
    pub fn next_in_tab_order(&self) -> Option<WidgetId> {
        if self.tab_order.is_empty() {
            return None;
        }
        let idx = match self.owner {
            Some(o) => self
                .tab_order
                .iter()
                .position(|&t| t == o)
                .map_or(0, |i| (i + 1) % self.tab_order.len()),
            None => 0,
        };
        Some(self.tab_order[idx])
    }

    pub fn prev_in_tab_order(&self) -> Option<WidgetId> {
        if self.tab_order.is_empty() {
            return None;
        }
        let n = self.tab_order.len();
        let idx = match self.owner {
            Some(o) => self
                .tab_order
                .iter()
                .position(|&t| t == o)
                .map_or(n - 1, |i| (i + n - 1) % n),
            None => n - 1,
        };
        Some(self.tab_order[idx])
    }
}
