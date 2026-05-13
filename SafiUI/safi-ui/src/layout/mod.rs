//! `LayoutEngine` — Taffy-backed CSS Flexbox compute over a [`VNode`] tree
//! (PRD §6.6, todo `10`).
//!
//! Walks a [`VNode`] tree, syncs each node into a [`taffy::TaffyTree`],
//! computes layout, and writes the resulting [`LayoutRect`]s back into
//! `vnode.layout`. Taffy node ids are cached by a stable [`NodeKey`] so
//! subsequent frames reuse the same nodes — `compute_if_dirty` skips restyling
//! subtrees whose layout-relevant props are unchanged.
//!
//! Coordinates stay in **dp** at this layer. Converting to physical pixels is
//! the renderer's job (PRD §7.3).

mod style;

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use taffy::{AvailableSpace, Size, TaffyTree};

use crate::dirty::SizingMode;
use crate::vnode::{LayoutRect, VNode};

use self::style::LAYOUT_PROP_KEYS;

/// Stable identity for a [`VNode`] across frames. Built from `vnode.id`
/// when present, otherwise from the `(parent_key, sibling_index, tag)`
/// chain.
///
/// Implemented as a `u64` hash so it's `Copy`, cheap to compare, and
/// uniform for the cache map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeKey(u64);

impl NodeKey {
    fn from_id(id: &str) -> Self {
        let mut h = DefaultHasher::new();
        b"safi-ui::layout::id\0".hash(&mut h);
        id.hash(&mut h);
        Self(h.finish())
    }

    fn from_path(parent: Self, sibling_idx: usize, tag: &str) -> Self {
        let mut h = DefaultHasher::new();
        parent.0.hash(&mut h);
        sibling_idx.hash(&mut h);
        tag.hash(&mut h);
        Self(h.finish())
    }
}

const ROOT_PARENT: NodeKey = NodeKey(0xD15C_0FFE_0BAD_F00D);

#[derive(Debug, Clone, Copy)]
struct CacheEntry {
    node: taffy::NodeId,
    prop_hash: u64,
    sizing: SizingMode,
}

/// Wraps a [`taffy::TaffyTree`] and a [`NodeKey`] → [`taffy::NodeId`] cache so
/// Taffy node ids survive across frames (PRD §6.6: "Reuse Taffy node IDs
/// across frames; only allocate on first compute").
pub struct LayoutEngine {
    tree: TaffyTree<()>,
    cache: HashMap<NodeKey, CacheEntry>,
    live: HashSet<NodeKey>,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
            cache: HashMap::new(),
            live: HashSet::new(),
        }
    }

    /// Compute layout for `root`, writing computed bounds into each
    /// `VNode::layout`. Available space is the viewport (typically the
    /// window size in dp).
    ///
    /// Forces a full restyle on every node — use [`Self::compute_if_dirty`]
    /// in the steady-state frame loop.
    pub fn compute(&mut self, root: &mut VNode, available: Size<AvailableSpace>) {
        self.compute_inner(root, available, /* respect_dirty = */ false);
    }

    /// Like [`Self::compute`] but only re-styles subtrees whose
    /// layout-relevant props (see [`LAYOUT_PROP_KEYS`]) changed since the
    /// last call. Siblings of an unchanged subtree are not re-styled.
    pub fn compute_if_dirty(&mut self, root: &mut VNode, available: Size<AvailableSpace>) {
        self.compute_inner(root, available, /* respect_dirty = */ true);
    }

    /// Sizing classification of the most recently laid-out node. Used by a
    /// caller wiring [`crate::dirty::DirtyTracker::set_sizing`] once
    /// `VNode`s are bound to widget ids (todo `13`).
    pub fn sizing_of(&self, key: NodeKey) -> SizingMode {
        self.cache
            .get(&key)
            .map_or(SizingMode::default(), |e| e.sizing)
    }

    /// Look up the [`NodeKey`] assigned to the root of `root`. Stable across
    /// frames as long as `root.id` (or its tag, when id-less) is stable.
    pub fn root_key(root: &VNode) -> NodeKey {
        Self::key_for(ROOT_PARENT, 0, root)
    }

    /// Iterator over every `NodeKey` currently in the cache. Useful for tests
    /// asserting node-reuse behaviour.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    fn compute_inner(
        &mut self,
        root: &mut VNode,
        available: Size<AvailableSpace>,
        respect_dirty: bool,
    ) {
        self.live.clear();
        let root_key = Self::key_for(ROOT_PARENT, 0, root);
        let root_node = self.sync_node(root_key, root, respect_dirty);

        // Layout in dp space — caller decided the available bounds.
        let _ = self.tree.compute_layout(root_node, available);

        // Walk back, writing computed bounds into each VNode.
        Self::write_back(&self.tree, &self.cache, ROOT_PARENT, 0, root, 0.0, 0.0);

        // Evict cache entries for nodes that disappeared this frame.
        self.evict_dead();
    }

    /// Walk one subtree post-order, building (or refreshing) Taffy nodes and
    /// returning the Taffy id of this node.
    fn sync_node(&mut self, key: NodeKey, vnode: &VNode, respect_dirty: bool) -> taffy::NodeId {
        self.live.insert(key);

        // Build children first so we can call set_children with the full list.
        let mut child_nodes = Vec::with_capacity(vnode.children.len());
        for (idx, child) in vnode.children.iter().enumerate() {
            let child_key = Self::key_for(key, idx, child);
            let child_id = self.sync_node(child_key, child, respect_dirty);
            child_nodes.push(child_id);
        }

        let prop_hash = hash_layout_props(&vnode.tag, &vnode.props);
        let style = style::style_from_tag_and_props(&vnode.tag, &vnode.props);
        let sizing = if style::is_fully_sized(&style) {
            SizingMode::Resolved
        } else {
            SizingMode::Auto
        };

        if let Some(entry) = self.cache.get(&key).copied() {
            let style_dirty = !respect_dirty || entry.prop_hash != prop_hash;
            if style_dirty {
                let _ = self.tree.set_style(entry.node, style);
            }
            // Topology may have changed even when props didn't.
            let _ = self.tree.set_children(entry.node, &child_nodes);
            self.cache.insert(
                key,
                CacheEntry {
                    node: entry.node,
                    prop_hash,
                    sizing,
                },
            );
            entry.node
        } else {
            // First sight of this key — allocate a fresh Taffy node.
            let id = self
                .tree
                .new_with_children(style, &child_nodes)
                .expect("taffy: new_with_children failed (out of memory?)");
            self.cache.insert(
                key,
                CacheEntry {
                    node: id,
                    prop_hash,
                    sizing,
                },
            );
            id
        }
    }

    fn write_back(
        tree: &TaffyTree<()>,
        cache: &HashMap<NodeKey, CacheEntry>,
        parent_key: NodeKey,
        sibling_idx: usize,
        vnode: &mut VNode,
        offset_x: f32,
        offset_y: f32,
    ) {
        let key = Self::key_for(parent_key, sibling_idx, vnode);
        let Some(entry) = cache.get(&key) else {
            return;
        };
        let Ok(layout) = tree.layout(entry.node) else {
            return;
        };
        let abs_x = offset_x + layout.location.x;
        let abs_y = offset_y + layout.location.y;
        vnode.layout = LayoutRect {
            x: abs_x,
            y: abs_y,
            width: layout.size.width,
            height: layout.size.height,
        };
        for (idx, child) in vnode.children.iter_mut().enumerate() {
            Self::write_back(tree, cache, key, idx, child, abs_x, abs_y);
        }
    }

    fn evict_dead(&mut self) {
        // Collect first to avoid holding a borrow during removal.
        let dead: Vec<NodeKey> = self
            .cache
            .keys()
            .copied()
            .filter(|k| !self.live.contains(k))
            .collect();
        for k in dead {
            if let Some(entry) = self.cache.remove(&k) {
                let _ = self.tree.remove(entry.node);
            }
        }
    }

    fn key_for(parent: NodeKey, sibling_idx: usize, vnode: &VNode) -> NodeKey {
        if let Some(id) = vnode.id.as_deref().filter(|s| !s.is_empty()) {
            NodeKey::from_id(id)
        } else {
            NodeKey::from_path(parent, sibling_idx, &vnode.tag)
        }
    }
}

fn hash_layout_props(tag: &str, props: &crate::vnode::Props) -> u64 {
    let mut h = DefaultHasher::new();
    // Tag participates because layout::style derives a default
    // flex_direction from it (see style::style_from_tag_and_props).
    tag.hash(&mut h);
    for key in LAYOUT_PROP_KEYS {
        if let Some(v) = props.get(*key) {
            (*key).hash(&mut h);
            v.hash(&mut h);
        }
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vnode::{Props, VNode};

    fn node(tag: &str, props: &[(&str, &str)], children: Vec<VNode>) -> VNode {
        let mut p = Props::new();
        for (k, v) in props {
            p.insert((*k).to_string(), (*v).to_string());
        }
        VNode {
            tag: tag.to_string(),
            props: p,
            children,
            text_content: None,
            layout: LayoutRect::default(),
            id: None,
            key: None,
        }
    }

    fn node_id(tag: &str, id: &str, props: &[(&str, &str)], children: Vec<VNode>) -> VNode {
        let mut v = node(tag, props, children);
        v.id = Some(id.to_string());
        v
    }

    fn definite(w: f32, h: f32) -> Size<AvailableSpace> {
        Size {
            width: AvailableSpace::Definite(w),
            height: AvailableSpace::Definite(h),
        }
    }

    #[test]
    fn root_takes_full_available_space_when_unbounded() {
        let mut root = node("Screen", &[("width", "100%"), ("height", "100%")], vec![]);
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        assert!((root.layout.width - 480.0).abs() < 0.5);
        assert!((root.layout.height - 800.0).abs() < 0.5);
    }

    #[test]
    fn column_with_gap_stacks_children() {
        let mut root = node(
            "Column",
            &[("width", "200"), ("height", "300"), ("gap", "10")],
            vec![
                node("View", &[("height", "40")], vec![]),
                node("View", &[("height", "40")], vec![]),
            ],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        // Children stack vertically with a 10dp gap.
        assert!((root.children[0].layout.y - 0.0).abs() < 0.5);
        assert!((root.children[1].layout.y - 50.0).abs() < 0.5);
    }

    #[test]
    fn row_flex_distributes() {
        let mut root = node(
            "Row",
            &[
                ("flexDirection", "row"),
                ("width", "300"),
                ("height", "100"),
                ("gap", "0"),
            ],
            vec![
                node("View", &[("flex", "1"), ("height", "100")], vec![]),
                node("View", &[("flex", "1"), ("height", "100")], vec![]),
                node("View", &[("flex", "1"), ("height", "100")], vec![]),
            ],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        for c in &root.children {
            assert!((c.layout.width - 100.0).abs() < 0.5);
        }
    }

    #[test]
    fn padding_inset_applies() {
        let mut root = node(
            "View",
            &[("width", "100"), ("height", "100"), ("padding", "10")],
            vec![node("View", &[("flex", "1"), ("height", "20")], vec![])],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        assert!((root.children[0].layout.x - 10.0).abs() < 0.5);
        assert!((root.children[0].layout.y - 10.0).abs() < 0.5);
    }

    #[test]
    fn percent_width_resolves_against_parent() {
        let mut root = node(
            "Row",
            &[("flexDirection", "row"), ("width", "200"), ("height", "50")],
            vec![node("View", &[("width", "50%"), ("height", "50")], vec![])],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        assert!((root.children[0].layout.width - 100.0).abs() < 0.5);
    }

    #[test]
    fn nested_three_deep_offsets_correctly() {
        let mut root = node(
            "Column",
            &[("padding", "10"), ("width", "200"), ("height", "200")],
            vec![node(
                "Column",
                &[("padding", "5"), ("flex", "1"), ("width", "100%")],
                vec![node("View", &[("width", "20"), ("height", "20")], vec![])],
            )],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        // 10 (outer pad) + 5 (inner pad) = 15.
        assert!((root.children[0].children[0].layout.x - 15.0).abs() < 0.5);
        assert!((root.children[0].children[0].layout.y - 15.0).abs() < 0.5);
    }

    #[test]
    fn justify_space_between_pushes_to_edges() {
        let mut root = node(
            "Row",
            &[
                ("flexDirection", "row"),
                ("justifyContent", "spaceBetween"),
                ("width", "300"),
                ("height", "50"),
            ],
            vec![
                node("View", &[("width", "50"), ("height", "50")], vec![]),
                node("View", &[("width", "50"), ("height", "50")], vec![]),
            ],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        assert!((root.children[0].layout.x - 0.0).abs() < 0.5);
        assert!((root.children[1].layout.x - 250.0).abs() < 0.5);
    }

    #[test]
    fn align_center_centers_cross_axis() {
        let mut root = node(
            "Row",
            &[
                ("flexDirection", "row"),
                ("alignItems", "center"),
                ("width", "200"),
                ("height", "100"),
            ],
            vec![node("View", &[("width", "50"), ("height", "40")], vec![])],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        // (100 - 40) / 2 = 30.
        assert!((root.children[0].layout.y - 30.0).abs() < 0.5);
    }

    #[test]
    fn margin_per_side_offsets_child() {
        let mut root = node(
            "View",
            &[("width", "200"), ("height", "200")],
            vec![node(
                "View",
                &[("width", "50"), ("height", "50"), ("marginLeft", "20")],
                vec![],
            )],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        assert!((root.children[0].layout.x - 20.0).abs() < 0.5);
    }

    #[test]
    fn id_keeps_stable_taffy_id_across_frames() {
        let mut tree = node_id(
            "Column",
            "root",
            &[("width", "200"), ("height", "200")],
            vec![node_id("View", "child", &[("height", "40")], vec![])],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut tree, definite(480.0, 800.0));
        let root_key = LayoutEngine::root_key(&tree);
        let child_key = NodeKey::from_id("child");
        let root_taffy_before = le.cache.get(&root_key).unwrap().node;
        let child_taffy_before = le.cache.get(&child_key).unwrap().node;

        // Mutate a layout-irrelevant prop on the child; recompute.
        tree.children[0]
            .props
            .insert("bg".to_string(), "#fff".to_string());
        le.compute_if_dirty(&mut tree, definite(480.0, 800.0));

        let root_taffy_after = le.cache.get(&root_key).unwrap().node;
        let child_taffy_after = le.cache.get(&child_key).unwrap().node;
        assert_eq!(root_taffy_before, root_taffy_after);
        assert_eq!(child_taffy_before, child_taffy_after);
    }

    #[test]
    fn evicting_removed_child_frees_cache_slot() {
        let mut tree = node(
            "Column",
            &[("width", "200"), ("height", "200")],
            vec![
                node("View", &[("height", "40")], vec![]),
                node("View", &[("height", "40")], vec![]),
            ],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut tree, definite(480.0, 800.0));
        let before = le.cache_size();
        assert!(before >= 3);

        tree.children.pop();
        le.compute_if_dirty(&mut tree, definite(480.0, 800.0));
        // Dropped child should be evicted from the cache.
        assert!(le.cache_size() < before);
    }

    #[test]
    fn sizing_classification_is_resolved_when_both_dims_set() {
        let mut tree = node_id("View", "n1", &[("width", "100"), ("height", "50")], vec![]);
        let mut le = LayoutEngine::new();
        le.compute(&mut tree, definite(480.0, 800.0));
        let key = NodeKey::from_id("n1");
        assert_eq!(le.sizing_of(key), SizingMode::Resolved);
    }

    #[test]
    fn sizing_classification_is_auto_when_height_missing() {
        let mut tree = node_id("View", "n2", &[("width", "100")], vec![]);
        let mut le = LayoutEngine::new();
        le.compute(&mut tree, definite(480.0, 800.0));
        let key = NodeKey::from_id("n2");
        assert_eq!(le.sizing_of(key), SizingMode::Auto);
    }

    #[test]
    fn flex_wrap_breaks_to_next_line() {
        let mut root = node(
            "Row",
            &[
                ("flexDirection", "row"),
                ("wrap", "wrap"),
                ("width", "100"),
                ("height", "200"),
            ],
            vec![
                node("View", &[("width", "60"), ("height", "40")], vec![]),
                node("View", &[("width", "60"), ("height", "40")], vec![]),
            ],
        );
        let mut le = LayoutEngine::new();
        le.compute(&mut root, definite(480.0, 800.0));
        // Second child should wrap below the first (different y).
        assert!(root.children[1].layout.y > root.children[0].layout.y);
    }

    #[test]
    fn dp_units_with_suffix_parse() {
        let mut tree = node("View", &[("width", "100dp"), ("height", "50dp")], vec![]);
        let mut le = LayoutEngine::new();
        le.compute(&mut tree, definite(480.0, 800.0));
        assert!((tree.layout.width - 100.0).abs() < 0.5);
        assert!((tree.layout.height - 50.0).abs() < 0.5);
    }
}
