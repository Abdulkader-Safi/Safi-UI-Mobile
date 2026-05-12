//! Flat `WidgetId`-indexed widget arena (PRD §5.1 Pillar 3, §6.2).
//!
//! Widgets reference each other by [`WidgetId`] (a `u32` index), never by
//! pointer. Removal uses tombstone slots — ids are stable for the lifetime
//! of the arena.

use crate::component::Component;
use crate::vnode::LayoutRect;

pub type WidgetId = u32;

pub struct InsertSpec {
    pub widget: Box<dyn Component>,
    pub parent: Option<WidgetId>,
}

struct Slot {
    widget: Option<Box<dyn Component>>,
    taffy_node: Option<taffy::NodeId>,
    bounds: LayoutRect,
    children: Vec<WidgetId>,
    parent: Option<WidgetId>,
}

impl Slot {
    fn tombstone() -> Self {
        Self {
            widget: None,
            taffy_node: None,
            bounds: LayoutRect::default(),
            children: Vec::new(),
            parent: None,
        }
    }
}

pub struct WidgetArena {
    slots: Vec<Slot>,
    roots: Vec<WidgetId>,
    live: usize,
}

impl Default for WidgetArena {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetArena {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            roots: Vec::new(),
            live: 0,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            slots: Vec::with_capacity(cap),
            roots: Vec::new(),
            live: 0,
        }
    }

    pub fn insert(&mut self, spec: InsertSpec) -> WidgetId {
        let parent = spec.parent.and_then(|p| {
            if self.is_alive(p) {
                Some(p)
            } else {
                debug_assert!(false, "WidgetArena::insert: parent {p} is not alive");
                eprintln!("safi-ui: WidgetArena::insert: parent {p} not alive; treating as root");
                None
            }
        });

        let id = u32::try_from(self.slots.len())
            .expect("WidgetArena: WidgetId exhausted (more than u32::MAX widgets)");
        self.slots.push(Slot {
            widget: Some(spec.widget),
            taffy_node: None,
            bounds: LayoutRect::default(),
            children: Vec::new(),
            parent,
        });
        self.live += 1;

        match parent {
            Some(p) => self.slots[p as usize].children.push(id),
            None => self.roots.push(id),
        }

        id
    }

    pub fn remove(&mut self, id: WidgetId) {
        if !self.is_alive(id) {
            return;
        }
        let children = std::mem::take(&mut self.slots[id as usize].children);
        for child in children {
            self.remove(child);
        }
        let parent = self.slots[id as usize].parent;
        match parent {
            Some(p) => {
                self.slots[p as usize].children.retain(|&c| c != id);
            }
            None => {
                self.roots.retain(|&r| r != id);
            }
        }
        self.slots[id as usize] = Slot::tombstone();
        self.live -= 1;
    }

    pub fn get(&self, id: WidgetId) -> Option<&dyn Component> {
        self.slot(id).and_then(|s| s.widget.as_deref())
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut (dyn Component + 'static)> {
        self.slot_mut(id).and_then(|s| s.widget.as_deref_mut())
    }

    pub fn contains(&self, id: WidgetId) -> bool {
        self.is_alive(id)
    }

    pub fn len(&self) -> usize {
        self.live
    }

    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    pub fn parent_of(&self, id: WidgetId) -> Option<WidgetId> {
        self.slot(id).and_then(|s| s.parent)
    }

    pub fn children_of(&self, id: WidgetId) -> &[WidgetId] {
        self.slot(id).map_or(&[], |s| &s.children)
    }

    pub fn roots(&self) -> &[WidgetId] {
        &self.roots
    }

    pub fn bounds(&self, id: WidgetId) -> Option<LayoutRect> {
        self.slot(id).map(|s| s.bounds)
    }

    pub fn set_bounds(&mut self, id: WidgetId, rect: LayoutRect) {
        if let Some(s) = self.slot_mut(id) {
            s.bounds = rect;
        }
    }

    pub fn taffy_node(&self, id: WidgetId) -> Option<taffy::NodeId> {
        self.slot(id).and_then(|s| s.taffy_node)
    }

    pub fn set_taffy_node(&mut self, id: WidgetId, node: taffy::NodeId) {
        if let Some(s) = self.slot_mut(id) {
            s.taffy_node = Some(node);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (WidgetId, &dyn Component)> + '_ {
        self.slots.iter().enumerate().filter_map(|(i, s)| {
            s.widget.as_deref().map(|w| {
                #[allow(clippy::cast_possible_truncation)]
                let id = i as WidgetId;
                (id, w)
            })
        })
    }

    pub fn iter_z_reverse(&self) -> impl Iterator<Item = WidgetId> + '_ {
        let mut order = Vec::with_capacity(self.live);
        for &root in self.roots.iter().rev() {
            self.walk_pre_order(root, &mut order);
        }
        order.reverse();
        order.into_iter()
    }

    fn walk_pre_order(&self, id: WidgetId, out: &mut Vec<WidgetId>) {
        out.push(id);
        for &child in &self.slots[id as usize].children {
            self.walk_pre_order(child, out);
        }
    }

    fn is_alive(&self, id: WidgetId) -> bool {
        self.slot(id).is_some_and(|s| s.widget.is_some())
    }

    fn slot(&self, id: WidgetId) -> Option<&Slot> {
        self.slots.get(id as usize).filter(|s| s.widget.is_some())
    }

    fn slot_mut(&mut self, id: WidgetId) -> Option<&mut Slot> {
        self.slots
            .get_mut(id as usize)
            .filter(|s| s.widget.is_some())
    }
}
