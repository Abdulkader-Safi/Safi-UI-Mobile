use std::collections::HashSet;

use proptest::prelude::*;
use safi_ui::arena::{InsertSpec, WidgetArena, WidgetId};
use safi_ui::component::Component;
use safi_ui::vnode::LayoutRect;

struct TestWidget(LayoutRect);

impl Component for TestWidget {
    fn bounds(&self) -> LayoutRect {
        self.0
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> LayoutRect {
    LayoutRect {
        x,
        y,
        width: w,
        height: h,
    }
}

fn boxed(r: LayoutRect) -> Box<dyn Component> {
    Box::new(TestWidget(r))
}

fn insert_root(a: &mut WidgetArena, r: LayoutRect) -> WidgetId {
    a.insert(InsertSpec {
        widget: boxed(r),
        parent: None,
    })
}

fn insert_child(a: &mut WidgetArena, parent: WidgetId, r: LayoutRect) -> WidgetId {
    a.insert(InsertSpec {
        widget: boxed(r),
        parent: Some(parent),
    })
}

#[test]
fn insert_and_get_round_trip() {
    let mut a = WidgetArena::new();
    let id0 = insert_root(&mut a, rect(0.0, 0.0, 10.0, 10.0));
    let id1 = insert_root(&mut a, rect(1.0, 1.0, 1.0, 1.0));
    let id2 = insert_root(&mut a, rect(2.0, 2.0, 2.0, 2.0));

    assert_eq!(a.len(), 3);
    assert_eq!(
        a.get(id0).map(Component::bounds),
        Some(rect(0.0, 0.0, 10.0, 10.0))
    );
    assert_eq!(
        a.get(id1).map(Component::bounds),
        Some(rect(1.0, 1.0, 1.0, 1.0))
    );
    assert_eq!(
        a.get(id2).map(Component::bounds),
        Some(rect(2.0, 2.0, 2.0, 2.0))
    );
}

#[test]
fn parent_child_topology() {
    let mut a = WidgetArena::new();
    let p = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let c1 = insert_child(&mut a, p, rect(0.0, 0.0, 0.0, 0.0));
    let c2 = insert_child(&mut a, p, rect(0.0, 0.0, 0.0, 0.0));

    assert_eq!(a.parent_of(p), None);
    assert_eq!(a.parent_of(c1), Some(p));
    assert_eq!(a.parent_of(c2), Some(p));
    assert_eq!(a.children_of(p), &[c1, c2]);
    assert_eq!(a.children_of(c1), &[] as &[WidgetId]);
    assert_eq!(a.roots(), &[p]);
}

#[test]
fn remove_leaf_shrinks_parent_children() {
    let mut a = WidgetArena::new();
    let p = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let c1 = insert_child(&mut a, p, rect(0.0, 0.0, 0.0, 0.0));
    let c2 = insert_child(&mut a, p, rect(0.0, 0.0, 0.0, 0.0));

    a.remove(c1);

    assert!(!a.contains(c1));
    assert!(a.contains(p));
    assert!(a.contains(c2));
    assert_eq!(a.children_of(p), &[c2]);
    assert_eq!(a.len(), 2);
}

#[test]
fn remove_subtree_removes_descendants() {
    let mut a = WidgetArena::new();
    let root = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let mid = insert_child(&mut a, root, rect(0.0, 0.0, 0.0, 0.0));
    let leaf1 = insert_child(&mut a, mid, rect(0.0, 0.0, 0.0, 0.0));
    let leaf2 = insert_child(&mut a, mid, rect(0.0, 0.0, 0.0, 0.0));
    let sibling = insert_child(&mut a, root, rect(0.0, 0.0, 0.0, 0.0));

    a.remove(mid);

    assert!(!a.contains(mid));
    assert!(!a.contains(leaf1));
    assert!(!a.contains(leaf2));
    assert!(a.contains(root));
    assert!(a.contains(sibling));
    assert_eq!(a.children_of(root), &[sibling]);
    assert_eq!(a.len(), 2);
}

#[test]
fn remove_then_get_returns_none() {
    let mut a = WidgetArena::new();
    let id = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    a.remove(id);
    assert!(a.get(id).is_none());
    assert!(a.get_mut(id).is_none());
}

#[test]
fn remove_root_unregisters_root() {
    let mut a = WidgetArena::new();
    let r1 = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let r2 = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    a.remove(r1);
    assert_eq!(a.roots(), &[r2]);
}

#[test]
fn double_remove_is_noop() {
    let mut a = WidgetArena::new();
    let id = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    a.remove(id);
    a.remove(id);
    assert_eq!(a.len(), 0);
}

#[test]
fn iter_yields_only_live_widgets() {
    let mut a = WidgetArena::new();
    let a0 = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let _a1 = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let a2 = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    a.remove(a0);

    let live: Vec<WidgetId> = a.iter().map(|(id, _)| id).collect();
    assert_eq!(live.len(), a.len());
    assert!(!live.contains(&a0));
    assert!(live.contains(&a2));
}

#[test]
fn iter_z_reverse_order_matches_inverse_paint() {
    let mut a = WidgetArena::new();
    let ra = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    let rb = insert_child(&mut a, ra, rect(0.0, 0.0, 0.0, 0.0));
    let rc = insert_child(&mut a, ra, rect(0.0, 0.0, 0.0, 0.0));
    let rd = insert_child(&mut a, rb, rect(0.0, 0.0, 0.0, 0.0));
    let re = insert_child(&mut a, rb, rect(0.0, 0.0, 0.0, 0.0));

    let order: Vec<WidgetId> = a.iter_z_reverse().collect();
    assert_eq!(order, vec![rc, re, rd, rb, ra]);
}

#[test]
fn set_bounds_round_trip() {
    let mut a = WidgetArena::new();
    let id = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    assert_eq!(a.bounds(id), Some(LayoutRect::default()));
    a.set_bounds(id, rect(5.0, 6.0, 7.0, 8.0));
    assert_eq!(a.bounds(id), Some(rect(5.0, 6.0, 7.0, 8.0)));
}

#[test]
fn set_taffy_node_round_trip() {
    let mut a = WidgetArena::new();
    let id = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    assert_eq!(a.taffy_node(id), None);
    let node = taffy::NodeId::new(42);
    a.set_taffy_node(id, node);
    assert_eq!(a.taffy_node(id), Some(node));
}

#[test]
fn dead_id_apis_are_safe() {
    let mut a = WidgetArena::new();
    let id = insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    a.remove(id);

    assert!(a.get(id).is_none());
    assert_eq!(a.parent_of(id), None);
    assert_eq!(a.children_of(id), &[] as &[WidgetId]);
    assert_eq!(a.bounds(id), None);
    assert_eq!(a.taffy_node(id), None);

    let unknown: WidgetId = 9999;
    assert!(a.get(unknown).is_none());
    assert_eq!(a.children_of(unknown), &[] as &[WidgetId]);
}

#[test]
fn with_capacity_smoke() {
    let mut a = WidgetArena::with_capacity(8);
    for _ in 0..8 {
        insert_root(&mut a, rect(0.0, 0.0, 0.0, 0.0));
    }
    assert_eq!(a.len(), 8);
}

// ---- proptest invariants ----

#[derive(Debug, Clone)]
enum Op {
    Insert(Option<usize>),
    Remove(usize),
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        proptest::option::of(0usize..16).prop_map(Op::Insert),
        (0usize..16).prop_map(Op::Remove),
    ]
}

fn live_ids(a: &WidgetArena) -> Vec<WidgetId> {
    a.iter().map(|(id, _)| id).collect()
}

fn assert_invariants(a: &WidgetArena) {
    let live: HashSet<WidgetId> = live_ids(a).into_iter().collect();
    assert_eq!(live.len(), a.len(), "iter count must equal len()");

    let mut counted = 0usize;
    let mut seen: HashSet<WidgetId> = HashSet::new();

    let roots: HashSet<WidgetId> = a.roots().iter().copied().collect();
    assert_eq!(roots.len(), a.roots().len(), "no duplicate roots");

    for &id in &live {
        counted += 1;
        match a.parent_of(id) {
            Some(p) => {
                assert!(live.contains(&p), "parent of {id} is not live: {p}");
                assert!(
                    a.children_of(p).contains(&id),
                    "parent {p} missing child {id}"
                );
                assert!(!roots.contains(&id), "non-root in roots: {id}");
            }
            None => {
                assert!(roots.contains(&id), "live root {id} missing from roots()");
            }
        }
        for &c in a.children_of(id) {
            assert!(live.contains(&c), "child {c} of {id} is not live");
            assert_eq!(a.parent_of(c), Some(id));
            assert!(seen.insert(c), "child {c} appears in multiple lists");
        }
    }
    assert_eq!(counted, a.len());
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, max_shrink_iters: 256, .. ProptestConfig::default() })]

    #[test]
    fn arena_invariants_hold_under_random_ops(ops in proptest::collection::vec(op_strategy(), 0..32)) {
        let mut arena = WidgetArena::new();
        let mut known: Vec<WidgetId> = Vec::new();

        for op in ops {
            match op {
                Op::Insert(parent_idx) => {
                    let parent = parent_idx
                        .and_then(|i| known.get(i % known.len().max(1)).copied())
                        .filter(|&p| arena.contains(p));
                    let id = arena.insert(InsertSpec {
                        widget: boxed(LayoutRect::default()),
                        parent,
                    });
                    known.push(id);
                }
                Op::Remove(idx) => {
                    if known.is_empty() { continue; }
                    let id = known[idx % known.len()];
                    arena.remove(id);
                }
            }
            assert_invariants(&arena);
        }
    }
}
