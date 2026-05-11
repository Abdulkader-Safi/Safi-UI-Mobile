# 31 — Three example apps (hello / todo / dashboard)

**Phase:** 7 — OSS Launch
**PRD refs:** §16 (Phase 7), §17.2

## Goal

Three runnable example apps that double as documentation and the v1 adoption KPI (PRD §17.2 targets >=3 example apps shipping).

## Deliverables

- `examples/hello/` — minimal "Hello, world" screen + button; smallest possible Safi-UI app
- `examples/todo/` — local todo list using `StateStore` + `FlatList` + `Input`; demonstrates state + lists + forms
- `examples/dashboard/` — the Appendix XML example: NavBar, ScrollView, UserCard, stats Row, FlatList of projects, TabBar; demonstrates user-defined XML components and the full component library
- Each example builds for Android and iOS via the CI matrix
- Each example has its own `README.md` walking through the relevant concepts
- Cross-link example screens from the docs site

## Dependencies

- All of Phases 1–6 (`02`–`29`)

## Acceptance

- All three examples build green in CI and run on Pixel 8 and iPhone 15
- Dashboard example's `FlatList` handles 1k mock projects at 60fps
- README walkthroughs reference real file paths and survive a manual smoke test
