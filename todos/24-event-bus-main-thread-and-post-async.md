# 24 — `EventBus` (main-thread + `post_async`)

**Phase:** 5 — State + Events
**PRD refs:** §6.11, §8.4, §6.12 (background-thread state pattern)

**Status:** ✅ Complete — `safi-ui::events::EventBus` with main-thread
`on(name, handler) -> HandlerId`, `emit(name)`, `off(name, id)`,
plus the cross-thread `post_async(name)` MPSC ingress and
frame-loop `drain_async() -> usize` helper. Process-wide singleton
via `EventBus::global()` (lazy `OnceLock<Mutex<EventBus>>`); tests
use `EventBus::new()` for isolation. `App::run` now hit-tests
`SDL_EVENT_FINGER_UP` against the VNode tree, finds the deepest
`onPress`-carrying node, and emits via the global bus. Workers
post via `bus.async_sender().send(name)` and the bus drains every
frame in FIFO order. 10 host tests cover the contract end-to-end
including 8-thread concurrent post.

**Deferred:** dependency declared on todo 23 (StateStore) in the
original spec is loose — the EventBus has no functional coupling
to StateStore. Background-thread state-update pattern (PRD §6.12)
will be documented when StateStore lands.

## Goal

Named publish/subscribe bus for the main thread with a safe cross-thread ingress.

## Deliverables

- `safi-ui::events::EventBus` per §6.11
- Main-thread API: `on(name, handler)`, `emit(name)`, `off(name, handler_id)`
- Cross-thread API: `post_async(name)` — pushes onto an MPSC queue
- Frame-loop helper: `drain_async()` drains queued events synchronously on the main thread
- Dot-notation event names by convention (`auth.login`, `nav.back`)
- Per-instance + global singleton, mirroring `StateStore`
- Documented pattern in `docs/` for the background-thread state-update flow (§6.12 example)

## Dependencies

- `23`

## Acceptance

- `post_async` from 8 worker threads dispatches all events in order on the next frame
- `emit` from a worker thread panics in dev / logs in release (forces correct usage)
- Handler unregistration works (no leaks across hot-reload)
