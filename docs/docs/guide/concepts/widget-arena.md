# Widget Arena

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned arena-storage model.
:::

`WidgetArena` is the flat storage for all widget instances. Widgets reference each other by **`WidgetId`** (a `u32` index), never by pointer. This is how Safi-UI sidesteps Rust's borrow-checker pain on UI trees.

## Why an arena

A naive UI tree in Rust quickly forces you into `Rc<RefCell<…>>` or unsafe interior mutability. Both are ergonomic dead ends. The arena pattern, common in game-engine ECS designs, gives:

- No reference cycles, no `Rc` overhead, no `RefCell` runtime panics
- Cache-friendly contiguous storage
- Cheap copy/compare for widget identity (`WidgetId` is a 32-bit integer)
- Easy serialization, debugging, and snapshotting

## Layout

```rust
pub type WidgetId = u32;

pub struct WidgetArena {
    widgets:     Vec<Box<dyn Component>>,
    taffy_nodes: Vec<taffy::NodeId>,
    bounds:      Vec<LayoutRect>,
    children:    Vec<Vec<WidgetId>>,
    parent:      Vec<Option<WidgetId>>,
}
```

Parallel `Vec`s indexed by `WidgetId`. Lookups are O(1).

## WidgetId stability

A `WidgetId` is stable for the lifetime of a widget instance. Hot-reload, FlatList recycling, and re-parenting all preserve `WidgetId` as long as the underlying instance is reused. Instances destroyed and re-created (e.g., a node removed and added back via XML edit) get fresh ids; state preservation across that boundary uses the stable [`id` prop](/guide/concepts/vnode-tree#identity-and-keying), not the `WidgetId`.

## See also

- [`WidgetArena` API](/api/core/widget-arena)
- [VNode Tree](/guide/concepts/vnode-tree)
