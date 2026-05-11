# `GestureRecognizer`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

Translates raw SDL3 finger events into semantic gestures.

```rust
pub enum Gesture {
    Tap       { pos: Vec2 },
    LongPress { pos: Vec2 },
    Pan       { delta: Vec2, velocity: Vec2 },
    Swipe     { direction: SwipeDirection, velocity: f32 },
}

pub struct GestureRecognizer {
    tap:        TapRecognizer,
    long_press: LongPressRecognizer,
    pan:        PanRecognizer,
    swipe:      SwipeRecognizer,
}
```

## Recognised gestures

| Gesture     | Trigger             | Threshold                |
| ----------- | ------------------- | ------------------------ |
| `Tap`       | Finger down + up    | < 200ms, < 10dp movement |
| `LongPress` | Finger held         | > 500ms                  |
| `Pan`       | Continuous movement |                          |
| `Swipe`     | Fast pan + release  | velocity threshold       |

## Sources

Recognizers operate on:

- `SDL_EVENT_FINGER_DOWN`
- `SDL_EVENT_FINGER_MOTION`
- `SDL_EVENT_FINGER_UP`

Multi-touch is fully supported; each finger has its own ID and gesture state.

## Routing

After a gesture is recognised, it is routed to the topmost interactive component at the gesture's position via `WidgetArena` reverse-Z hit testing. The component's `on_gesture(g) -> bool` returns `true` to consume the gesture.

## See also

- [Gestures and Input](/guide/concepts/gestures-and-input)
- [Component trait](/api/core/component-trait)
