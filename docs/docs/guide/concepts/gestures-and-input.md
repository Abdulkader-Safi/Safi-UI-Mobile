# Gestures and Input

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned gesture pipeline.
:::

A `GestureRecognizer` layer sits between SDL3 finger events and the component system. It translates raw touch events into semantic gestures before routing them to components via hit testing.

## Recognised gestures

| Gesture     | Trigger             | Threshold                | Used by                     |
| ----------- | ------------------- | ------------------------ | --------------------------- |
| `Tap`       | Finger down + up    | < 200ms, < 10dp movement | Button, Checkbox, `onPress` |
| `LongPress` | Finger held         | > 500ms                  | Tooltip, context menus      |
| `Pan`       | Continuous movement |                          | ScrollView drag             |
| `Swipe`     | Fast pan + release  | velocity threshold       | Drawer, BottomSheet dismiss |

```rust
pub enum Gesture {
    Tap       { pos: Vec2 },
    LongPress { pos: Vec2 },
    Pan       { delta: Vec2, velocity: Vec2 },
    Swipe     { direction: SwipeDirection, velocity: f32 },
}
```

## Hit testing

Gestures route to the topmost interactive component at the gesture's position. Hit testing walks the `WidgetArena` in **reverse Z order** (last-added widget first), respecting `Clip` regions from the command buffer.

A component's `hit_test()` defaults to a bounds check. Override it for non-rectangular interactive areas.

## Multi-touch

SDL3's `SDL_EVENT_FINGER_*` events carry a finger ID. Each finger has its own gesture state in the recognizer. Pinch and rotate gestures are not built-in in v1 (community can add them via custom recognizers).

## See also

- [Component trait](/api/core/component-trait)
- [`GestureRecognizer` API](/api/core/gesture-recognizer)
