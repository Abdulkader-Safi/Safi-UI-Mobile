# Introduction

:::warning Status: Pre-Implementation
Safi-UI is a specification-stage project. The architecture, APIs, and component library described in these docs reflect the v1.0 design as captured in `PRD.md`. None of it is implemented yet. Use this documentation as the planning reference while the library is being built. See the [Implementation Status](/status) page for the current build state.
:::

**Safi-UI** is an open-source Rust library that brings a declarative, XML-driven component model to native mobile development. It targets Android and iOS using a workflow inspired by React Native, but compiles natively with **zero managed runtime overhead**.

Instead of writing low-level rendering code, developers author UI in structured XML files that are loaded at runtime, parsed into a virtual node tree, laid out via the [Taffy](https://github.com/DioxusLabs/taffy) flex engine, and rendered through a custom retained-mode paint system built on SDL3 and SDL_GPU.

## What makes it different

- **Native GPU on both platforms.** Vulkan on Android, Metal on iOS, both driven through SDL_GPU. No translation layers, no MoltenVK, no OpenGL ES fallback.
- **No managed runtime.** No Dart VM, no V8, no JVM. Pure Rust, statically compiled.
- **Retained-mode with per-subtree dirty tracking.** The GPU only runs when something actually changed, and only affected subtrees rebuild. Built for mobile battery life.
- **Arena-based widget storage.** Widgets reference each other by `WidgetId`, never by pointer. Solves Rust's borrow-checker UI problem cleanly and maps to the game-engine ECS pattern.
- **Hot-reload with seamless state preservation.** Edit XML, save, see your change in under 100ms with scroll positions, input cursors, and open dialogs preserved.

## Why not Flutter / React Native / Compose?

These frameworks introduce managed runtimes that increase binary size, startup latency, and memory overhead. They also prevent direct GPU access and make interop with C/C++ game engines painful.

## Why not ImGui or MicroUI directly?

Both are immediate-mode libraries designed for developer tooling. They redraw every frame regardless of whether anything changed (battery drain on mobile), have no flex layout system, treat touch as mouse emulation, use bitmap fonts unusable at mobile DPI, and have no component reuse model.

Safi-UI is architecturally inspired by MicroUI's command-list pattern — that core idea is retained, repurposed, and rebuilt for production mobile use in Rust.

## Who it's for

| Audience                     | Why                                                                             |
| ---------------------------- | ------------------------------------------------------------------------------- |
| **Rust mobile developers**   | Native mobile UI without adopting a managed runtime                             |
| **Game developers**          | Companion-app or in-game-overlay UI for custom C/Rust engines                   |
| **Embedded / IoT engineers** | Android targets where Flutter or React Native is too heavy                      |
| **Indie developers**         | Full control over the rendering pipeline and binary size                        |
| **Open source contributors** | Extend the component library, add platform backends, or write language bindings |

## Next

- [Getting Started](/guide/start/getting-started) — install the toolchain and create a project (planned)
- [Architecture](/guide/start/architecture) — the three pillars and the data flow
- [Implementation Status](/status) — what's built, what's WIP, what's planned
