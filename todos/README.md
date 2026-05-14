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

## Progress

| Todo                                | Status                                                                                                                           |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `00-repo-and-cargo-setup`           | ✅ Completed                                                                                                                     |
| `01-ci-pipeline`                    | ✅ Completed                                                                                                                     |
| `02-sdl3-window-on-android-and-ios` | ✅ Android (Pixel 8 emu) + ✅ iOS (iPhone 17 Pro Max) + sim via SDL_Renderer                                                     |
| `03-vnode-and-vnode-macro`          | ✅ Completed                                                                                                                     |
| `04-widget-arena`                   | ✅ Completed                                                                                                                     |
| `05-command-buffer`                 | ✅ Completed                                                                                                                     |
| `06-dirty-tracker-per-subtree`      | ✅ Completed                                                                                                                     |
| `07-ui-context`                     | ✅ Completed                                                                                                                     |
| `08-gesture-recognizer`             | ✅ Completed                                                                                                                     |
| `09-gpu-renderer-rect-and-text`     | ✅ Batcher + scaffolding · ⚠️ Device demo pending                                                                                |
| `10-taffy-layout-engine`            | ✅ `LayoutEngine` + window-smoke layout demo (canvas fallback paints; GPU path logs)                                             |
| `11-xml-parser`                     | ✅ `parse::{XmlParser, ParseError}` (roxmltree); host-tested; `vnode!` ≡ XML output                                              |
| `12-asset-loader-and-dpi`           | ✅ Trait + host + Android (`AAssetManager` via JNI) + iOS (`NSBundle.mainBundle` via objc2) + `DpiScale` + orientation re-layout |
| `13-component-trait-and-prop-utils` | ✅ Component trait (build + lifecycle hooks) + `PropUtils` (Color, Dimension, bindings) — 33 new host tests                       |
| `14-component-registry`             | ✅ `ComponentRegistry` + `register_component!` macro + `DebugBox` fallback — 7 new host tests                                     |
| `15-base-widgets-view-text-button`  | ✅ `View` (+Screen/Row/Column/Stack/Spacer) + `Text` (+Heading/Label) + `Button` (4 variants) + `register_builtins` — 18 tests   |
| `16-font-atlas-fontdue-rustybuzz`   | ✅ `FontAtlas` (fontdue) + `TextShaper` (rustybuzz) + `App::with_font_bytes` + per-pixel SDL_Renderer blit — 9 tests              |
| `17-image-pipeline-channel-based`   | ⏳ Not started                                                                                                                   |
| `18`–`23`                           | ⏳ Not started                                                                                                                   |
| `24-event-bus-main-thread-and-post-async` | ✅ `EventBus` (on/emit/off + post_async + drain_async) + App::run hit-test dispatch — 10 tests                              |
| `25`–`34`                           | ⏳ Not started                                                                                                                   |

Each completed todo gets a `**Status:**` line near the top of its file and a row flipped above.
