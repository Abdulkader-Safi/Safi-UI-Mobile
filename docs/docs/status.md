# Implementation Status

**Last updated:** May 2026 — todo `00` complete (Cargo workspace stood up). Source of truth: [`PRD.md`](https://github.com/AbdulKaderSafi/safi-ui/blob/main/PRD.md) v2.3.

:::info Phase 0 in progress
The Cargo workspace lives under `SafiUI/` at the repo root. `safi-ui` and `safi-ui-macros` are empty stub crates that `cargo check` / `cargo clippy -D warnings` / `cargo fmt --check` all pass cleanly. Modules will fill in across todos `01`…`32`.
:::

## Overall

| Phase   | Description        | Target   | Status                   |
| ------- | ------------------ | -------- | ------------------------ |
| Phase 0 | Foundations        | Wk 1–2   | In progress (todo 00 ✅) |
| Phase 1 | Core Engine        | Wk 3–6   | Not started              |
| Phase 2 | Layout + Parse     | Wk 7–9   | Not started              |
| Phase 3 | Component Registry | Wk 10–12 | Not started              |
| Phase 4 | Component Library  | Wk 13–18 | Not started              |
| Phase 5 | State + Events     | Wk 19–21 | Not started              |
| Phase 6 | Platform Polish    | Wk 22–24 | Not started              |
| Phase 7 | OSS Launch         | Wk 25–26 | Not started              |
| Post-v1 | CLI (`safi`)       | v1.1     | Not started              |

## Core systems

| System              | Spec                                        | Status |
| ------------------- | ------------------------------------------- | ------ |
| `VNode`             | [API](/api/core/vnode)                      | WIP    |
| `XmlParser`         | roxmltree-based                             | WIP    |
| `LayoutEngine`      | Taffy integration                           | WIP    |
| `WidgetArena`       | [API](/api/core/widget-arena)               | WIP    |
| `UIContext`         | [API](/api/core/ui-context)                 | WIP    |
| `CommandBuffer`     | [API](/api/core/command-buffer)             | WIP    |
| `DirtyTracker`      | [API](/api/core/dirty-tracker), per-subtree | WIP    |
| `StateStore`        | [API](/api/core/state-store)                | WIP    |
| `EventBus`          | [API](/api/core/event-bus)                  | WIP    |
| `PropUtils`         | [API](/api/core/prop-utils)                 | WIP    |
| `Component` trait   | [API](/api/core/component-trait)            | WIP    |
| `GestureRecognizer` | [API](/api/core/gesture-recognizer)         | WIP    |
| `GpuRenderer`       | SDL_GPU command submission + batching       | WIP    |
| `FontAtlas`         | fontdue + rustybuzz                         | WIP    |
| `ImageCache`        | LRU + channel-based decode signalling       | WIP    |
| `AssetLoader`       | AAssetManager + Bundle.main                 | WIP    |
| `HotReloadWatcher`  | inotify / kqueue, dev-only                  | WIP    |
| `vnode!` macro      | [API](/api/macros#vnode)                    | WIP    |

## Built-in components (target: 30+)

| Category   | Tags                                                                               | Status |
| ---------- | ---------------------------------------------------------------------------------- | ------ |
| Layout     | `Screen`, `View`, `Row`, `Column`, `Stack`, `ScrollView`, `SafeAreaView`, `Spacer` | WIP    |
| Typography | `Text`, `Heading`, `Label`, `Code`                                                 | WIP    |
| Input      | `Button`, `Input`, `TextArea`, `Checkbox`, `Switch`, `Select`, `Slider`            | WIP    |
| Display    | `Image`, `Avatar`, `Icon`, `Badge`, `Divider`, `ProgressBar`, `Spinner`, `Tooltip` | WIP    |
| Navigation | `NavBar`, `TabBar`, `Drawer`, `Modal`, `BottomSheet`                               | WIP    |
| Data       | `FlatList`, `Card`, `Table`, `EmptyState`                                          | WIP    |

## Platform support

| Platform | Backend                  | Min version             | Status                            |
| -------- | ------------------------ | ----------------------- | --------------------------------- |
| Android  | Vulkan via SDL_GPU       | API 24 (NDK r25+)       | WIP                               |
| iOS      | Metal via SDL_GPU        | iOS 16                  | WIP                               |
| Desktop  | SDL3 host (preview only) | macOS / Linux / Windows | Planned for `safi preview` (v1.1) |

## CLI commands (v1.1)

| Command            | Status         |
| ------------------ | -------------- |
| `safi new`         | WIP            |
| `safi generate`    | WIP            |
| `safi doctor`      | WIP            |
| `safi dev`         | WIP            |
| `safi build`       | WIP            |
| `safi lint`        | WIP            |
| **`safi preview`** | WIP (capstone) |

## Out of scope for v1

- Visual drag-and-drop editor (planned v2)
- Animation system (static layouts only in v1)
- CSS stylesheet files
- Accessibility / screen reader support (reserved props stubbed for v2)
- WebAssembly target
- Lua / Python scripting bindings
- Multi-window (iPad split-view, Android freeform)
- Built-in navigation stack (deferred to v1.1)
- C FFI custom component registration (Rust-only in v1)

## How to read these docs

Treat every page as a contract for the implementation. When code lands for a system, that system's WIP marker is removed and any deviations from the spec are documented inline.

If you find an inconsistency between a doc page and `PRD.md`, **`PRD.md` wins** until the implementation lands and supersedes both.
