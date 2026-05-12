use std::collections::HashSet;

use safi_ui::arena::{InsertSpec, WidgetArena, WidgetId};
use safi_ui::component::Component;
use safi_ui::dirty::{DirtyTracker, SizingMode};
use safi_ui::vnode::LayoutRect;

struct TestWidget;
impl Component for TestWidget {
    fn bounds(&self) -> LayoutRect {
        LayoutRect::default()
    }
}

fn boxed() -> Box<dyn Component> {
    Box::new(TestWidget)
}

fn insert(a: &mut WidgetArena, parent: Option<WidgetId>) -> WidgetId {
    a.insert(InsertSpec {
        widget: boxed(),
        parent,
    })
}

/// Build `root → mid → leaf` with the supplied sizing modes on `root` and `mid`.
/// (Leaf sizing doesn't affect cascade from below; it defaults to `Auto`.)
fn tree_3deep(
    root_mode: SizingMode,
    mid_mode: SizingMode,
) -> (WidgetArena, DirtyTracker, WidgetId, WidgetId, WidgetId) {
    let mut a = WidgetArena::new();
    let root = insert(&mut a, None);
    let mid = insert(&mut a, Some(root));
    let leaf = insert(&mut a, Some(mid));
    let mut t = DirtyTracker::new();
    t.set_sizing(root, root_mode);
    t.set_sizing(mid, mid_mode);
    (a, t, root, mid, leaf)
}

fn dirty_set(t: &DirtyTracker) -> HashSet<WidgetId> {
    t.dirty_widgets().collect()
}

// ---- cascade rule (three scenarios the todo calls out) ----

#[test]
fn cascade_stops_at_fixed_parent() {
    let (a, mut t, root, mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Resolved);
    t.mark_dirty(&a, leaf);
    let d = dirty_set(&t);
    assert_eq!(d, HashSet::from([leaf]));
    assert!(!t.is_dirty(mid));
    assert!(!t.is_dirty(root));
}

#[test]
fn cascade_through_auto_parent() {
    let (a, mut t, root, mid, leaf) = tree_3deep(SizingMode::Auto, SizingMode::Auto);
    t.mark_dirty(&a, leaf);
    assert_eq!(dirty_set(&t), HashSet::from([leaf, mid, root]));
}

#[test]
fn cascade_stops_at_flex_constrained() {
    // "Flex inside a sized parent" == Resolved for the tracker.
    let (a, mut t, root, mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Resolved);
    t.mark_dirty(&a, leaf);
    assert_eq!(dirty_set(&t), HashSet::from([leaf]));
    let _ = (root, mid);
}

// ---- mixed cascade ----

#[test]
fn cascade_partial_through_auto_then_stops() {
    let (a, mut t, root, mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Auto);
    t.mark_dirty(&a, leaf);
    assert_eq!(dirty_set(&t), HashSet::from([leaf, mid]));
    assert!(!t.is_dirty(root));
}

#[test]
fn cascade_short_circuits_on_already_dirty_ancestor() {
    // root=Auto, mid=Auto → naive cascade would mark all three.
    // Pre-mark mid; cascade from leaf should stop at mid and NOT re-walk to root.
    let mut a = WidgetArena::new();
    let root = insert(&mut a, None);
    let mid = insert(&mut a, Some(root));
    let leaf = insert(&mut a, Some(mid));
    let mut t = DirtyTracker::new();
    t.set_sizing(root, SizingMode::Auto);
    t.set_sizing(mid, SizingMode::Auto);

    // Directly seed mid in the dirty set, without cascading.
    // We use the public API: mark_dirty(mid) would also cascade to root and
    // defeat the test. So mark a different widget that cascades up to mid only.
    // Easiest: temporarily set root to Resolved, mark mid, then flip root back.
    t.set_sizing(root, SizingMode::Resolved);
    t.mark_dirty(&a, mid);
    t.set_sizing(root, SizingMode::Auto);
    assert!(t.is_dirty(mid));
    assert!(!t.is_dirty(root));

    // Now mark leaf — cascade hits mid (already dirty) and stops.
    t.mark_dirty(&a, leaf);
    assert!(t.is_dirty(leaf));
    assert!(t.is_dirty(mid));
    assert!(
        !t.is_dirty(root),
        "cascade must short-circuit on already-dirty ancestor"
    );
}

// ---- dirty bit API ----

#[test]
fn mark_dirty_then_needs_redraw_returns_true() {
    let (a, mut t, _root, _mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Resolved);
    assert!(!t.needs_redraw());
    t.mark_dirty(&a, leaf);
    assert!(t.needs_redraw());
}

#[test]
fn on_frame_complete_clears_dirty_keeps_subs_and_sizing() {
    let (a, mut t, root, _mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Resolved);
    t.subscribe("user.name", leaf);
    t.set_sizing(root, SizingMode::Auto);
    t.mark_dirty(&a, leaf);

    t.on_frame_complete();

    assert!(!t.needs_redraw());
    assert_eq!(t.sizing(root), SizingMode::Auto, "sizing persists");

    // Subscriptions persist: invalidating the key still marks leaf.
    t.invalidate_key(&a, "user.name");
    assert!(t.is_dirty(leaf));
}

#[test]
fn is_dirty_for_unmarked_returns_false() {
    let t = DirtyTracker::new();
    assert!(!t.is_dirty(0));
    assert!(!t.is_dirty(999));
}

#[test]
fn mark_dirty_unknown_id_is_noop() {
    let a = WidgetArena::new();
    let mut t = DirtyTracker::new();
    t.mark_dirty(&a, 9999);
    assert!(!t.needs_redraw());
    assert_eq!(dirty_set(&t).len(), 0);
}

// ---- state subscriptions ----

#[test]
fn invalidate_key_marks_subscribers_dirty() {
    let (a, mut t, _root, _mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Resolved);
    t.subscribe("user.name", leaf);
    t.invalidate_key(&a, "user.name");
    assert!(t.is_dirty(leaf));
}

#[test]
fn invalidate_key_cascades_through_auto_parents() {
    let (a, mut t, root, mid, leaf) = tree_3deep(SizingMode::Auto, SizingMode::Auto);
    t.subscribe("user.name", leaf);
    t.invalidate_key(&a, "user.name");
    assert_eq!(dirty_set(&t), HashSet::from([leaf, mid, root]));
}

#[test]
fn invalidate_unknown_key_is_noop() {
    let (a, mut t, _root, _mid, _leaf) = tree_3deep(SizingMode::Auto, SizingMode::Auto);
    t.invalidate_key(&a, "no.such.key");
    assert!(!t.needs_redraw());
}

#[test]
fn unsubscribe_widget_removes_from_all_keys() {
    let (a, mut t, _root, _mid, leaf) = tree_3deep(SizingMode::Resolved, SizingMode::Resolved);
    t.subscribe("a", leaf);
    t.subscribe("b", leaf);

    t.unsubscribe_widget(leaf);

    t.invalidate_key(&a, "a");
    t.invalidate_key(&a, "b");
    assert!(!t.needs_redraw());
}

// ---- sizing side-table ----

#[test]
fn default_sizing_is_auto() {
    let t = DirtyTracker::new();
    assert_eq!(t.sizing(0), SizingMode::Auto);
}

#[test]
fn set_sizing_round_trip() {
    let mut t = DirtyTracker::new();
    t.set_sizing(7, SizingMode::Resolved);
    assert_eq!(t.sizing(7), SizingMode::Resolved);
    t.set_sizing(7, SizingMode::Auto);
    assert_eq!(t.sizing(7), SizingMode::Auto);
}

#[test]
fn sizing_for_unknown_id_returns_auto() {
    let t = DirtyTracker::new();
    assert_eq!(t.sizing(123_456), SizingMode::Auto);
}
