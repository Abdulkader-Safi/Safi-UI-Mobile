# 10 — `LayoutEngine` (Taffy integration)

**Status:** ✅ Headless `LayoutEngine` + window-smoke layout demo (host SDL_Renderer + on-device log) · 22 layout tests + 50-node perf test green
**Phase:** 2 — Layout + Parse
**PRD refs:** §6.6

## Goal

Compute CSS Flexbox layout from a `VNode` tree into `LayoutRect`s on every dirty pass.

## Deliverables

- `safi-ui::layout::LayoutEngine` wrapping a `taffy::TaffyTree`
- Prop → Taffy mapping table from §6.6:
  - `flexDirection`, `justifyContent`, `alignItems`, `flex`, `width`/`height`, `padding`/`margin`, `gap`, `wrap`
- `compute(root: &mut VNode, available: Size)` walks the tree, syncs Taffy nodes, computes layout, writes back into each `VNode::layout`
- `compute_if_dirty` — only recomputes subtrees whose props changed
- Reuse Taffy node IDs across frames; only allocate on first compute
- DP coordinate system preserved at this layer (renderer converts to physical px later)

## Dependencies

- `03`, `06`

## Acceptance

- Reference layouts match Yoga / browser flexbox on 20+ test trees
- Re-laying out a single subtree skips siblings
- < 2ms layout time for 50-node tree (PRD §17.1)
