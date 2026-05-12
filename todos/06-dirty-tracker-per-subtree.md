# 06 — `DirtyTracker` (per-subtree, `WidgetId`-keyed)

**Status:** ✅ Completed. `DirtyTracker` in `safi-ui/src/dirty.rs` with `mark_dirty(arena, id)` implementing the §5.3 cascade rule (walks parents, stops at `Resolved` ancestors, short-circuits on already-dirty), key-subscription `subscribe`/`invalidate_key`/`unsubscribe_widget`, and a `SizingMode` side-table (`Resolved` / `Auto`, default `Auto`) that todo 10 will populate post-layout. Signature deviates from §6.4 sample (takes `&WidgetArena`) — §5.3 cascade requires it. 16 unit tests covering all three sizing scenarios + mixed cascade + short-circuit + subscriptions + frame lifecycle. fmt + clippy `-D warnings` clean.

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
