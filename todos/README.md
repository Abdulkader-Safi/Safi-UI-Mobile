# Safi-UI Build Todos

Priority-ordered build plan derived from `PRD.md` v2.3. Files are numbered `NN-name.md` where `NN` is the build order.

## Phase mapping

| Files     | Phase   | Description                                                             |
| --------- | ------- | ----------------------------------------------------------------------- |
| `00`–`03` | Phase 0 | Foundations (repo, CI, SDL3, vnode! macro)                              |
| `04`–`09` | Phase 1 | Core Engine (arena, command buffer, dirty tracker, gestures, GPU)       |
| `10`–`13` | Phase 2 | Layout + Parse (Taffy, XML, asset loader, DPI)                          |
| `14`–`16` | Phase 3 | Component Registry (registry, PropUtils, Component trait, base widgets) |
| `17`–`22` | Phase 4 | Component Library (built-ins, fonts, images)                            |
| `23`–`26` | Phase 5 | State + Events (StateStore, EventBus, FlatList, XML templates)          |
| `27`–`30` | Phase 6 | Platform Polish (safe area, hot-reload, panic isolation)                |
| `31`–`32` | Phase 7 | OSS Launch (docs, examples, release)                                    |
| `33`–`34` | Post-v1 | CLI tool (`safi`) with `safi preview` capstone                          |

## Conventions per todo file

- **Goal** — one-line outcome
- **PRD refs** — exact section pointers
- **Deliverables** — concrete artifacts (modules, types, tests)
- **Dependencies** — earlier todos that must land first
- **Acceptance** — measurable done criteria

## How to work this list

Work sequentially unless a todo is explicitly marked parallelisable. Don't skip the Phase 1 acceptance demo (tap-to-flip button on both platforms) — every later phase assumes it works.
