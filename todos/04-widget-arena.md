# 04 — `WidgetArena`

**Status:** ✅ Completed. Flat tombstone-based arena in `safi-ui/src/arena.rs` with parallel topology vectors (parent / children / bounds / `Option<taffy::NodeId>`). Stub `Component` trait in `safi-ui/src/component.rs` (just `bounds()`; todo `13` extends). `iter_z_reverse` walks roots in reverse insertion order, reverse pre-order DFS. 13 unit tests + 1 proptest (256 random op sequences, invariants checked after every op) all pass; fmt + clippy `-D warnings` clean.

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
