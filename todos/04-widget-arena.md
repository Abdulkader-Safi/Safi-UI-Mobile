# 04 — `WidgetArena`

**Phase:** 1 — Core Engine
**PRD refs:** §5.1 (Pillar 3), §6.2

## Goal

Flat arena that owns every widget instance indexed by `WidgetId` (a `u32`), with parent/child topology stored in parallel vectors.

## Deliverables

- `safi-ui::arena::{WidgetId, WidgetArena}` per §6.2
- Parallel storage: `widgets`, `taffy_nodes`, `bounds`, `children`, `parent`
- API: `insert`, `remove`, `get`, `get_mut`, `parent_of`, `children_of`, `iter`, `iter_z_reverse` (for hit testing)
- Capacity hints + `with_capacity` constructor
- Property tests: insert / remove / re-insert stays consistent; no orphan children after removal

## Dependencies

- `03`

## Acceptance

- O(1) lookup by `WidgetId`
- Removing a parent removes all descendants (recursive)
- `iter_z_reverse` walks deepest-last for reverse Z hit testing
