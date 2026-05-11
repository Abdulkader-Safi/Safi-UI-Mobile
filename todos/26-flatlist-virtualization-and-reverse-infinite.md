# 26 — `FlatList` (windowed recycling + reverse-infinite-scroll)

**Phase:** 5 — State + Events
**PRD refs:** §10.6, §18 (decision #2)

## Goal

Windowed-recycling list that handles 1k+ items at 60fps, supports chat-style reverse-infinite-scroll, and preserves item state via the `key` prop across data reorders.

## Deliverables

- `FlatList` component per §10.6: `data`, `renderItem`, `keyExtractor`, `separator`
- Windowed recycling: only items in the viewport (+ small overscan) live in the arena
- Recycled items fire `on_unmount` when scrolled out and `on_mount` when recycled back in
- `key` prop drives state preservation across data reorders (separate from globally-unique `id`)
- Reverse-infinite-scroll mode (chat lists) — anchored at bottom, scroll loads older items
- Stable scroll position when prepending items
- Diagnostics: per-frame log of "items recycled" + "items mounted" in dev

## Dependencies

- `15`, `18`, `23`

## Acceptance

- 1k-item list scrolls at 60fps on Pixel 8 and iPhone 15
- Reordering the `data` array preserves item state by `key` (verified with stateful item components)
- Reverse-infinite-scroll chat demo with 500 messages prepends without jumping
