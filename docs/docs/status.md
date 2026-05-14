# Implementation Status

**Last updated:** May 2026 — todos `00`–`12` complete. Source of truth: [`PRD.md`](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md) v2.3.

:::tip Phase 2 complete — AssetLoader + DPI shipped end-to-end
Todo 12 lands the full `AssetLoader` stack: real `AndroidAssetLoader` (`AAssetManager` via NDK + JNI) and real `IosAssetLoader` (`NSBundle.mainBundle` via objc2), plus host `FilesystemAssetLoader`, `MockAssetLoader`, and `DpiScale` resolved from `SDL_GetDisplayContentScale` at `App::run` startup. Orientation changes trigger re-layout. Next up: todo 13 (`Component` trait lifecycle hooks + `PropUtils`).
:::

## Overall

| Phase   | Description        | Target   | Status                                           |
| ------- | ------------------ | -------- | ------------------------------------------------ |
| Phase 0 | Foundations        | Wk 1–2   | ✅ Complete (00–03)                              |
| Phase 1 | Core Engine        | Wk 3–6   | ✅ Mostly complete (04–09; device demo deferred) |
| Phase 2 | Layout + Parse     | Wk 7–9   | ✅ Complete (10 ✅ 11 ✅ 12 ✅)                  |
| Phase 3 | Component Registry | Wk 10–12 | Not started                                      |
| Phase 4 | Component Library  | Wk 13–18 | Not started                                      |
| Phase 5 | State + Events     | Wk 19–21 | Not started                                      |
| Phase 6 | Platform Polish    | Wk 22–24 | Not started                                      |
| Phase 7 | OSS Launch         | Wk 25–26 | Not started                                      |
| Post-v1 | CLI (`safi`)       | v1.1     | Not started                                      |

## Core systems

| System              | Spec                                        | Status                                          |
| ------------------- | ------------------------------------------- | ----------------------------------------------- |
| `VNode`             | [API](/api/core/vnode)                      | ✅ Shipped                                      |
| `XmlParser`         | roxmltree-based                             | ✅ Shipped (todo 11)                            |
| `LayoutEngine`      | Taffy integration                           | ✅ Shipped (todo 10)                            |
| `WidgetArena`       | [API](/api/core/widget-arena)               | ✅ Shipped                                      |
| `UIContext`         | [API](/api/core/ui-context)                 | ✅ Shipped                                      |
| `CommandBuffer`     | [API](/api/core/command-buffer)             | ✅ Shipped                                      |
| `DirtyTracker`      | [API](/api/core/dirty-tracker), per-subtree | ✅ Shipped                                      |
| `StateStore`        | [API](/api/core/state-store)                | WIP                                             |
| `EventBus`          | [API](/api/core/event-bus)                  | WIP                                             |
| `PropUtils`         | [API](/api/core/prop-utils)                 | WIP (todo 13)                                   |
| `Component` trait   | [API](/api/core/component-trait)            | Partial (lifecycle hooks in todo 13)            |
| `GestureRecognizer` | [API](/api/core/gesture-recognizer)         | ✅ Shipped                                      |
| `GpuRenderer`       | SDL_GPU command submission + batching       | Partial (batcher ✅; device demo pending)       |
| `FontAtlas`         | fontdue + rustybuzz                         | WIP                                             |
| `ImageCache`        | LRU + channel-based decode signalling       | WIP                                             |
| `AssetLoader`       | [API](/api/core/asset-loader)               | ✅ Shipped (host + Android + iOS, todo 12)      |
| `DpiScale`          | [API](/api/core/asset-loader#dpi-scaling)   | ✅ Shipped (todo 12)                            |
| `HotReloadWatcher`  | inotify / kqueue, dev-only                  | WIP                                             |
| `vnode!` macro      | [API](/api/macros#vnode)                    | ✅ Shipped                                      |

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

| Platform | Backend                  | Min version             | Status                                      |
| -------- | ------------------------ | ----------------------- | ------------------------------------------- |
| Android  | Vulkan via SDL_GPU       | API 24 (NDK r25+)       | ✅ Verified on Pixel 8 emulator (Vulkan)    |
| iOS      | Metal via SDL_GPU        | iOS 16                  | Smoke test code landed ⚠️ needs real iPhone |
| Desktop  | SDL3 host (preview only) | macOS / Linux / Windows | Planned for `safi preview` (v1.1)           |

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
