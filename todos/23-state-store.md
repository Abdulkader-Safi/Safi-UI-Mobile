# 23 — `StateStore` (per-widget subscriptions)

**Phase:** 5 — State + Events
**PRD refs:** §6.12

## Goal

Reactive key-value store driving `{{binding}}` props with per-widget invalidation.

## Deliverables

- `safi-ui::state::StateStore` — main-thread-only `set` / `get`
- `subscribe(key, callback)` for app-level listeners
- `set` calls `DirtyTracker::invalidate_key`, so only subscribed widgets rebuild
- Per-instance (`StateStore::new`) for tests + global singleton (`StateStore::global`) for apps
- Binding rules per §6.12:
  - Missing key → empty string (never an error)
  - Bindings allowed in any prop, including `width`, `src`, `onPress`
  - Composite bindings (`"Hello {{name}}!"`) — every key registers a subscription
  - Dynamic event bindings (`onPress="{{action}}"`) supported

## Dependencies

- `06`, `13`, `14`

## Acceptance

- Changing a single key invalidates only its subscribed widgets
- 1000 keys × 100 widgets: `set` completes in < 0.5ms
- Cross-thread set from a worker thread is **rejected** (forces use of `EventBus::post_async`, see `24`)
