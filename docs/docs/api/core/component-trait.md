# `Component` Trait

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

The trait every widget implements. `build()` is **strictly main-thread**. The `Component` trait itself has **no `Send + Sync` bound** — only the factory closures stored in `ComponentRegistry` (which may be invoked from any thread during registration) require `Send + Sync`. This lets custom components hold `Rc<…>`, `RefCell<…>`, or other non-`Send` handles without ceremony.

```rust
pub trait Component {
    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect);

    fn hit_test(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }

    fn on_gesture(&mut self, gesture: Gesture) -> bool { false }

    fn on_mount(&mut self, _ctx: &mut UIContext)    {}
    fn on_unmount(&mut self, _ctx: &mut UIContext)  {}
    fn on_layout(&mut self, _bounds: LayoutRect)    {}

    fn bounds(&self) -> LayoutRect;
}
```

## Required methods

| Method   | Purpose                                           |
| -------- | ------------------------------------------------- |
| `build`  | Emit draw commands for the widget's current state |
| `bounds` | Return the widget's current layout bounds         |

## Default-impl methods (override as needed)

| Method       | Default                         | When to override                                         |
| ------------ | ------------------------------- | -------------------------------------------------------- |
| `hit_test`   | Bounds containment check        | Non-rectangular interactive areas                        |
| `on_gesture` | Returns `false` (don't consume) | Buttons, scrollables, anything tappable / swipeable      |
| `on_mount`   | Noop                            | Allocate caches, register observers                      |
| `on_unmount` | Noop                            | Release resources, deregister                            |
| `on_layout`  | Noop                            | Position dependent on actual bounds (tooltips, popovers) |

## Lifecycle semantics

- `on_mount` fires **whenever a component enters the live tree** (first appearance, hot-reload remount of an unmatched node, FlatList recycle-back-into-window)
- `on_unmount` fires whenever a component leaves the live tree, including FlatList recycle-out
- `visible="false"` skips `build()` only — does **not** fire `on_unmount`
- `on_layout` fires **only when bounds change** (delta vs the last laid-out frame)

See [Lifecycle](/guide/concepts/lifecycle) for the full table.

## Registering a custom component

See [`register_component!`](/api/macros#register_component).
