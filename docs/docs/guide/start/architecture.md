# Architecture

:::warning Status: Specification (v1.0)
This page describes the planned v1.0 architecture as captured in `PRD.md`. None of it is implemented yet.
:::

## The three pillars

Safi-UI is built on three architectural pillars. Every other design decision flows from these.

### Pillar 1 — Command list pattern (from MicroUI)

All rendering is expressed as a flat list of typed commands emitted during the build phase and consumed by the SDL_GPU renderer. **No component ever calls GPU APIs directly.** This decouples UI logic from rendering completely and enables efficient GPU batching.

### Pillar 2 — Retained mode with per-subtree dirty tracking

The UI does not redraw every frame. A per-subtree dirty flag system tracks exactly which nodes have changed, keyed by `WidgetId`. The GPU is only invoked when something actually needs repainting, and only affected subtrees are rebuilt. This is a v1 requirement, not a v1.1 retrofit.

### Pillar 3 — Arena-based widget storage

All widgets live in a flat arena indexed by `WidgetId`. Widgets reference each other by ID, never by pointer. This solves Rust's borrow-checker challenges with UI trees (no `Rc<RefCell<>>`, no cycles) and maps naturally to the game-engine ECS pattern.

## System layers

| Layer       | Module                      | Responsibility                                         |
| ----------- | --------------------------- | ------------------------------------------------------ |
| 1. Source   | XML Files (`assets/ui/`)    | UI authored as `.xml` files, loaded at runtime         |
| 2. Parse    | `XmlParser` (roxmltree)     | Parses XML into a `VNode` tree                         |
| 3. Resolve  | `ComponentRegistry`         | Maps XML tag names to Rust component factories         |
| 4. Layout   | `LayoutEngine` (Taffy)      | Computes CSS Flexbox layout for every node             |
| 5. Build    | `UIContext` + `WidgetArena` | Walks tree, calls `Component::build()`, emits commands |
| 6. Render   | `GpuRenderer` (SDL_GPU)     | Batches and submits command list to Vulkan / Metal     |
| 7. Platform | `PlatformBridge`            | Safe area, keyboard height, DPI, lifecycle via SDL3    |

## Data flow

```
XML File
  └─► XmlParser::parse()              →  VNode tree (id + key fields)
        └─► LayoutEngine::compute()       →  VNode tree + LayoutRect (Taffy)
              └─► DirtyTracker::check()       (per-subtree, WidgetId-keyed)
                    └─► [if dirty] UIContext::build()
                          └─► ComponentRegistry::resolve(tag)
                                └─► Component::build(ctx, props, bounds)
                                      └─► CommandBuffer::push(command)
                                            └─► GpuRenderer::flush()
                                                  ├─► Vulkan (Android)
                                                  └─► Metal (iOS)

SDL3 Event Loop
  └─► SDL_FINGER_* events
        └─► GestureRecognizer
              └─► HitTest (reverse Z walk on WidgetArena)
                    └─► Component::on_gesture()
                          └─► EventBus / StateStore update (main thread only)
                                └─► DirtyTracker::mark_dirty(widget_id)

Background threads
  └─► Image decode (thread pool)
        └─► channel → main thread → SDL_GPU upload → mark_dirty
```

## Dirty cascade rule

`DirtyTracker::mark_dirty(widget_id)` flags a single widget. The `DirtyTracker` automatically walks the parent chain in `WidgetArena` and also flags any ancestor whose **layout sizing depends on the marked widget** (e.g., an `auto`-sized parent of a `Text` whose content changed). Ancestors with fully resolved bounds (fixed `width` + `height`, or `flex` constrained by a sized parent) are not cascaded.

## Component resolution order

When the parser encounters a tag, the registry resolves it in this order:

1. Check `ComponentRegistry` for a Rust-registered component
2. Check `XmlTemplateLoader` for a user-defined `.xml` component file
3. Fall back to `DebugBox` — renders a red outlined rectangle with the unknown tag name (in dev builds)

## Threading model

- `Component::build()` is strictly **main-thread**
- `StateStore::set()` and `EventBus::emit()` are **main-thread-only**
- `EventBus::post_async()` is the safe cross-thread posting path
- Background image decode signals the main thread via a **channel**

See [State and Events](/guide/concepts/state-and-events) for the recommended pattern when updating state from a background thread.
