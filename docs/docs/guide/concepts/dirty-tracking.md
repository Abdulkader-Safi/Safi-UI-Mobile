# Dirty Tracking

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned per-subtree dirty model.
:::

The `DirtyTracker` is **per-subtree from v1**, keyed by `WidgetId`. It is the mechanism that makes Safi-UI viable on battery-powered devices.

## What it does

- Tracks which widget subtrees are stale and need re-`build()`
- Tracks which `StateStore` keys each widget subscribed to
- Cascades dirty up the parent chain when layout sizing depends on the changed widget

## API sketch

```rust
pub struct DirtyTracker {
    dirty_widgets: HashSet<WidgetId>,
    state_subs:    HashMap<String, Vec<WidgetId>>,
}

impl DirtyTracker {
    pub fn mark_dirty(&mut self, id: WidgetId);
    pub fn needs_redraw(&self) -> bool;
    pub fn on_frame_complete(&mut self);
    pub fn subscribe(&mut self, key: &str, id: WidgetId);
    pub fn invalidate_key(&mut self, key: &str);
}
```

## How invalidation propagates

1. App or framework code calls `StateStore::set("user.name", "Safi")`
2. `StateStore` calls `dirty.invalidate_key("user.name")`
3. `DirtyTracker` looks up subscribed `WidgetId`s for that key
4. Each subscribed widget is marked dirty
5. The dirty cascade rule walks each widget's parent chain and also marks ancestors whose layout depends on the changed widget (e.g., an `auto`-sized parent of a `Text` whose content changed)
6. On the next frame, only marked subtrees re-`build()`
7. Affected command-buffer ranges are rebuilt; the GPU draws

## Cascade rule

Ancestors with fully resolved bounds (fixed `width` + `height`, or `flex` constrained by a sized parent) are **not** cascaded. Only ancestors whose intrinsic sizing depends on the changed child get marked.

## Composite bindings

A binding like `"Hello {{name}}!"` registers subscriptions on **every** key referenced in the template. Any one of them changing invalidates the owning subtree.

## Static screens

On a fully static screen (no state changes, no incoming events), the main loop skips both `build()` and `GpuRenderer::flush()` and yields the CPU. Frame CPU time target: **< 0.1ms**.

## See also

- [`DirtyTracker` API](/api/core/dirty-tracker)
- [State and Events](/guide/concepts/state-and-events)
