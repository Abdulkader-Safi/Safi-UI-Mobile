# 20 — Built-in components: Display

**Phase:** 4 — Component Library
**PRD refs:** §10.4

## Goal

Image, icon, and feedback components.

## Deliverables

- `Image` — `src`, `width`, `height`, `radius`, `fit` (`cover` / `contain` / `fill`); resolves from `assets/images/`
- `Avatar` — `src`, `size`, `fallback` (initials), `radius`
- `Icon` — `name`, `size`, `color`; icon atlas with name → UV mapping (icon set TBD in Phase 4 decision)
- `Badge` — `text`, `color`, `bg`, `size`
- `Divider` — `color`, `thickness`, `margin`
- `ProgressBar` — `value`, `max`, `color`, `height`
- `Spinner` — `size`, `color` (rotation handled without an animation system: rebuilds on a fixed-rate dirty pulse)
- `Tooltip` — `text`, `position`; wraps child, shows on long-press, uses `on_layout` for positioning

## Dependencies

- `15`, `17`

## Acceptance

- All seven components render correctly across DPI scales
- `Tooltip` repositions when its anchor's bounds change (verifies `on_layout` hook)
- Icon atlas decision documented in `docs/guide/concepts/`
