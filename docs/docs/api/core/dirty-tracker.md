# `DirtyTracker`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

Per-subtree dirty bits keyed by `WidgetId`, plus a key → subscriber index used by `StateStore` to invalidate just the subtrees that depend on a changed key.

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

## Methods

| Method                         | Notes                                                                              |
| ------------------------------ | ---------------------------------------------------------------------------------- |
| `mark_dirty(id)`               | Flag a single widget dirty; cascade rule walks ancestors with dependent layout    |
| `needs_redraw()`               | True if any widget is dirty; the main loop uses this to skip frames                |
| `on_frame_complete()`          | Clear all dirty bits at end of frame                                               |
| `subscribe(key, id)`           | Called from `PropUtils` when resolving a binding; registers a subscription         |
| `invalidate_key(key)`          | Called from `StateStore::set`; marks all subscribed widgets dirty                  |

## Composite binding subscriptions

A binding like `"Hello {{name}}!"` registers subscriptions on **every** key referenced in the template. Any one of them changing invalidates the owning subtree.

## Dirty cascade

`mark_dirty` automatically walks the parent chain in `WidgetArena` and also flags any ancestor whose **layout sizing depends** on the marked widget. Ancestors with fully resolved bounds are not cascaded.

## See also

- [Dirty Tracking concept](/guide/concepts/dirty-tracking)
- [`StateStore`](/api/core/state-store)
