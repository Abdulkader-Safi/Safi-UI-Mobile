# 24 — `EventBus` (main-thread + `post_async`)

**Phase:** 5 — State + Events
**PRD refs:** §6.11, §8.4, §6.12 (background-thread state pattern)

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
