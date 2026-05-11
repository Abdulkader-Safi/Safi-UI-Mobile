# 29 — Hot-reload with seamless state preservation

**Phase:** 6 — Platform Polish
**PRD refs:** §6.10, §14.2 (parse-error overlay)

## Goal

File-watcher → re-parse → tree-diff → state-preserving remount, all under 100ms with no visible flash. Dev-only.

## Deliverables

- `safi-ui::dev::HotReloadWatcher` behind the `dev` feature flag
- File watcher: `inotify` on Android (via `notify` crate or direct), `kqueue` on iOS host targets, `notify` for desktop preview hosts
- Watch scope: `assets/ui/` recursive
- On change:
  1. Re-parse affected file(s)
  2. Tree-diff against the running tree, matching stateful nodes by stable `id` prop
  3. Map old `WidgetId` → new `WidgetId`, preserving: scroll offsets, input cursor positions, open/closed sheets/drawers/modals, focus, in-flight gesture state
  4. Re-register event subscriptions
  5. `DirtyTracker::mark_dirty` on every affected subtree
- Unmatched stateful nodes fire `on_mount` as new instances
- Parse-error overlay per §14.2: full-screen red overlay listing **all** errors across all reloaded files; dismissible by tap; previous valid UI visible behind; opt-out hook for custom dev UI
- Release builds: `HotReloadWatcher` compiled out entirely

## Dependencies

- `11`, `12`, `25`, `28`

## Acceptance

- File save → new frame in < 100ms on a modest dev machine
- `ScrollView` and `Input` cursor state survive a hot-reload with stable `id`
- Editing an XML file with a syntax error shows the dismissible overlay; fixing the error restores normal UI
- Release build has zero `HotReloadWatcher` symbols (verified with `nm`)
