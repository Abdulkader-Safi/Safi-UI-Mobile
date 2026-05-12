# 07 — `UIContext`, `ClipStack`, `FocusSystem`

**Status:** ✅ Completed. `UIContext` in `safi-ui/src/context.rs` aggregates `CommandBuffer` / `DirtyTracker` / `FocusSystem` / `ClipStack` + `dpi_scale` + `safe_area`. `push_clip` / `pop_clip` own the `Command::Clip` / `ClipPop` pairing (debug_assert + eprintln on mismatch). `request_focus` / `clear_focus` take `&WidgetArena` and mark prev + new dirty. New modules: `clip.rs`, `focus.rs`, `edge_insets.rs`. 16 tests pass (incl. catch_unwind debug-panic assertion); fmt + clippy `-D warnings` clean.

**Phase:** 1 — Core Engine
**PRD refs:** §6.2

## Goal

Central per-frame state object passed by `&mut` into every `Component::build`.

## Deliverables

- `safi-ui::context::UIContext` owning `CommandBuffer`, `DirtyTracker`, `FocusSystem`, `ClipStack`, `dpi_scale`, `safe_area`
- `ClipStack` push/pop helpers that emit matching `Clip` / `ClipPop` commands
- `FocusSystem` — focus owner (`Option<WidgetId>`), `request_focus`, `clear_focus`, tab order list
- `EdgeInsets` struct for safe area
- Builder for tests: `UIContext::test_default()`

## Dependencies

- `04`, `05`, `06`

## Acceptance

- Mismatched `push_clip` / `pop_clip` panics in dev (debug_assert) and logs in release
- Focus changes mark the previous + new focus owner dirty
