# 18 — Built-in components: Layout + Typography

**Phase:** 4 — Component Library
**PRD refs:** §10.1, §10.2

## Goal

Round out the layout and typography component set. `View`, `Text`, `Button` are already in (todo `15`).

## Deliverables

**Layout (§10.1):**

- `Screen` — root container, fills viewport, handles safe-area insets
- `Row`, `Column`, `Stack` — flex containers and absolute-stack layering
- `ScrollView` — `id` required, scroll offset preserved across hot-reload, `horizontal`, `showsBar`
- `SafeAreaView` — `edges` prop, queries `PlatformBridge::safe_area`
- `Spacer` — `flex: 1` or fixed `size`

**Typography (§10.2):**

- `Heading` — `level` 1–6, pre-sized
- `Label` — small uppercase form label
- `Code` — monospace, `language`, `bg`

## Dependencies

- `15`, `16`

## Acceptance

- `ScrollView` retains its scroll position across a hot-reload (smoke test, full hot-reload in `29`)
- Each component has snapshot tests across DPI scales 1.0 / 2.625 / 3.0
