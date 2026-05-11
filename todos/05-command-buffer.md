# 05 — `CommandBuffer`

**Phase:** 1 — Core Engine
**PRD refs:** §5.1 (Pillar 1), §6.3

## Goal

The growable list of typed draw commands that decouples component logic from GPU calls.

## Deliverables

- `safi-ui::commands::{Command, CommandBuffer}`
- All variants per §6.3: `Rect`, `Border`, `Text`, `Image`, `Shadow`, `Clip`, `ClipPop`
- `CommandBuffer::new_with_capacity(cap)` configurable at `App::init` (default `8192`)
- Grow-on-overflow with a single `warn!` log per frame (never drop, never panic)
- Debug-mode `warn!` once per frame when utilisation crosses 75%
- `push`, `clear`, `len`, `as_slice`, range tracking for per-subtree dirty redraws

## Dependencies

- `00`

## Acceptance

- Pushing past initial capacity grows + logs a single warning (regression test)
- 75% threshold warning fires exactly once per frame (regression test)
- Zero-allocation `as_slice` view for the renderer
