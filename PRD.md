# Safi-UI, Product Requirements Document

**v2.2 | Safi Studio**

> **v2.2 change note:** Spec ambiguities flagged in the v2.1 verification pass are resolved: lifecycle rules clarified (§6.8), `Component` no longer requires `Send + Sync`, release-mode panic behaviour pinned down (§14.1), `vnode!` macro specified (§6.15), `on_layout` change-detection rule added, dirty-cascade rule added (§5.3), composite-binding subscription semantics added (§6.12), background-thread state-update pattern documented, EventBus thread-safety wording tightened, frame-loop snippet made compileable.
>
> v2.1 resolved all 15 open questions from the v2.0 review. Section 18 still summarises every decision.

---

| Field        | Value                     |
| ------------ | ------------------------- |
| **Version**  | 2.2, Verification Applied |
| **Status**   | In Review                 |
| **Date**     | May 2026                  |
| **Author**   | Abdul Kader Safi          |
| **Project**  | Safi Studio, Open Source  |
| **Language** | Rust (2021 Edition)       |
| **License**  | MIT (proposed)            |

---

## 1. Executive Summary

Safi-UI is an open-source Rust library that brings a declarative, XML-driven component model to native mobile development, enabling developers to build applications for Android and iOS using a workflow inspired by React Native, but compiled natively with zero managed runtime overhead.

Instead of writing verbose low-level rendering code, developers author UI in structured XML files that are loaded at runtime, parsed into a virtual node tree, laid out via the Taffy flex engine, and rendered through a custom retained-mode paint system built on SDL3 and SDL_GPU.

The rendering pipeline uses native Vulkan on Android and native Metal on iOS, both driven through SDL_GPU with no translation layers, no MoltenVK, and no OpenGL ES fallback. The library is a ground-up Rust rewrite inspired by MicroUI's command list architecture, rebuilt with mobile-first design, a dirty-driven repaint model, and a full component system.

The library targets Rust and C++ developers, game engineers, and embedded programmers who want a native mobile UI without adopting Flutter, React Native, or any managed runtime. It ships as a Cargo library with CMake interop for NDK and Xcode integration.

---

## 2. Problem Statement

### 2.1 The Gap

Building a structured, component-based mobile UI in native code today requires adopting a managed runtime (Flutter's Dart VM, React Native's V8, Compose's JVM). For performance-critical apps, games, or embedded targets this is unacceptable. The alternatives, writing raw platform UI in Java/Kotlin or Swift/ObjC, require maintaining two completely separate codebases.

There is no mature open-source library that provides a declarative XML component system, a flex layout engine, and a native GPU renderer (Vulkan + Metal) for mobile from a single pure Rust codebase.

### 2.2 Why Not Flutter / React Native / Compose?

These frameworks introduce managed runtimes that significantly increase binary size, startup latency, and memory overhead. They also prevent direct GPU access and make interop with C/C++ game engines painful.

### 2.3 Why Not ImGui or MicroUI Directly?

Both are immediate-mode libraries designed for developer tooling. They redraw every frame regardless of whether anything changed (battery drain on mobile), have no flex layout system, treat touch as mouse emulation, use bitmap fonts unusable at mobile DPI, and have no component reuse model.

Safi-UI is architecturally inspired by MicroUI's command list pattern, that core idea is retained, repurposed, and rebuilt for production mobile use in Rust.

### 2.4 The Opportunity

A declarative XML-driven mobile UI framework in pure Rust on SDL3 does not exist. Safi-UI fills this gap.

---

## 3. Goals and Non-Goals

### 3.1 Goals

- Provide an XML-driven, declarative UI authoring experience for Android and iOS
- Implement a retained-mode, **per-subtree** dirty repaint system for battery efficiency
- Use SDL3 and SDL_GPU with native Vulkan on Android and native Metal on iOS, no translation layers
- Rebuild MicroUI's command list architecture in Rust with mobile-first design
- Use Taffy (pure Rust) for CSS Flexbox-compatible layout computation
- Support hot-reload of XML files in development mode with **seamless state preservation**
- Ship as a Cargo library with full Android NDK and iOS Xcode integration
- Provide a reactive state store and named event bus for component communication
- Support a full component system: built-in components and user-defined components via XML templates or Rust registration
- Build a community around easy XML authoring, users write XML, not Rust
- Support FlatList virtualization with windowed recycling for 1k+ items from v1
- Support reverse-infinite-scroll (chat-style) in FlatList from v1

### 3.2 Non-Goals (v1)

- Visual drag-and-drop editor (planned for v2)
- Windows / macOS / Linux desktop targets (community can add SDL3 backends later)
- Animation system, static layouts only in v1
- CSS stylesheet files, styling is done via XML props only
- Accessibility (screen reader) support, reserved props stubbed for v2
- WebAssembly target
- Scripting language bindings (Lua, Python), v2 stretch goal
- **Multi-window** (iPad split-view, Android freeform), deliberate v1 trade-off
- **Navigation stack**, deferred to v1.1
- **C FFI custom component registration**, Rust-only in v1

---

## 4. Target Users

| User Segment                 | Description                                                                                    |
| ---------------------------- | ---------------------------------------------------------------------------------------------- |
| **Rust Mobile Developers**   | Engineers building native mobile apps in Rust who want structured UI without a managed runtime |
| **Game Developers**          | Developers using custom C/Rust engines who want a companion app or in-game overlay UI          |
| **Embedded / IoT Engineers** | Developers targeting Android hardware where Flutter or React Native is too heavy               |
| **Open Source Contributors** | Developers who want to extend the component library, add platform backends, or write bindings  |
| **Indie App Developers**     | Solo developers who want full control over their rendering pipeline and binary size            |

---

## 5. Architecture Overview

### 5.1 Core Design Philosophy

**Pillar 1, Command List Pattern (from MicroUI)**
All rendering is expressed as a flat list of typed commands emitted during the build phase and consumed by the SDL_GPU renderer. No component ever calls GPU APIs directly. This decouples UI logic from rendering completely and enables efficient GPU batching.

**Pillar 2, Retained Mode with Per-Subtree Dirty Tracking**
The UI does not redraw every frame. A per-subtree dirty flag system tracks exactly which nodes have changed, keyed by `WidgetId`. The GPU is only invoked when something actually needs repainting, and only affected subtrees are rebuilt. This is a v1 requirement, not a v1.1 retrofit.

**Pillar 3, Arena-Based Widget Storage**
All widgets live in a flat arena indexed by `WidgetId`. Widgets reference each other by ID, not by pointer. This solves Rust's borrow checker challenges with UI trees (no `Rc<RefCell<>>`, no cycles) and maps naturally to the game-engine ECS pattern.

### 5.2 System Layers

| Layer       | Module                      | Responsibility                                         |
| ----------- | --------------------------- | ------------------------------------------------------ |
| 1, Source   | XML Files (`assets/ui/`)    | UI authored as `.xml` files, loaded at runtime         |
| 2, Parse    | `XmlParser` (roxmltree)     | Parses XML into a `VNode` tree                         |
| 3, Resolve  | `ComponentRegistry`         | Maps XML tag names to Rust component factories         |
| 4, Layout   | `LayoutEngine` (Taffy)      | Computes CSS Flexbox layout for every node             |
| 5, Build    | `UIContext` + `WidgetArena` | Walks tree, calls `Component::build()`, emits commands |
| 6, Render   | `GpuRenderer` (SDL_GPU)     | Batches and submits command list to Vulkan / Metal     |
| 7, Platform | `PlatformBridge`            | Safe area, keyboard height, DPI, lifecycle via SDL3    |

### 5.3 Data Flow

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

**Dirty cascade rule.** `DirtyTracker::mark_dirty(widget_id)` flags a single widget. The `DirtyTracker` automatically walks the parent chain in `WidgetArena` and also flags any ancestor whose layout sizing depends on the marked widget (e.g., an `auto`-sized parent of a `Text` whose content changed). Ancestors with fully resolved bounds (fixed `width` + `height`, or `flex` constrained by a sized parent) are not cascaded.

### 5.4 Component Resolution Order

1. Check `ComponentRegistry` for a Rust-registered component
2. Check `XmlTemplateLoader` for a user-defined `.xml` component file
3. Fall back to `DebugBox`, renders a red outlined rectangle with the unknown tag name

---

## 6. Core Modules, Detailed Specifications

### 6.1 VNode, Virtual DOM Node

Every XML element is parsed into a `VNode`. It is the single data structure that flows through the parse, layout, and build phases.

**Identity and keying:** Stateful components (`Input`, `ScrollView`, `FlatList` items, `BottomSheet`, etc.) require an `id` prop. A separate `key` prop (React-style, scoped to siblings) is distinct from `id` (globally unique) and is used by FlatList recycling to preserve item state across data reorders.

```rust
pub struct VNode {
    pub tag:          String,
    pub props:        Props,           // HashMap<String, String>
    pub children:     Vec<VNode>,
    pub text_content: Option<String>,
    pub layout:       LayoutRect,      // populated by LayoutEngine
    pub id:           Option<String>,  // required for stateful components
    pub key:          Option<String>,  // sibling-scoped, for list recycling
}

pub type Props = HashMap<String, String>;

pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

All prop values are strings. Components parse them via `PropUtils` helpers. Coordinates are in density-independent pixels (dp); the renderer converts to physical pixels using the DPI scale factor.

### 6.2 UIContext and WidgetArena

`UIContext` is the central frame state. It owns the `CommandBuffer`, `DirtyTracker`, `FocusSystem`, and `ClipStack`. It is passed by mutable reference to every `Component::build()` call.

`WidgetArena` is the flat storage for all widget instances. Widgets reference each other by `WidgetId` (a `u32` index), never by pointer.

```rust
pub type WidgetId = u32;

pub struct WidgetArena {
    widgets:    Vec<Box<dyn Component>>,
    taffy_nodes: Vec<taffy::NodeId>,
    bounds:     Vec<LayoutRect>,
    children:   Vec<Vec<WidgetId>>,
    parent:     Vec<Option<WidgetId>>,
}

pub struct UIContext {
    pub commands:   CommandBuffer,
    pub dirty:      DirtyTracker,
    pub focus:      FocusSystem,
    pub clips:      ClipStack,
    pub dpi_scale:  f32,
    pub safe_area:  EdgeInsets,
}
```

### 6.3 CommandBuffer

A growable array of typed draw commands. The initial capacity is set at `App::init()` time (default: 8192). If the buffer reaches capacity it grows and a warning is logged. A debug-mode warning is emitted when utilisation crosses 75% on any frame.

```rust
pub enum Command {
    Rect    { rect: Rect, color: Color, radius: f32 },
    Border  { rect: Rect, color: Color, radius: f32, thickness: f32 },
    Text    { pos: Vec2, text: String, font: FontHandle, size: f32, color: Color },
    Image   { rect: Rect, texture: TextureHandle, radius: f32, fit: ImageFit },
    Shadow  { rect: Rect, color: Color, blur: f32, offset: Vec2 },
    Clip    { rect: Rect },
    ClipPop,
}

// Grows on overflow (log + grow, never drop or panic).
// Initial cap configurable at App::init().
// Debug warning at 75% utilisation.
const COMMAND_BUFFER_CAPACITY_DEFAULT: usize = 8192;
```

### 6.4 DirtyTracker, Per-Subtree Granularity

The `DirtyTracker` is per-subtree from v1, keyed by `WidgetId`. `StateStore::set()` tracks which `WidgetId`s subscribed to which keys, so only those subtrees are invalidated. The GPU renderer tracks which command-buffer ranges changed, avoiding full buffer rebuilds on partial updates.

```rust
pub struct DirtyTracker {
    dirty_widgets: HashSet<WidgetId>,            // per-subtree dirty bits
    state_subs:    HashMap<String, Vec<WidgetId>>, // key → subscribed widgets
}

impl DirtyTracker {
    pub fn mark_dirty(&mut self, id: WidgetId)    { self.dirty_widgets.insert(id); }
    pub fn needs_redraw(&self) -> bool             { !self.dirty_widgets.is_empty() }
    pub fn on_frame_complete(&mut self)            { self.dirty_widgets.clear(); }
    pub fn subscribe(&mut self, key: &str, id: WidgetId) {
        self.state_subs.entry(key.to_string()).or_default().push(id);
    }
    pub fn invalidate_key(&mut self, key: &str) {
        if let Some(ids) = self.state_subs.get(key) {
            for &id in ids { self.dirty_widgets.insert(id); }
        }
    }
}
```

The main loop only calls `UIContext::build()` and `GpuRenderer::flush()` on dirty subtrees. On a fully static screen, no GPU work is performed between frames.

### 6.5 XmlParser

| Attribute          | Detail                                                   |
| ------------------ | -------------------------------------------------------- |
| **Crate**          | `roxmltree` (pure Rust, MIT licensed)                    |
| **Input**          | File path (`std::path::Path`) or string slice            |
| **Output**         | `Result<VNode, ParseError>`                              |
| **Encoding**       | UTF-8 only                                               |
| **Error handling** | Returns `Err(ParseError)` with file name and line number |
| **Max file size**  | Recommended < 512 KB per screen file                     |

### 6.6 LayoutEngine (Taffy Integration)

The `LayoutEngine` wraps the Taffy crate to compute CSS Flexbox layout from the VNode tree before the build phase. Taffy is pure Rust, no C FFI, no Yoga dependency.

| Supported Prop     | Taffy Mapping                                                               |
| ------------------ | --------------------------------------------------------------------------- |
| `flexDirection`    | `FlexDirection::Row / Column / RowReverse / ColumnReverse`                  |
| `justifyContent`   | `JustifyContent::FlexStart / Center / FlexEnd / SpaceBetween / SpaceAround` |
| `alignItems`       | `AlignItems::FlexStart / Center / FlexEnd / Stretch`                        |
| `flex`             | `Style::flex_grow`                                                          |
| `width / height`   | `Dimension::Points / Percent / Auto`                                        |
| `padding / margin` | `Rect<LengthPercentageAuto>`                                                |
| `gap`              | `Size<LengthPercentage>`                                                    |
| `wrap`             | `FlexWrap::NoWrap / Wrap / WrapReverse`                                     |

### 6.7 ComponentRegistry

```rust
pub type ComponentFactory =
    Box<dyn Fn(&Props) -> Box<dyn Component> + Send + Sync>;

pub struct ComponentRegistry {
    factories: HashMap<String, ComponentFactory>,
}
```

Supports both built-in registration (at library init) and user registration (at app startup). The `register_component!(tag, Type)` macro is provided as a convenience wrapper. Duplicate registrations log a warning; the last registration wins.

### 6.8 Component Trait

Every widget implements `Component`. `build()` is strictly **main-thread**. The `Component` trait itself has **no `Send + Sync` bound**, only the factory closures stored in `ComponentRegistry` (which may be invoked from any thread during registration) require `Send + Sync`. This lets custom components hold `Rc<…>`, `RefCell<…>`, or other non-`Send` handles without ceremony, since instances only ever live on the main thread.

```rust
pub trait Component {
    // Emit draw commands for this widget's current state
    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect);

    // Return true if the point is inside this widget's interactive area
    fn hit_test(&self, point: Vec2) -> bool { self.bounds().contains(point) }

    // Handle a recognised gesture; return true to consume it
    fn on_gesture(&mut self, gesture: Gesture) -> bool { false }

    // Lifecycle hooks
    fn on_mount(&mut self, _ctx: &mut UIContext)    {}  // fires whenever entering the live tree
    fn on_unmount(&mut self, _ctx: &mut UIContext)  {}  // fires whenever leaving the live tree
    fn on_layout(&mut self, _bounds: LayoutRect)    {}  // fires only when bounds change

    fn bounds(&self) -> LayoutRect;
}
```

**Lifecycle semantics:**

- `on_mount` fires **whenever a component enters the live tree**: first appearance, post-hot-reload remount of an unmatched node, or recycle-back-into-window for a virtualised FlatList item
- `on_unmount` fires whenever a component leaves the live tree, including FlatList recycle-out
- `visible="false"` skips `build()` only, does **not** fire `on_unmount`; the instance remains alive and its state is preserved
- `on_layout` fires **only when bounds change** (delta vs the last laid-out frame). It does not fire every frame on static layouts. Use this to position tooltips or measure-dependent children without thrashing.

### 6.9 GestureRecognizer

| Gesture     | Trigger             | Threshold                | Used By                     |
| ----------- | ------------------- | ------------------------ | --------------------------- |
| `Tap`       | Finger down + up    | < 200ms, < 10dp movement | Button, Checkbox, `onPress` |
| `LongPress` | Finger held         | > 500ms                  | Tooltip, context menus      |
| `Pan`       | Continuous movement | ,                        | ScrollView drag             |
| `Swipe`     | Fast pan + release  | velocity threshold       | Drawer, BottomSheet dismiss |

```rust
pub enum Gesture {
    Tap       { pos: Vec2 },
    LongPress { pos: Vec2 },
    Pan       { delta: Vec2, velocity: Vec2 },
    Swipe     { direction: SwipeDirection, velocity: f32 },
}
```

### 6.10 Hot-Reload System (Dev Mode)

Hot-reload is **seamless**. All component state is preserved across re-parses. Stateful components are matched between old and new trees using their stable `id` prop. Components without a matching `id` in the new tree are treated as new instances.

**State preservation:** scroll offsets, input cursor positions, open/closed states, focus, and in-flight gesture state are preserved by mapping old `WidgetId`s to new `WidgetId`s via the stable `id` prop. After re-parse, event subscriptions are re-registered and the `DirtyTracker` marks affected subtrees dirty.

| Attribute              | Detail                                                          |
| ---------------------- | --------------------------------------------------------------- |
| **Feature flag**       | `safi-ui = { features = ["dev"] }` in Cargo.toml                |
| **Trigger**            | File modification timestamp change on any watched `.xml` file   |
| **Watch scope**        | `assets/ui/` directory tree (recursive)                         |
| **Reload latency**     | < 100ms from file save to new frame                             |
| **Reload scope**       | Affected screen only                                            |
| **State preservation** | Seamless via stable `id` mapping, no visible flash              |
| **Error display**      | See §14 for dev error overlay UX                                |
| **Release builds**     | `HotReloadWatcher` compiled out entirely, zero runtime overhead |

### 6.11 EventBus

A main-thread publish/subscribe bus for named events with a cross-thread `post_async` ingress. `emit()` and handler registration are **main-thread-only**. `post_async()` is the only API callable from background threads; it enqueues the event onto a thread-safe queue that the main loop drains on the next frame.

```rust
// Register a handler (main thread)
EventBus::global().on("auth.login", || {
    AuthService::login();
});

// Emit synchronously (main thread only)
EventBus::global().emit("auth.login");

// Safe cross-thread post, dispatched on next frame
EventBus::global().post_async("data.refresh");

// In XML:  onPress="auth.login"
// Dynamic: onPress="{{dynamicAction}}"  ← binding supported
```

Event names use dot notation by convention.

### 6.12 StateStore

A reactive key-value store. Prop bindings use `{{key}}` syntax in XML. `StateStore::set()` calls `DirtyTracker::invalidate_key()`, so only subscribed subtrees rebuild. `set()` and `get()` are **main-thread-only**; background threads use `EventBus::post_async` to trigger state updates.

```rust
// App code uses the global singleton
StateStore::global().set("user.name", "Safi");
let name = StateStore::global().get("user.name");

// Tests create isolated instances
let store = StateStore::new();

// Subscribe to changes
StateStore::global().subscribe("user.name", |value| {
    println!("Name changed to: {}", value);
});
```

**Binding rules:**

- Missing key resolves to **empty string** (never an error)
- Bindings are allowed in **any prop**, including `width`, `src`, `onPress`
- Composite bindings are supported: `"Hello {{name}}!"`, `"{{first}} {{last}}"`
- Composite bindings register a `DirtyTracker` subscription on **every** key referenced in the template, so any one of them changing invalidates the owning subtree
- Dynamic event bindings are supported: `onPress="{{dynamicAction}}"`
- Type coercion: bound values are strings; `PropUtils::parse_*` coerces as normal

**Background-thread state updates.** Network fetches, file IO, and other background work cannot call `StateStore::set` directly. The recommended pattern is to post a marker event from the worker thread and apply the state mutation in its main-thread handler:

```rust
// On a background thread (e.g. an HTTP client task):
let json = http::get("/api/projects").await?;
PENDING_PROJECTS.lock().unwrap().replace(json);
EventBus::global().post_async("projects.fetched");

// Registered once on the main thread at startup:
EventBus::global().on("projects.fetched", || {
    if let Some(json) = PENDING_PROJECTS.lock().unwrap().take() {
        StateStore::global().set("projects.recent", json);
    }
});
```

The store is not persisted to disk in v1.

### 6.13 UIInstance and App

A `UIInstance` / `App` handle owns its own `StateStore` and `EventBus`. `::global()` is a convenience returning the default instance. Multi-window is explicitly out of scope for v1; one process = one app is the expected model. Unit tests create new `StateStore` and `EventBus` instances per test case for clean isolation.

### 6.14 PropUtils

A module of typed prop parsing helpers used by all components. Always returns a typed default, no component ever receives an unhandled `None`.

```rust
let label   = props.get_str("label", "Button");
let size    = props.parse_f32("size", 14.0);
let visible = props.parse_bool("visible", true);
let color   = props.parse_color("color", Color::WHITE);
let width   = props.parse_dim("width", Dimension::Auto);

// Resolves {{key}} bindings (missing key → "")
let text = props.resolve_binding("label", state_store);
```

### 6.15 `vnode!` Macro (Programmatic Authoring)

A declarative macro that builds a `VNode` tree directly in Rust, used by Phase 1 work (before the XML parser exists), by tests, and by anyone embedding Safi-UI without an `assets/ui/` directory. Syntactically it mirrors XML.

```rust
let tree: VNode = vnode! {
    <Screen bg="#0f0f1a" safeArea="true">
        <Column gap="12" padding="16">
            <Heading level="2" color="#fff">"Hello"</Heading>
            <Button id="cta" label="Tap me" onPress="demo.tap" />
        </Column>
    </Screen>
};
```

| Attribute          | Detail                                                                                                          |
| ------------------ | --------------------------------------------------------------------------------------------------------------- |
| **Output type**    | `VNode` (same struct produced by `XmlParser::parse`)                                                            |
| **Prop values**    | String literals only (matches the runtime model where all props are `String`)                                   |
| **Text content**   | A bare string literal becomes `VNode::text_content`                                                             |
| **Bindings**       | Written verbatim: `value="{{user.name}}"`                                                                       |
| **Compile errors** | Unknown tags compile fine (resolved at runtime via `ComponentRegistry`); malformed syntax fails at compile time |
| **Hot-reload**     | Trees produced by `vnode!` are not hot-reloadable (no source file to watch)                                     |

The macro lives in a `safi-ui-macros` proc-macro crate and is re-exported from `safi-ui`. It is the only sanctioned way to construct `VNode` trees by hand — direct struct construction is technically possible but bypasses validation.

---

## 7. Font and Text Pipeline

### 7.1 Font Rasterization

Safi-UI uses `fontdue` for font rasterization, a pure Rust font engine with no C dependencies and no FreeType requirement. Glyphs are rasterized into a GPU texture atlas at startup and rebuilt when the DPI scale changes. Default bundled fonts: **Inter** (Latin scripts) and **Noto Sans Arabic** (RTL support).

### 7.2 Text Shaping

The `rustybuzz` crate (a pure Rust port of HarfBuzz) handles text shaping for complex scripts and RTL text before rasterization. This enables correct Arabic, Hindi, Thai, and other complex script rendering.

### 7.3 Density-Independent Pixels

All XML coordinates are in **dp** (density-independent pixels). Conversion to physical pixels happens at the `GpuRenderer` boundary:

```
physical_pixels = dp_value × dpi_scale

Pixel 8 (420 dpi):   dpi_scale = 2.625
iPhone 15 Pro:        dpi_scale = 3.0
Desktop 1080p:        dpi_scale = 1.0
```

`SDL_GetDisplayContentScale()` provides the scale factor at runtime.

---

## 8. GPU Rendering Pipeline

### 8.1 SDL3 and SDL_GPU

| Platform | GPU Backend | Notes                        |
| -------- | ----------- | ---------------------------- |
| Android  | Vulkan 1.1  | Native, no translation layer |
| iOS      | Metal       | Native, no MoltenVK          |

No OpenGL ES. No MoltenVK. No translation layers of any kind.

### 8.2 Shader Strategy

Safi-UI ships GLSL shaders compiled to both SPIR-V (Vulkan / Android) and MSL (Metal / iOS) at build time via `glslc` in `build.rs`.

| Shader        | Purpose                                                          |
| ------------- | ---------------------------------------------------------------- |
| `rect.glsl`   | Solid and gradient filled rectangles with optional corner radius |
| `border.glsl` | Bordered rectangles with corner radius                           |
| `text.glsl`   | Font atlas glyph sampling                                        |
| `image.glsl`  | Texture sampling with cover / contain / fill modes               |
| `shadow.glsl` | Box shadow approximation                                         |

### 8.3 GPU Batching

The `GpuRenderer` iterates the `CommandBuffer` after each build phase and batches consecutive compatible commands into single draw calls. A typical screen of 50–80 components produces 5–15 actual GPU draw calls. The renderer tracks which command-buffer ranges changed (per-subtree dirty) to avoid full buffer rebuilds on partial updates.

### 8.4 Threading Model

- `Component::build()` is strictly **main-thread**
- `StateStore::set()` and `EventBus::emit()` are **main-thread-only**
- `EventBus::post_async()` is the safe cross-thread posting path
- Background image decode signals the main thread via a **channel**; the main thread uploads to SDL_GPU and marks dirty
- `Component` itself has no `Send + Sync` bound; only the registry's factory closures do (so registration can happen from any thread, while instances stay main-thread-only)

### 8.5 Frame Loop

The decoded-image channel carries `DecodedImage { owner_id: WidgetId, src: String, pixels: image::RgbaImage }`. The main thread drains it once per frame, uploads each payload to a GPU texture, inserts it into the LRU cache, and marks the requesting widget dirty.

```rust
struct DecodedImage {
    owner_id: WidgetId,         // widget that requested the image
    src:      String,            // cache key
    pixels:   image::RgbaImage,  // decoded pixels, ready for upload
}

let image_channel: crossbeam::channel::Receiver<DecodedImage> = /* … */;

loop {
    for event in sdl.event_pump() {
        match event {
            Event::FingerDown { id, x, y, .. } => {
                gesture_recognizer.finger_down(id, Vec2::new(x, y));
            }
            Event::WillEnterBackground => gpu.release_resources(),
            Event::DidEnterForeground  => gpu.recreate_resources(),
            Event::LowMemory           => image_cache.evict_all(),
            Event::Quit                => break 'main,
            _ => {}
        }
    }

    // Drain async-posted EventBus messages from background threads
    event_bus.drain_async();

    // Process decode completions from the background thread pool
    while let Ok(decoded) = image_channel.try_recv() {
        let texture = gpu.upload_texture(&decoded.pixels);
        image_cache.insert(decoded.src, texture);
        ctx.dirty.mark_dirty(decoded.owner_id);
    }

    gesture_recognizer.flush(&mut arena, &mut event_bus);

    if ctx.dirty.needs_redraw() {
        layout_engine.compute_if_dirty(&mut vnode_tree);
        ctx.commands.clear();
        build_dirty_subtrees(&mut ctx, &arena, &vnode_tree);
        gpu_renderer.flush(&ctx.commands);
        ctx.dirty.on_frame_complete();
    } else {
        std::thread::sleep(Duration::from_millis(8));
    }
}
```

---

## 9. Platform Bridge

### 9.1 Android

| Attribute       | Detail                                                       |
| --------------- | ------------------------------------------------------------ |
| **NDK version** | r25 or later                                                 |
| **API level**   | minSdk 24 (Android 7.0, Vulkan guaranteed), targetSdk 35     |
| **GPU**         | Vulkan 1.1 via SDL_GPU                                       |
| **Input**       | SDL3 `SDL_EVENT_FINGER_*`, multi-touch native                |
| **Keyboard**    | `SDL_EVENT_TEXT_INPUT` + JNI bridge for keyboard height      |
| **Safe area**   | `WindowInsetsCompat` via JNI → `PlatformBridge::safe_area()` |
| **DPI**         | `SDL_GetDisplayContentScale()`                               |
| **Build tool**  | `cargo-ndk`                                                  |
| **Assets**      | APK `assets/` dir, accessed via `AAssetManager`              |

### 9.2 iOS

| Attribute               | Detail                                             |
| ----------------------- | -------------------------------------------------- |
| **Minimum iOS version** | 16.0                                               |
| **GPU**                 | Metal via SDL_GPU                                  |
| **Input**               | SDL3 `SDL_EVENT_FINGER_*`, UITouch bridged by SDL3 |
| **Keyboard**            | `UIKeyboardWillShowNotification` via ObjC bridge   |
| **Safe area**           | `UIView.safeAreaInsets` via ObjC bridge            |
| **DPI**                 | `SDL_GetDisplayContentScale()`                     |
| **Orientation**         | `SDL_EVENT_DISPLAY_ORIENTATION` → Taffy re-layout  |
| **Build tool**          | `cargo-xcode` or `cargo-mobile2`                   |
| **Assets**              | `.app` bundle, accessed via `Bundle.main`          |

### 9.3 Unified AssetLoader

A unified `AssetLoader` abstraction in Rust wraps `AAssetManager` (Android) and `Bundle.main` (iOS). All asset paths are relative:

| Path Convention         | Contents                                       |
| ----------------------- | ---------------------------------------------- |
| `assets/ui/screens/`    | Screen XML files                               |
| `assets/ui/components/` | User-defined XML components                    |
| `assets/images/`        | Image assets referenced by `<Image src="...">` |

Hot-reload in dev mode loads from bundled assets (not a dev-machine network path).

### 9.4 Safe Area and Keyboard Layout

The `SafeAreaView` component queries `PlatformBridge::safe_area()` and adds padding insets automatically. When the soft keyboard appears, `PlatformBridge::keyboard_height()` returns the current height in dp. Taffy re-runs layout with a reduced available height, pushing focused inputs into the visible area.

---

## 10. Built-in Component Library

All built-in components accept base layout props (`width`, `height`, `padding`, `margin`, `flex`, `visible`, `opacity`, `id`, `key`, `testID`, `onMount`, `onUnmount`, `accessibilityLabel`, `accessibilityRole`) in addition to their specific props.

> **`visible="false"`** hides the component and preserves layout space but does **not** fire `on_unmount`. The component instance remains alive and its state is preserved.

### 10.1 Layout Components

| Component    | Tag              | Key Props                          | Notes                                                     |
| ------------ | ---------------- | ---------------------------------- | --------------------------------------------------------- |
| Screen       | `<Screen>`       | `bg`, `safeArea`                   | Root container, fills viewport, handles safe-area insets  |
| View         | `<View>`         | `bg`, `radius`, `border`, `shadow` | Generic box container                                     |
| Row          | `<Row>`          | `gap`, `align`, `justify`, `wrap`  | `flexDirection: row`                                      |
| Column       | `<Column>`       | `gap`, `align`, `justify`          | `flexDirection: column`                                   |
| Stack        | `<Stack>`        | `align`                            | Absolute-positioned children layered on top of each other |
| ScrollView   | `<ScrollView>`   | `horizontal`, `showsBar`           | `id` required; scroll offset preserved across hot-reload  |
| SafeAreaView | `<SafeAreaView>` | `edges`                            | Platform safe-area inset padding                          |
| Spacer       | `<Spacer>`       | `size`                             | `flex: 1` spacer or fixed gap                             |

### 10.2 Typography Components

| Component | Tag         | Key Props                                                  | Notes                      |
| --------- | ----------- | ---------------------------------------------------------- | -------------------------- |
| Text      | `<Text>`    | `size`, `color`, `weight`, `align`, `italic`, `lineHeight` | Supports `{{binding}}`     |
| Heading   | `<Heading>` | `level` (1–6), `color`                                     | Pre-sized by heading level |
| Label     | `<Label>`   | `size`, `color`, `uppercase`                               | Small uppercase form label |
| Code      | `<Code>`    | `language`, `bg`                                           | Monospace text block       |

### 10.3 Input Components

| Component | Tag          | Key Props                                                 | Notes                                                      |
| --------- | ------------ | --------------------------------------------------------- | ---------------------------------------------------------- |
| Button    | `<Button>`   | `label`, `onPress`, `variant`, `size`, `icon`, `disabled` | `variant`: `primary` \| `secondary` \| `ghost` \| `danger` |
| Input     | `<Input>`    | `placeholder`, `value`, `onChange`, `type`, `maxLength`   | `id` required; cursor position preserved                   |
| TextArea  | `<TextArea>` | `placeholder`, `rows`, `onChange`                         | `id` required; multiline                                   |
| Checkbox  | `<Checkbox>` | `label`, `checked`, `onChange`                            |                                                            |
| Switch    | `<Switch>`   | `value`, `onChange`, `label`                              | Toggle switch                                              |
| Select    | `<Select>`   | `options`, `value`, `onChange`, `placeholder`             | Dropdown picker                                            |
| Slider    | `<Slider>`   | `min`, `max`, `value`, `step`, `onChange`                 |                                                            |

### 10.4 Display Components

| Component   | Tag             | Key Props                                 | Notes                                                                 |
| ----------- | --------------- | ----------------------------------------- | --------------------------------------------------------------------- |
| Image       | `<Image>`       | `src`, `width`, `height`, `radius`, `fit` | `fit`: `cover` \| `contain` \| `fill`; resolved from `assets/images/` |
| Avatar      | `<Avatar>`      | `src`, `size`, `fallback`, `radius`       | `fallback`: initials string                                           |
| Icon        | `<Icon>`        | `name`, `size`, `color`                   | Icon atlas; name maps to UV coords                                    |
| Badge       | `<Badge>`       | `text`, `color`, `bg`, `size`             | Small pill label                                                      |
| Divider     | `<Divider>`     | `color`, `thickness`, `margin`            | Horizontal rule                                                       |
| ProgressBar | `<ProgressBar>` | `value`, `max`, `color`, `height`         |                                                                       |
| Spinner     | `<Spinner>`     | `size`, `color`                           | Loading indicator                                                     |
| Tooltip     | `<Tooltip>`     | `text`, `position`                        | Wraps child; shows on long-press; uses `on_layout` for positioning    |

### 10.5 Navigation Components

| Component   | Tag             | Key Props                                                | Notes                                      |
| ----------- | --------------- | -------------------------------------------------------- | ------------------------------------------ |
| NavBar      | `<NavBar>`      | `title`, `leftAction`, `rightAction`, `bg`, `titleColor` | Fixed top navigation bar                   |
| TabBar      | `<TabBar>`      | `tabs`, `activeTab`, `onTabChange`, `bg`                 | Bottom tab bar                             |
| Drawer      | `<Drawer>`      | `open`, `onClose`, `side`, `width`                       | `id` required; open/closed state preserved |
| Modal       | `<Modal>`       | `open`, `onClose`, `title`, `size`                       | `id` required; centered modal dialog       |
| BottomSheet | `<BottomSheet>` | `open`, `onClose`, `snapPoints`                          | `id` required; slide-up bottom sheet       |

### 10.6 Data Components

| Component  | Tag            | Key Props                                         | Notes                                                                                           |
| ---------- | -------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| FlatList   | `<FlatList>`   | `data`, `renderItem`, `keyExtractor`, `separator` | Windowed recycling. 1k+ items. Reverse-infinite-scroll supported. `key` prop required on items. |
| Card       | `<Card>`       | `elevation`, `bg`, `radius`, `border`, `padding`  | Elevated container                                                                              |
| Table      | `<Table>`      | `columns`, `data`, `striped`, `border`            | Grid table                                                                                      |
| EmptyState | `<EmptyState>` | `icon`, `title`, `message`, `action`              | Placeholder for empty lists                                                                     |

> **FlatList virtualization:** Uses windowed recycling from v1. Recycled item components fire `on_unmount` when scrolled out and `on_mount` when recycled back into view. The `key` prop (not `WidgetId`) is used for state preservation across data reorders. Without a `key`, item state cannot be preserved when the data array reorders.

---

## 11. User-Defined Component System

### 11.1 XML Template Components

Users define reusable components as `.xml` files in `assets/ui/components/`. The engine auto-discovers and registers them at startup.

```xml
<Component name="UserCard" props="name,avatar,role,onPress">
  <Card elevation="4" bg="#1e1e2e">
    <Row align="center" gap="12" onPress="{{onPress}}">
      <Avatar src="{{avatar}}" size="48" />
      <Column flex="1">
        <Text size="16" weight="bold" color="#fff">{{name}}</Text>
        <Text size="12" color="#888">{{role}}</Text>
      </Column>
    </Row>
  </Card>
</Component>
```

Usage: `<UserCard name="Safi" avatar="safi.png" role="Lead Engineer" onPress="nav.profile" />`

Default values: `props="name:Anonymous,role:Member"`.

### 11.2 Rust Registered Components

For components requiring custom rendering, implement `Component` and register via macro. Custom component registration is **Rust-only** in v1.

```rust
pub struct ChartComponent {
    data_key: String,
    color:    Color,
    bounds:   LayoutRect,
}

impl Component for ChartComponent {
    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        let data = StateStore::global().get(&self.data_key);
        ctx.commands.push(Command::Rect {
            rect:   bounds.into(),
            color:  self.color,
            radius: 8.0,
        });
    }
    fn bounds(&self) -> LayoutRect { self.bounds }
}

register_component!("Chart", |props| ChartComponent {
    data_key: props.get_str("data", ""),
    color:    props.parse_color("color", Color::BLUE),
    bounds:   LayoutRect::zero(),
});
```

Usage: `<Chart data="{{analytics.weekly}}" color="#4F8EF7" height="200" />`

---

## 12. XML Authoring Specification

### 12.1 File Structure

| Rule                    | Detail                                               |
| ----------------------- | ---------------------------------------------------- |
| **Encoding**            | UTF-8 required                                       |
| **Root element**        | Each screen file must have exactly one root element  |
| **File extension**      | `.xml`                                               |
| **Screens location**    | `assets/ui/screens/`                                 |
| **Components location** | `assets/ui/components/`                              |
| **Images location**     | `assets/images/`                                     |
| **Screen naming**       | lowercase-hyphen: `home-screen.xml`                  |
| **Component naming**    | PascalCase: `UserCard.xml`                           |
| **Comments**            | Standard XML: `<!-- -->`                             |
| **Max nesting depth**   | No hard limit; 20+ levels logs a performance warning |

### 12.2 Prop Value Types

| Type                  | Format / Examples                                                               |
| --------------------- | ------------------------------------------------------------------------------- |
| **String**            | `label="Sign In"`                                                               |
| **Number**            | `size="18"`, `padding="12"`, `opacity="0.8"`                                    |
| **Boolean**           | `disabled="true"`, `bold="false"`                                               |
| **Color**             | `"#RRGGBB"`, `"#AARRGGBB"`, `"rgba(255,100,0,0.5)"`, `"white"`, `"transparent"` |
| **Dimension**         | `"200"` (dp), `"50%"` (percent of parent), `"auto"`                             |
| **Binding**           | `"{{stateKey}}"`, resolves from StateStore; missing key → `""`                  |
| **Composite binding** | `"Hello {{name}}!"`, `"{{first}} {{last}}"`                                     |
| **Event name**        | `"auth.login"`, `"nav.back"`, `"{{dynamicAction}}"`, bindings allowed           |

### 12.3 Special Props (All Components)

| Prop                 | Purpose                                                                                                                                                |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `id`                 | Globally unique identifier. **Required for stateful components.** Used for StateStore bindings, EventBus targeting, and hot-reload state preservation. |
| `key`                | Sibling-scoped identifier for FlatList item recycling. Required for correct state preservation across data reorders.                                   |
| `visible`            | Boolean; hides component but preserves layout space. Does **not** unmount the component.                                                               |
| `opacity`            | 0.0–1.0 float; applied to entire subtree                                                                                                               |
| `testID`             | String identifier for automated UI testing                                                                                                             |
| `onMount`            | Event name fired when component first appears in the tree                                                                                              |
| `onUnmount`          | Event name fired when component is removed (or recycled out of FlatList window)                                                                        |
| `accessibilityLabel` | Reserved for v2 accessibility support                                                                                                                  |
| `accessibilityRole`  | Reserved for v2 accessibility support                                                                                                                  |

---

## 13. Image Loading Pipeline

| Attribute            | Detail                                                                  |
| -------------------- | ----------------------------------------------------------------------- |
| **Formats**          | PNG, JPEG, WebP via the `image` Rust crate                              |
| **Decoding**         | Async, decoded on a background thread pool                              |
| **Main thread sync** | Decode completion signalled via channel; GPU upload on main thread only |
| **Cache**            | LRU texture cache keyed by `src` string; configurable max size          |
| **Placeholder**      | Shows `Spinner` while loading; `EmptyState` on error                    |
| **Network images**   | `src` starting with `https://` triggers async HTTP fetch (v1.1)         |
| **Eviction**         | `SDL_EVENT_LOW_MEMORY` triggers full cache eviction                     |
| **Asset path**       | Relative paths resolved from `assets/images/` via `AssetLoader`         |

---

## 14. Error Handling and Fault Isolation

### 14.1 Panic Isolation

Panic isolation is **dev builds only** (`#[cfg(debug_assertions)]`). In dev builds, `UIContext` state is snapshotted before each `build()` call (`CommandBuffer.len`, `ClipStack` depth, `FocusSystem` state) and restored on panic. A `DebugBox` is rendered using the **intended layout bounds**. The rest of the UI continues rendering normally. Panics are forwarded to a registered global error handler for crash analytics integration.

**Release-build behaviour.** Release builds do **not** wrap `build()` in `catch_unwind`. A panic inside `Component::build` unwinds into the Safi-UI frame loop and the process aborts (matching the standard Rust `panic = "abort"` profile recommended for mobile binary-size reasons). Components are expected to be panic-safe in release. The global crash handler registered via the dev hook is **also invoked in release** before abort, so apps can flush crash analytics — but it cannot resume rendering.

| Build profile | `build()` panic outcome                                         |
| ------------- | --------------------------------------------------------------- |
| Dev           | Snapshot + restore `UIContext`; render `DebugBox`; UI continues |
| Release       | Crash handler fires (analytics flush), then process aborts      |

### 14.2 Dev Error Overlay UX

| Error Type                    | Behaviour                                                                   |
| ----------------------------- | --------------------------------------------------------------------------- |
| Runtime panic in a component  | Inline `DebugBox` at intended layout bounds                                 |
| XML parse error in hot-reload | Full-screen red overlay showing **all** errors across all reloaded files    |
| Overlay interaction           | Dismissible by tap, previous valid UI visible behind it                     |
| Opt-out                       | Apps can register a custom crash UI in dev and suppress the default overlay |

### 14.3 Invalid Props

`PropUtils` always returns a typed default, no component ever receives an unhandled `None` for a required prop.

---

## 15. Build System and Integration

### 15.1 Cargo.toml

```toml
[package]
name    = "safi-ui"
version = "1.0.0"
edition = "2021"

[features]
default = []
dev     = ["hot-reload"]

[dependencies]
sdl3      = "0.1"
taffy     = "0.4"
fontdue   = "0.8"
rustybuzz = "0.14"
roxmltree = "0.20"
image     = "0.25"
glam      = "0.28"
serde     = { version = "1", features = ["derive"] }
hashbrown = "0.14"
```

### 15.2 Android Build

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android
cargo ndk -t arm64-v8a -o ./android/app/src/main/jniLibs build --release
```

### 15.3 iOS Build

```bash
cargo install cargo-mobile2
cargo mobile init
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --release
```

### 15.4 C FFI (cbindgen)

The C FFI surface is **small**: init, load_screen, frame, shutdown, set_state, on_event. Auto-generated via `cbindgen` at build time. Custom component registration is Rust-only. No C++ wrapper layer, raw C header only.

```c
safi_ui_init(config);
safi_ui_load_screen("home-screen");
safi_ui_frame();
safi_ui_set_state("user.name", "Safi");
safi_ui_on_event(event);
safi_ui_shutdown();
```

### 15.5 CMake Interop

```cmake
add_library(safi_ui STATIC IMPORTED)
set_target_properties(safi_ui PROPERTIES
    IMPORTED_LOCATION
    "${CMAKE_SOURCE_DIR}/target/aarch64-linux-android/release/libsafi_ui.a"
)
target_link_libraries(my_game PRIVATE safi_ui)
```

---

## 16. Development Roadmap

| Phase       | Milestone          | Deliverables                                                                                                                                                                                                                       |
| ----------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Phase 0** | Foundations        | Repo, Cargo setup, `vnode!` macro DSL, SDL3 window on Android + iOS, Vulkan/Metal surface confirmed, GitHub Actions CI                                                                                                             |
| **Phase 1** | Core Engine        | `CommandBuffer` (growable), per-subtree `DirtyTracker`, `UIContext`, `WidgetArena`, `GestureRecognizer`, SDL_GPU rect + text on both platforms. **Acceptance demo:** hand-built button that flips colour on tap on both platforms. |
| **Phase 2** | Layout + Parse     | Taffy integration, roxmltree XML parser, `VNode` tree (`id` + `key` fields), dp unit system, DPI scaling, `AssetLoader` abstraction                                                                                                |
| **Phase 3** | Component Registry | `ComponentRegistry`, `PropUtils` (binding resolution, composite bindings), `Component` trait (`on_layout` hook), `View` + `Text` + `Button` rendering correctly                                                                    |
| **Phase 4** | Component Library  | All built-in components from §10, font atlas (fontdue + rustybuzz), image pipeline (channel-based signalling), icon system decision                                                                                                |
| **Phase 5** | State + Events     | `StateStore` (per-widget subscriptions), `EventBus` (main-thread + `post_async`), `FlatList` windowed recycling + reverse-infinite-scroll, XML template components, `register_component!` macro                                    |
| **Phase 6** | Platform Polish    | Safe area, keyboard layout shift, lifecycle events, hot-reload with seamless state preservation (stable `id` mapping), dev error overlay (dismissible, opt-out), panic isolation + crash handler                                   |
| **Phase 7** | OSS Launch         | Docs, component reference, contribution guide, GitHub release, MIT license, 3 example apps                                                                                                                                         |

---

## 17. Success Metrics

### 17.1 Engineering KPIs

All latency targets measured at **p99**. Binary size measured as **worst case** (arm64).

| Metric                                     | Target                              |
| ------------------------------------------ | ----------------------------------- |
| Cold parse time (1 screen, ~50 nodes)      | < 5ms on Pixel 8 / iPhone 15        |
| Layout compute time (50-node tree)         | < 2ms per frame                     |
| Frame render time (typical screen, dirty)  | < 4ms GPU                           |
| Frame CPU time (static screen, not dirty)  | < 0.1ms (sleep loop)                |
| Hot-reload latency (file save → new frame) | < 100ms                             |
| Binary size overhead vs bare SDL3          | < 800KB stripped (worst case arm64) |
| Built-in component count at v1 launch      | >= 30                               |

### 17.2 Adoption KPIs

| Metric                                       | Target                        |
| -------------------------------------------- | ----------------------------- |
| GitHub stars at 3 months post-launch         | > 300                         |
| Example apps shipping with library           | >= 3 (hello, todo, dashboard) |
| Community PRs in first 3 months              | > 10                          |
| Issues opened by community (adoption signal) | > 20                          |

---

## 18. Resolved Design Decisions

All 15 open questions from the v2.0 review session are resolved. This section is a quick-reference summary; full details are incorporated in the relevant sections above.

| #   | Topic                    | Decision                                                                                                                                                                                                             |
| --- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Hot-reload state         | Seamless. State preserved via stable `id` mapping. No visible flash.                                                                                                                                                 |
| 2   | FlatList virtualization  | Windowed recycling from v1. 1k+ items supported. Reverse-infinite-scroll in v1 scope. `key` prop required on items.                                                                                                  |
| 3   | DirtyTracker granularity | Per-subtree `WidgetId`-keyed from v1. `StateStore` tracks subscriptions per key. GPU renderer tracks changed cmd-buffer ranges.                                                                                      |
| 4   | Panic isolation          | Dev builds only. Snapshot + restore `UIContext` on panic. `DebugBox` at intended layout bounds. Global crash handler hook.                                                                                           |
| 5   | Global singletons        | Multi-window out of scope for v1. Tests use `StateStore::new()` for isolation. `UIInstance` / `App` owns state. `::global()` is a convenience.                                                                       |
| 6   | CommandBuffer overflow   | Grow + warn (never drop or panic). Configurable initial cap at `App::init()`. Debug warning at 75% utilisation.                                                                                                      |
| 7   | Roadmap Phase ordering   | Phase 1 uses programmatic `VNode` trees. `vnode!` macro added in Phase 0. Acceptance demo: tap-to-flip button on both platforms.                                                                                     |
| 8   | Success metrics          | Split into Engineering KPIs (p99 latency, binary size worst-case) and Adoption KPIs (stars, PRs, issues).                                                                                                            |
| 9   | VNode identity / keying  | `id` required for stateful components (globally unique). Separate `key` prop for FlatList siblings (sibling-scoped). `key` required for correct recycle state on reorder.                                            |
| 10  | Threading model          | `build()` main-thread only. `StateStore` / `EventBus` main-thread only. `post_async` for cross-thread. Image decode signals via channel.                                                                             |
| 11  | Prop binding edge cases  | Missing key → empty string. Bindings in any prop. Composite bindings supported. Dynamic event bindings (`onPress="{{action}}"`) supported.                                                                           |
| 12  | Component lifecycle      | `on_mount` fires whenever a component enters the live tree (incl. FlatList recycle-in). `on_unmount` on every leave. `visible="false"` skips `build()` only (no unmount). `on_layout` fires only when bounds change. |
| 13  | Dev error overlay UX     | Runtime panic → inline `DebugBox`. Parse errors → dismissible full-screen overlay showing all errors. Opt-out for custom crash UI.                                                                                   |
| 14  | C FFI surface            | Small surface only: init / load / frame / shutdown / set_state / on_event. Component registration Rust-only. Raw C header via `cbindgen`.                                                                            |
| 15  | Asset bundling           | Android: `AAssetManager`. iOS: `Bundle.main`. Unified `AssetLoader` in Rust. Images from `assets/images/`. Hot-reload uses bundled assets.                                                                           |

---

## 19. External Dependencies

| Crate / Tool    | Version | License    | Purpose                                                  |
| --------------- | ------- | ---------- | -------------------------------------------------------- |
| `sdl3`          | 0.1+    | Zlib       | Window, input, GPU surface, lifecycle                    |
| `taffy`         | 0.4+    | MIT        | Pure Rust CSS Flexbox layout                             |
| `fontdue`       | 0.8+    | MIT        | Pure Rust font rasterization                             |
| `rustybuzz`     | 0.14+   | MIT        | Pure Rust text shaping (HarfBuzz port)                   |
| `roxmltree`     | 0.20+   | MIT        | Pure Rust XML parsing                                    |
| `image`         | 0.25+   | MIT        | Pure Rust image decoding (PNG, JPEG, WebP)               |
| `glam`          | 0.28+   | MIT        | SIMD math primitives (Vec2, Rect, Mat4)                  |
| `serde`         | 1.0+    | MIT        | Serialisation (StateStore persistence v2)                |
| `cargo-ndk`     | 3.0+    | MIT        | Android NDK Rust cross-compilation                       |
| `cargo-mobile2` | 0.x+    | MIT/Apache | iOS Xcode project generation                             |
| `cbindgen`      | 0.26+   | MIT        | C FFI header generation (small surface)                  |
| `glslc`         | SDK     | Apache 2.0 | GLSL → SPIR-V / MSL shader compilation (build-time only) |
| Android NDK     | r25+    | Apache 2.0 | Android native build toolchain                           |
| iOS SDK / Metal | iOS 16+ | Apple EULA | iOS platform GPU API (used via SDL_GPU)                  |

---

## Appendix, Full XML Example

```xml
<Screen bg="#0f0f1a" safeArea="true">

  <NavBar title="Dashboard" bg="#1a1a2e"
          titleColor="#fff"
          rightAction="nav.settings" />

  <ScrollView id="main-scroll" flex="1" padding="16">

    <!-- User profile card (user-defined XML component) -->
    <UserCard name="{{user.name}}"
              avatar="{{user.avatar}}"
              role="{{user.role}}"
              onPress="nav.profile" />

    <Spacer size="16" />

    <!-- Stats row -->
    <Row gap="12" justify="spaceBetween">
      <Card flex="1" bg="#1e1e2e" padding="16" radius="12">
        <Label color="#4F8EF7">PROJECTS</Label>
        <Heading level="2" color="#fff">{{stats.projects}}</Heading>
      </Card>
      <Card flex="1" bg="#1e1e2e" padding="16" radius="12">
        <Label color="#27AE60">CLIENTS</Label>
        <Heading level="2" color="#fff">{{stats.clients}}</Heading>
      </Card>
    </Row>

    <Spacer size="16" />

    <!-- Recent projects list, virtualized, key required on items -->
    <Text size="18" weight="bold" color="#fff">Recent Projects</Text>
    <Spacer size="8" />
    <FlatList id="projects-list"
              data="projects.recent"
              renderItem="ProjectCard"
              keyExtractor="id"
              separator="8" />

    <Spacer size="24" />
    <Button label="Log Out"
            variant="ghost"
            onPress="auth.logout"
            color="#E74C3C" />

  </ScrollView>

  <TabBar activeTab="{{nav.activeTab}}"
          onTabChange="nav.setTab"
          tabs="home,projects,clients,settings" />

</Screen>
```

---

_Safi-UI · PRD v2.1 · Safi Studio · May 2026 · Confidential Draft_
