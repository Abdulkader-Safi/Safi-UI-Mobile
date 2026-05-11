# 06 — `DirtyTracker` (per-subtree, `WidgetId`-keyed)

**Phase:** 1 — Core Engine
**PRD refs:** §5.1 (Pillar 2), §5.3 (dirty cascade rule), §6.4

## Goal

Per-subtree dirty tracking from v1. Not a retrofit.

## Deliverables

- `safi-ui::dirty::DirtyTracker` per §6.4
- `mark_dirty(widget_id)` — flag one widget
- `subscribe(key, widget_id)` — wire state-key → widget mapping
- `invalidate_key(key)` — flips every subscribed widget dirty
- `needs_redraw`, `on_frame_complete`
- **Dirty cascade rule (§5.3):** auto-walk parent chain, flagging any ancestor whose layout depends on the marked widget (e.g. `auto`-sized parent of changed `Text`). Stop at ancestors with fully resolved bounds.
- Unit tests for the cascade rule covering: fixed `width`+`height` parent (stops), `auto` parent (cascades), `flex` parent in a sized container (stops)

## Dependencies

- `04`

## Acceptance

- Marking one widget dirty in a static screen rebuilds only that subtree
- Cascade tests pass for the three sizing scenarios
- `on_frame_complete` clears state
