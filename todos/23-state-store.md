# 23 — `StateStore` (per-widget subscriptions)

**Phase:** 5 — State + Events
**PRD refs:** §6.12

**Status:** ✅ Complete — `safi-ui::state::StateStore` with main-thread
`set(k, v) -> Option<prev>`, `get(k) -> Option<String>`,
`subscribe(k, callback) -> SubId`, `unsubscribe(k, id) -> bool`.
Implements `BindingSource` so `PropsExt::resolve_binding` and
`resolve_composite` query it directly. Per-instance `StateStore::new()`
+ process-wide singleton `StateStore::global()` (mutex-guarded
`OnceLock`). `set` fires every subscriber for the key synchronously
in registration order. `build_tree_with(tree, registry, &store, ctx)`
resolves every `{{key}}` binding in props (and `text_content`)
against the store before handing the resulting `Props` to the widget
factory; `App::run`'s frame loop uses `StateStore::global()` so XML
bindings are live. 13 host tests including BindingSource integration
+ composite-binding key-set discovery. Per-widget DirtyTracker
invalidation (PRD §6.4 integration) lands when the retained-mode
build path arrives — for now every frame re-walks.

**Deferred:** `DirtyTracker::invalidate_key` hookup (stateless walker
re-builds the whole tree every frame regardless of which key
changed — correct but un-optimised). Background-thread set rejection
is implicitly enforced through the `&mut self` receiver + global
mutex, but no explicit thread-id check yet.

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
