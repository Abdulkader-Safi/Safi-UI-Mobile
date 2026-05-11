# `WidgetArena`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

Flat storage for all widget instances. Widgets reference each other by `WidgetId`, never by pointer.

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

## Why parallel `Vec`s

Cache-friendly. The render walk reads `bounds` and `children` together; the dirty cascade reads `parent`. Parallel storage means each pass touches contiguous memory.

## `WidgetId` lifecycle

A `WidgetId` is stable for the lifetime of the underlying instance. Hot-reload, FlatList recycling, and re-parenting all preserve `WidgetId` as long as the underlying instance is reused. Instances destroyed and re-created get fresh ids.

State preservation across instance boundaries (e.g., hot-reload across an XML edit that removed and re-added a node) uses the stable [`id` prop](/guide/concepts/vnode-tree#identity-and-keying), not `WidgetId`.

## See also

- [Widget Arena concept](/guide/concepts/widget-arena)
