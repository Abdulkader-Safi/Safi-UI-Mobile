# 07 — `UIContext`, `ClipStack`, `FocusSystem`

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
