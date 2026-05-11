# 21 — Built-in components: Navigation

**Phase:** 4 — Component Library
**PRD refs:** §10.5

## Goal

Top-bar / tab-bar chrome and overlay containers. Note: a navigation _stack_ is deferred to v1.1 (§3.2).

## Deliverables

- `NavBar` — `title`, `leftAction`, `rightAction`, `bg`, `titleColor`; fixed top
- `TabBar` — `tabs`, `activeTab`, `onTabChange`, `bg`; bottom
- `Drawer` — `id` required; `open`, `onClose`, `side`, `width`; open/closed state preserved
- `Modal` — `id` required; `open`, `onClose`, `title`, `size`; centered, dims background
- `BottomSheet` — `id` required; `open`, `onClose`, `snapPoints`; pan-to-dismiss via `Pan` + `Swipe`

## Dependencies

- `15`, `18`

## Acceptance

- Drawer/Modal/BottomSheet state survives a `vnode!` re-render with stable `id`
- BottomSheet snap points work without an animation system (instant snap; animation deferred)
- All five components respect safe-area insets
