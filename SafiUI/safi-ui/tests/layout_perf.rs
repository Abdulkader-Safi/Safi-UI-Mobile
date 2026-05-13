//! Perf microtest for `LayoutEngine` — asserts the PRD §17.1 budget of
//! < 2ms for a 50-node tree on Pixel 8 / iPhone 15.
//!
//! Host machines are typically faster than Pixel 8, so the assertion floor
//! here is a generous 5ms — the goal is to catch *regressions* (a >2×
//! slowdown) on CI, not to certify the on-device number. Real device
//! measurement happens in todo `33` (`safi doctor` perf probe).

use std::time::Instant;

use safi_ui::layout::LayoutEngine;
use safi_ui::vnode::{LayoutRect, Props, VNode};
use taffy::{AvailableSpace, Size};

fn leaf(tag: &str, w: f32, h: f32) -> VNode {
    let mut props = Props::new();
    props.insert("width".to_string(), format!("{w}"));
    props.insert("height".to_string(), format!("{h}"));
    VNode {
        tag: tag.to_string(),
        props,
        children: Vec::new(),
        text_content: None,
        layout: LayoutRect::default(),
        id: None,
        key: None,
    }
}

fn row_of(n: usize, gap: f32) -> VNode {
    let mut props = Props::new();
    props.insert("flexDirection".to_string(), "row".to_string());
    props.insert("gap".to_string(), format!("{gap}"));
    props.insert("width".to_string(), "100%".to_string());
    props.insert("height".to_string(), "60".to_string());
    VNode {
        tag: "Row".to_string(),
        props,
        children: (0..n).map(|_| leaf("View", 40.0, 60.0)).collect(),
        text_content: None,
        layout: LayoutRect::default(),
        id: None,
        key: None,
    }
}

fn build_50_node_tree() -> VNode {
    // 1 root + 7 rows × (1 row container + 6 children) = 1 + 7*7 = 50 nodes.
    let mut root = Props::new();
    root.insert("width".to_string(), "100%".to_string());
    root.insert("height".to_string(), "100%".to_string());
    root.insert("gap".to_string(), "8".to_string());
    root.insert("padding".to_string(), "16".to_string());
    VNode {
        tag: "Column".to_string(),
        props: root,
        children: (0..7).map(|_| row_of(6, 4.0)).collect(),
        text_content: None,
        layout: LayoutRect::default(),
        id: None,
        key: None,
    }
}

fn count_nodes(v: &VNode) -> usize {
    1 + v.children.iter().map(count_nodes).sum::<usize>()
}

fn definite(w: f32, h: f32) -> Size<AvailableSpace> {
    Size {
        width: AvailableSpace::Definite(w),
        height: AvailableSpace::Definite(h),
    }
}

#[test]
fn fifty_node_tree_lays_out_under_five_ms_on_host() {
    let mut tree = build_50_node_tree();
    assert_eq!(
        count_nodes(&tree),
        50,
        "tree builder must yield exactly 50 nodes"
    );

    let mut le = LayoutEngine::new();
    // Warm up (allocate Taffy nodes, prime caches).
    le.compute(&mut tree, definite(480.0, 800.0));

    // Steady-state measurement: incremental recompute on an unchanged tree.
    let start = Instant::now();
    le.compute_if_dirty(&mut tree, definite(480.0, 800.0));
    let elapsed = start.elapsed();

    let budget_ms = if cfg!(debug_assertions) { 15.0 } else { 5.0 };
    let took_ms = elapsed.as_secs_f64() * 1000.0;
    assert!(
        took_ms < budget_ms,
        "50-node compute_if_dirty took {took_ms:.3}ms (budget: {budget_ms}ms); regression?",
    );
}

#[test]
fn cold_compute_under_twenty_ms_on_host() {
    // Cold path: no cache, all Taffy nodes allocated on this call.
    let mut tree = build_50_node_tree();
    let mut le = LayoutEngine::new();

    let start = Instant::now();
    le.compute(&mut tree, definite(480.0, 800.0));
    let elapsed = start.elapsed();

    let budget_ms = if cfg!(debug_assertions) { 50.0 } else { 20.0 };
    let took_ms = elapsed.as_secs_f64() * 1000.0;
    assert!(
        took_ms < budget_ms,
        "50-node cold compute took {took_ms:.3}ms (budget: {budget_ms}ms)",
    );
}
