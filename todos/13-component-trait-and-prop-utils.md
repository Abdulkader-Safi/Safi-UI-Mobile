# 13 — `Component` trait + `PropUtils`

**Phase:** 3 — Component Registry
**PRD refs:** §6.8, §6.14, §6.12 (binding rules)

## Goal

The contract every widget implements, plus the prop-parsing helpers that keep components ergonomic and typed-defaulted.

## Deliverables

- `safi-ui::component::Component` trait per §6.8 — **no `Send + Sync` bound**
- Lifecycle hooks: `on_mount`, `on_unmount`, `on_layout` (bounds-change-only)
- `safi-ui::props::PropUtils` helpers:
  - `get_str`, `parse_f32`, `parse_bool`, `parse_color`, `parse_dim`
  - `resolve_binding(prop, store)` — missing key returns `""`
  - `resolve_composite(template, store)` — `"Hello {{name}}!"` style; registers a subscription on every key referenced
  - Dynamic event binding: `onPress="{{dynamicAction}}"` resolves at gesture time
- Color parsing accepts: `#RRGGBB`, `#AARRGGBB`, `rgba(…)`, named colors, `"transparent"`
- Dimension parsing accepts: `"200"` (dp), `"50%"`, `"auto"`

## Dependencies

- `06`, `07`

## Acceptance

- Missing-key bindings never panic and never return `None`
- Composite-binding subscription fires on any referenced key change
- 100% coverage on `parse_color` / `parse_dim` edge cases
