# `Component` trait

:::tip Status: ✅ Shipped (todo 13)
Full PRD §6.8 surface: `bounds`, `build`, `hit_test`, `on_gesture`, and the three lifecycle hooks (`on_mount`, `on_unmount`, `on_layout`).
:::

Every widget — built-in or user-defined — implements `Component`. The trait deliberately carries **no `Send + Sync` bound** because component instances live on the main thread only; cross-thread coordination happens via `EventBus::post_async` and the image-decode channel.

## Definition

```rust
use safi_ui::component::Component;
use safi_ui::context::UIContext;
use safi_ui::gestures::Gesture;
use safi_ui::vnode::LayoutRect;
use glam::Vec2;

pub trait Component {
    fn bounds(&self) -> LayoutRect;

    fn build(&self, _ctx: &mut UIContext, _bounds: LayoutRect) {}

    fn hit_test(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }

    fn on_gesture(&mut self, _gesture: Gesture) -> bool { false }

    fn on_mount(&mut self, _ctx: &mut UIContext)   {}
    fn on_unmount(&mut self, _ctx: &mut UIContext) {}
    fn on_layout(&mut self, _bounds: LayoutRect)   {}
}
```

Every method except `bounds` has a default no-op impl, making the trait incrementally adoptable: a stateful widget that needs lifecycle wiring but not rendering can omit `build`; a static widget needs only `build` and `bounds`.

## Lifecycle semantics

| Hook         | Fires when                                                                                                                                                                                                       |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `build`      | Once per dirty subtree per frame (per-subtree dirty tracking, PRD §6.4). Main-thread only. Emit draw commands here.                                                                                              |
| `on_mount`   | Every time the component enters the live tree: first appearance, post-hot-reload remount of an unmatched node, recycle-back-into-window for a virtualised `FlatList` item.                                       |
| `on_unmount` | Every time the component leaves the live tree, including `FlatList` recycle-out. `visible="false"` does **not** trigger this — it skips `build` only and preserves instance state.                               |
| `on_layout`  | **Only when bounds change** vs the last laid-out frame. Quiet on static layouts. Use for positioning tooltips or measuring child intrinsic sizes without thrashing every frame.                                  |

See [Lifecycle](/guide/concepts/lifecycle) for the full table.

## Custom component example

```rust
use std::cell::Cell;
use safi_ui::component::Component;
use safi_ui::context::UIContext;
use safi_ui::vnode::LayoutRect;

pub struct Toggle {
    on: Cell<bool>,
    bounds: LayoutRect,
}

impl Component for Toggle {
    fn bounds(&self) -> LayoutRect { self.bounds }

    fn on_gesture(&mut self, _g: safi_ui::gestures::Gesture) -> bool {
        self.on.set(!self.on.get());
        true
    }

    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        // emit Rect/Border/Text commands based on self.on.get()
    }
}
```

The `Cell<bool>` would not compile if `Component` required `Send + Sync` — and that's intentional. Custom components on the main thread reach for `Cell` / `RefCell` / `Rc` freely.

## Registering a custom component

See [`register_component!`](/api/macros#register_component) (todo 14, in progress).

## See also

- [`PropUtils`](/api/core/prop-utils) — typed prop parsing for component constructors
- [`UIContext`](/api/core/ui-context) — passed to `build`, `on_mount`, `on_unmount`
- [`GestureRecognizer`](/api/core/gesture-recognizer) — dispatches gestures to `on_gesture`
- [PRD §6.8](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md)
