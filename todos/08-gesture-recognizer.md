# 08 — `GestureRecognizer`

**Phase:** 1 — Core Engine
**PRD refs:** §6.9, §5.3 (event flow)

## Goal

Translate raw SDL3 finger events into the four canonical `Gesture` variants and dispatch via hit testing.

## Deliverables

- `safi-ui::gestures::{Gesture, GestureRecognizer, SwipeDirection}`
- Finger lifecycle: `finger_down`, `finger_motion`, `finger_up`, `finger_cancel`
- Recognisers and thresholds per §6.9:
  - `Tap` — < 200ms duration, < 10dp movement
  - `LongPress` — held > 500ms with no movement
  - `Pan` — continuous movement, emits delta + velocity
  - `Swipe` — fast pan + release above velocity threshold
- `flush(arena, event_bus)` runs each frame: reverse-Z hit test on `WidgetArena`, dispatches to `Component::on_gesture`, stops on first `true` return
- Multi-touch: track per-finger state by `SDL_FingerID`

## Dependencies

- `04`, `07`

## Acceptance

- Tap vs long-press disambiguation tested at the boundary (199ms, 201ms, 499ms, 501ms)
- Pan velocity calc accurate within 5% on synthetic input
- Reverse-Z order respected (deepest hit wins)
