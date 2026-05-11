# Safi-UI, Product Requirements Document

**v2.0 | Safi Studio**

---

| Field        | Value                    |
| ------------ | ------------------------ |
| **Version**  | 2.0, Draft               |
| **Status**   | In Review                |
| **Date**     | May 2026                 |
| **Author**   | Abdul Kader Safi         |
| **Project**  | Safi Studio, Open Source |
| **Language** | Rust (2021 Edition)      |
| **License**  | MIT (proposed)           |

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

Both are immediate-mode libraries designed for developer tooling. They redraw every frame regardless of whether anything changed (battery drain on mobile), have no flex layout system, treat touch as mouse emulation, use bitmap fonts that are unusable at mobile DPI, and have no component reuse model.

Safi-UI is architecturally inspired by MicroUI's command list pattern, that core idea is retained, repurposed, and rebuilt for production mobile use in Rust.

### 2.4 The Opportunity

A declarative XML-driven mobile UI framework in pure Rust on SDL3 does not exist. Safi-UI fills this gap.

---

## 3. Goals and Non-Goals

### 3.1 Goals

- Provide an XML-driven, declarative UI authoring experience for Android and iOS
- Implement a retained-mode, dirty-driven repaint system for battery efficiency
- Use SDL3 and SDL_GPU with native Vulkan on Android and native Metal on iOS, no translation layers
- Rebuild MicroUI's command list architecture in Rust with mobile-first design
- Use Taffy (pure Rust) for CSS Flexbox-compatible layout computation
- Support hot-reload of XML files in development mode without recompilation
- Ship as a Cargo library with full Android NDK and iOS Xcode integration
- Provide a reactive state store and named event bus for component communication
- Support a full component system: built-in components and user-defined components via XML templates or Rust registration
- Build a community around easy XML authoring, users write XML, not Rust

### 3.2 Non-Goals (v1)

- Visual drag-and-drop editor (planned for v2)
- Windows / macOS / Linux desktop targets (community can add SDL3 backends later)
- Animation system, static layouts only in v1
- CSS stylesheet files, styling is done via XML props only
- Accessibility (screen reader) support, reserved props stubbed for v2
- WebAssembly target
- Scripting language bindings (Lua, Python), v2 stretch goal

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

Safi-UI is built on three architectural pillars:

**Pillar 1, Command List Pattern (from MicroUI)**
All rendering is expressed as a flat list of typed commands emitted during the build phase and consumed by the SDL_GPU renderer. No component ever calls GPU APIs directly. This decouples UI logic from rendering completely and enables efficient GPU batching.

**Pillar 2, Retained Mode with Dirty Tracking**
The UI does not redraw every frame. A dirty flag system tracks which nodes have changed. The GPU is only invoked when something actually needs repainting. This is essential for mobile battery life.

**Pillar 3, Arena-Based Widget Storage**
All widgets live in a flat arena indexed by `WidgetId`. Widgets reference each other by ID, not by pointer. This solves Rust's borrow checker challenges with UI trees (no `Rc<RefCell<>>`, no cycles) and maps naturally to the game-engine ECS pattern.

### 5.2 System Layers

| Layer       | Module                      | Responsibility                                         |
| ----------- | --------------------------- | ------------------------------------------------------ |
| 1. Source   | XML Files (`assets/ui/`)    | UI authored as `.xml` files, loaded at runtime         |
| 2. Parse    | `XmlParser` (roxmltree)     | Parses XML into a `VNode` tree                         |
| 3. Resolve  | `ComponentRegistry`         | Maps XML tag names to Rust component factories         |
| 4. Layout   | `LayoutEngine` (Taffy)      | Computes CSS Flexbox layout for every node             |
| 5. Build    | `UIContext` + `WidgetArena` | Walks tree, calls `Component::build()`, emits commands |
| 6. Render   | `GpuRenderer` (SDL_GPU)     | Batches and submits command list to Vulkan / Metal     |
| 7. Platform | `PlatformBridge`            | Safe area, keyboard height, DPI, lifecycle via SDL3    |

### 5.3 Data Flow

```
XML File
  └─► XmlParser::parse()           →  VNode tree
        └─► LayoutEngine::compute()    →  VNode tree + LayoutRect (Taffy)
              └─► DirtyTracker::check()
                    └─► [if dirty] UIContext::build()
                          └─► ComponentRegistry::resolve(tag)
                                └─► Component::build(ctx, props, bounds)
                                      └─► CommandBuffer::push(command)
                                            └─► GpuRenderer::flush()
                                                  └─► SDL_GPU
                                                        ├─► Vulkan (Android)
                                                        └─► Metal (iOS)

SDL3 Event Loop
  └─► SDL_FINGER_* events
        └─► GestureRecognizer
              └─► HitTest (reverse Z walk on WidgetArena)
                    └─► Component::on_gesture()
                          └─► EventBus / StateStore update
                                └─► DirtyTracker::mark_dirty()
```

### 5.4 Component Resolution Order

1. Check `ComponentRegistry` for a Rust-registered component
2. Check `XmlTemplateLoader` for a user-defined `.xml` component file
3. Fall back to `DebugBox`, renders a red outlined rectangle with the unknown tag name

---

## 6. Core Modules, Detailed Specifications

### 6.1 VNode, Virtual DOM Node

Every XML element is parsed into a `VNode`. It is the single data structure that flows through the parse, layout, and build phases.

```rust
pub struct VNode {
    pub tag: String,
    pub props: Props,                    // HashMap<String, String>
    pub children: Vec<VNode>,
    pub text_content: Option<String>,
    pub layout: LayoutRect,              // populated by LayoutEngine
    pub id: Option<String>,              // optional, for StateStore binding
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

`WidgetArena` is the flat storage for all widget instances. Widgets reference each other by `WidgetId` (a `u32` index), never by pointer. This is the arena pattern that solves Rust's borrow checker UI problem cleanly.

```rust
pub type WidgetId = u32;

pub struct WidgetArena {
    widgets:    Vec<Box<dyn Component>>,
    yoga_nodes: Vec<taffy::NodeId>,
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

The CommandBuffer is a fixed-capacity array of typed draw commands, allocated once at startup. No heap allocation occurs during a frame. This is MicroUI's core insight, preserved and improved.

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

pub struct CommandBuffer {
    commands: Box<[Command; COMMAND_BUFFER_CAPACITY]>,
    len: usize,
}

const COMMAND_BUFFER_CAPACITY: usize = 8192;
```

The `GpuRenderer` iterates the command list after `UIContext::build()` completes and batches consecutive same-type commands into single GPU draw calls.

### 6.4 DirtyTracker

The `DirtyTracker` ensures the GPU is only invoked when the UI has actually changed. On a static screen with no user interaction, no GPU work is performed between frames.

```rust
pub struct DirtyTracker {
    frame_dirty: bool,
    state_hash:  u64,
}

impl DirtyTracker {
    pub fn mark_dirty(&mut self)              { self.frame_dirty = true; }
    pub fn needs_redraw(&self) -> bool        { self.frame_dirty }
    pub fn on_frame_complete(&mut self)       { self.frame_dirty = false; }
    pub fn on_state_changed(&mut self, new_hash: u64) {
        if new_hash != self.state_hash {
            self.state_hash  = new_hash;
            self.frame_dirty = true;
        }
    }
}
```

The main loop only calls `UIContext::build()` and `GpuRenderer::flush()` when `needs_redraw()` returns true. Otherwise it calls `std::thread::sleep(Duration::from_millis(8))` and yields the CPU.

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

The layout pass runs once per frame when the VNode tree is dirty. On static screens the layout is computed once at load time and cached.

### 6.7 ComponentRegistry

A registry mapping tag name strings to factory closures. Supports both built-in registration (at library init) and user registration (at app startup).

```rust
pub type ComponentFactory =
    Box<dyn Fn(&Props) -> Box<dyn Component> + Send + Sync>;

pub struct ComponentRegistry {
    factories: HashMap<String, ComponentFactory>,
}

impl ComponentRegistry {
    pub fn register<C, F>(&mut self, tag: &str, factory: F)
    where
        C: Component + 'static,
        F: Fn(&Props) -> C + Send + Sync + 'static,
    {
        self.factories.insert(
            tag.to_string(),
            Box::new(move |props| Box::new(factory(props))),
        );
    }
}
```

The `register_component!(tag, Type)` macro is provided as a convenience wrapper. Duplicate registrations log a warning; the last registration wins.

### 6.8 Component Trait

Every widget implements `Component`. The trait separates the build phase (emit commands) from hit testing and gesture handling.

```rust
pub trait Component: Send + Sync {
    // Emit draw commands for this widget's current state
    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect);

    // Return true if the point is inside this widget's interactive area
    fn hit_test(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }

    // Handle a recognized gesture; return true to consume it
    fn on_gesture(&mut self, gesture: Gesture) -> bool { false }

    // Lifecycle hooks
    fn on_mount(&mut self, _ctx: &mut UIContext)   {}
    fn on_unmount(&mut self, _ctx: &mut UIContext) {}

    fn bounds(&self) -> LayoutRect;
}
```

### 6.9 GestureRecognizer

A gesture recognizer layer sits between SDL3 finger events and the component system. It translates raw touch events into semantic gestures before routing them to components via hit testing.

```rust
pub enum Gesture {
    Tap       { pos: Vec2 },
    LongPress { pos: Vec2 },
    Pan       { delta: Vec2, velocity: Vec2 },
    Swipe     { direction: SwipeDirection, velocity: f32 },
}

pub struct GestureRecognizer {
    tap:        TapRecognizer,        // finger down + up < 200ms, < 10dp movement
    long_press: LongPressRecognizer,  // finger held > 500ms
    pan:        PanRecognizer,        // continuous move, used by ScrollView
    swipe:      SwipeRecognizer,      // fast pan with release velocity
}
```

Recognizers operate on `SDL_EVENT_FINGER_DOWN`, `SDL_EVENT_FINGER_MOTION`, and `SDL_EVENT_FINGER_UP`. Multi-touch is fully supported; each finger has its own ID.

### 6.10 Hot-Reload System (Dev Mode)

In development builds (`features = ["dev"]`), the `HotReloadWatcher` monitors XML files for modification.

- **Android:** Uses `inotify` via JNI bridge
- **iOS:** Uses `kqueue` via Objective-C bridge

When a modification is detected, the watcher signals the main thread to re-parse and re-layout the affected screen on the next frame. The previous VNode tree is atomically replaced. In release builds the watcher is compiled out entirely with `#[cfg(feature = "dev")]`.

| Attribute              | Detail                                                                  |
| ---------------------- | ----------------------------------------------------------------------- |
| **Feature flag**       | `safi-ui = { features = ["dev"] }` in Cargo.toml                        |
| **Reload latency**     | < 100ms from file save to new frame                                     |
| **Reload scope**       | Affected screen only                                                    |
| **State preservation** | `StateStore` values preserved; event subscriptions re-registered        |
| **Error display**      | Parse errors shown as in-app red overlay with file name and line number |

### 6.11 EventBus

A thread-safe publish/subscribe bus for named events. Components emit events by name; Rust handlers subscribe by name.

```rust
// Register a handler
EventBus::global().on("auth.login", || {
    AuthService::login();
});

// Emit (also triggered from XML: onPress="auth.login")
EventBus::global().emit("auth.login");

// Cross-thread async posting
EventBus::global().post_async("data.refresh");
```

Event names use dot notation by convention. Synchronous dispatch on the main thread. `post_async` queues the event to be dispatched on the next frame.

### 6.12 StateStore

A reactive key-value store. Prop bindings use `{{key}}` syntax in XML. Changes automatically mark the UI dirty via `DirtyTracker`.

```rust
// Set a value
StateStore::global().set("user.name", "Safi");

// Read a value
let name = StateStore::global().get("user.name");

// Subscribe to changes
StateStore::global().subscribe("user.name", |value| {
    println!("Name changed to: {}", value);
});
```

In XML: `<Text>{{user.name}}</Text>`, resolved at build time from the store. The store is not persisted to disk in v1.

### 6.13 PropUtils

A module of typed prop parsing helpers used by all components.

```rust
let label   = props.get_str("label", "Button");
let size    = props.parse_f32("size", 14.0);
let columns = props.parse_i32("columns", 1);
let visible = props.parse_bool("visible", true);
let color   = props.parse_color("color", Color::WHITE);
let width   = props.parse_dim("width", Dimension::Auto);
let align   = props.parse_enum("align", Align::Start, &[
    ("start",  Align::Start),
    ("center", Align::Center),
    ("end",    Align::End),
]);
// Resolves {{key}} bindings from StateStore
let text = props.resolve_binding("label", state_store);
```

---

## 7. Font and Text Pipeline

### 7.1 Font Rasterization

Safi-UI uses `fontdue` for font rasterization, a pure Rust font engine with no C dependencies and no FreeType requirement.

- Fonts are loaded from `.ttf` or `.otf` files bundled in app assets
- Glyphs are rasterized at startup into a GPU texture atlas
- The atlas is rebuilt when the DPI scale changes (orientation change, display change)
- Default bundled fonts: **Inter** (Latin scripts) and **Noto Sans Arabic** (RTL support)

### 7.2 Text Shaping

For complex scripts and RTL text, Safi-UI uses the `rustybuzz` crate (a pure Rust port of HarfBuzz) for text shaping before rasterization. This enables correct Arabic, Hindi, Thai, and other complex script rendering.

### 7.3 Density-Independent Pixels

All coordinates in the XML authoring layer are in **dp** (density-independent pixels). Conversion to physical pixels happens at the `GpuRenderer` boundary:

```
physical_pixels = dp_value * dpi_scale

Pixel 8 (420 dpi):   dpi_scale = 2.625
iPhone 15 Pro:        dpi_scale = 3.0
Desktop 1080p:        dpi_scale = 1.0
```

`SDL_GetDisplayContentScale()` provides the scale factor at runtime.

---

## 8. GPU Rendering Pipeline

### 8.1 SDL3 and SDL_GPU

Safi-UI uses SDL3's `SDL_GPU` API as the GPU abstraction layer. SDL_GPU selects the native GPU backend per platform automatically:

| Platform | GPU Backend | Notes                        |
| -------- | ----------- | ---------------------------- |
| Android  | Vulkan 1.1  | Native, no translation layer |
| iOS      | Metal       | Native, no MoltenVK          |

No OpenGL ES. No MoltenVK. No translation layers of any kind.

### 8.2 Shader Strategy

Safi-UI ships GLSL shaders compiled to both SPIR-V (Vulkan / Android) and MSL (Metal / iOS) at build time via `glslc` in `build.rs`. The correct binary is selected at runtime by SDL_GPU's platform detection.

| Shader        | Purpose                                                          |
| ------------- | ---------------------------------------------------------------- |
| `rect.glsl`   | Solid and gradient filled rectangles with optional corner radius |
| `border.glsl` | Bordered rectangles with corner radius                           |
| `text.glsl`   | Font atlas glyph sampling                                        |
| `image.glsl`  | Texture sampling with cover / contain / fill modes               |
| `shadow.glsl` | Box shadow approximation                                         |

### 8.3 GPU Batching

The `GpuRenderer` iterates the `CommandBuffer` after each build phase and batches consecutive compatible commands into single draw calls. A typical screen of 50–80 components produces 5–15 actual GPU draw calls.

### 8.4 Frame Loop

```rust
loop {
    // 1. Poll SDL3 events
    for event in sdl.event_pump() {
        match event {
            Event::FingerDown { id, x, y, .. } => {
                gesture_recognizer.finger_down(id, Vec2::new(x, y));
                ctx.dirty.mark_dirty();
            }
            Event::WillEnterBackground => gpu.release_resources(),
            Event::DidEnterForeground  => gpu.recreate_resources(),
            Event::LowMemory           => image_cache.evict_all(),
            Event::Quit                => break 'main,
            _ => {}
        }
    }

    // 2. Process gestures → component on_gesture → state updates
    gesture_recognizer.flush(&mut arena, &mut event_bus);

    // 3. Only render if dirty
    if ctx.dirty.needs_redraw() {
        layout_engine.compute_if_dirty(&mut vnode_tree);
        ctx.commands.clear();
        build_tree(&mut ctx, &arena, &vnode_tree);
        gpu_renderer.flush(&ctx.commands);
        ctx.dirty.on_frame_complete();
    } else {
        std::thread::sleep(Duration::from_millis(8));
    }
}
```

---

## 9. Platform Bridge

SDL3 handles window creation, event loop, and GPU surface on both platforms. A thin `PlatformBridge` module supplements SDL3 with mobile-specific information SDL3 does not expose.

### 9.1 Android

| Attribute       | Detail                                                                    |
| --------------- | ------------------------------------------------------------------------- |
| **NDK version** | r25 or later                                                              |
| **API level**   | minSdk 24 (Android 7.0), targetSdk 35                                     |
| **GPU**         | Vulkan 1.1 via SDL_GPU                                                    |
| **Surface**     | SDL3 manages `ANativeWindow` → Vulkan surface                             |
| **Input**       | SDL3 `SDL_EVENT_FINGER_*`, multi-touch native                             |
| **Keyboard**    | `SDL_EVENT_TEXT_INPUT` + JNI bridge for keyboard height                   |
| **Safe area**   | `WindowInsetsCompat` via JNI → `PlatformBridge::safe_area()`              |
| **DPI**         | `SDL_GetDisplayContentScale()`                                            |
| **Lifecycle**   | `SDL_EVENT_WILL_ENTER_BACKGROUND / FOREGROUND / LOW_MEMORY / TERMINATING` |
| **Build tool**  | `cargo-ndk`                                                               |

### 9.2 iOS

| Attribute               | Detail                                                                  |
| ----------------------- | ----------------------------------------------------------------------- |
| **Minimum iOS version** | 16.0                                                                    |
| **GPU**                 | Metal via SDL_GPU                                                       |
| **Surface**             | SDL3 manages `CAMetalLayer` → Metal surface                             |
| **Input**               | SDL3 `SDL_EVENT_FINGER_*`, UITouch bridged by SDL3                      |
| **Keyboard**            | `UIKeyboardWillShowNotification` via ObjC bridge → keyboard height      |
| **Safe area**           | `UIView.safeAreaInsets` via ObjC bridge → `PlatformBridge::safe_area()` |
| **DPI**                 | `SDL_GetDisplayContentScale()`                                          |
| **Orientation**         | `SDL_EVENT_DISPLAY_ORIENTATION` → Taffy re-layout                       |
| **Build tool**          | `cargo-xcode` or `cargo-mobile2`                                        |

### 9.3 Safe Area and Keyboard Layout

The `SafeAreaView` component queries `PlatformBridge::safe_area()` and adds padding insets automatically. When the soft keyboard appears, `PlatformBridge::keyboard_height()` returns the current height in dp. Taffy re-runs layout with a reduced available height, pushing focused inputs into the visible area.

---

## 10. Built-in Component Library

All built-in components accept base layout props (`width`, `height`, `padding`, `margin`, `flex`, `visible`, `opacity`, `id`, `testID`, `onMount`, `onUnmount`, `accessibilityLabel`, `accessibilityRole`) in addition to their specific props.

### 10.1 Layout Components

| Component    | Tag              | Key Props                          | Notes                                                     |
| ------------ | ---------------- | ---------------------------------- | --------------------------------------------------------- |
| Screen       | `<Screen>`       | `bg`, `safeArea`                   | Root container, fills viewport, handles safe-area insets  |
| View         | `<View>`         | `bg`, `radius`, `border`, `shadow` | Generic box container                                     |
| Row          | `<Row>`          | `gap`, `align`, `justify`, `wrap`  | `flexDirection: row`                                      |
| Column       | `<Column>`       | `gap`, `align`, `justify`          | `flexDirection: column`                                   |
| Stack        | `<Stack>`        | `align`                            | Absolute-positioned children layered on top of each other |
| ScrollView   | `<ScrollView>`   | `horizontal`, `showsBar`           | Scrollable via pan gesture                                |
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
| Input     | `<Input>`    | `placeholder`, `value`, `onChange`, `type`, `maxLength`   | `type`: `text` \| `password` \| `number` \| `email`        |
| TextArea  | `<TextArea>` | `placeholder`, `rows`, `onChange`                         | Multiline input                                            |
| Checkbox  | `<Checkbox>` | `label`, `checked`, `onChange`                            |                                                            |
| Switch    | `<Switch>`   | `value`, `onChange`, `label`                              | Toggle switch                                              |
| Select    | `<Select>`   | `options`, `value`, `onChange`, `placeholder`             | Dropdown picker                                            |
| Slider    | `<Slider>`   | `min`, `max`, `value`, `step`, `onChange`                 |                                                            |

### 10.4 Display Components

| Component   | Tag             | Key Props                                 | Notes                                 |
| ----------- | --------------- | ----------------------------------------- | ------------------------------------- |
| Image       | `<Image>`       | `src`, `width`, `height`, `radius`, `fit` | `fit`: `cover` \| `contain` \| `fill` |
| Avatar      | `<Avatar>`      | `src`, `size`, `fallback`, `radius`       | `fallback`: initials string           |
| Icon        | `<Icon>`        | `name`, `size`, `color`                   | Icon atlas; name maps to UV coords    |
| Badge       | `<Badge>`       | `text`, `color`, `bg`, `size`             | Small pill label                      |
| Divider     | `<Divider>`     | `color`, `thickness`, `margin`            | Horizontal rule                       |
| ProgressBar | `<ProgressBar>` | `value`, `max`, `color`, `height`         |                                       |
| Spinner     | `<Spinner>`     | `size`, `color`                           | Loading indicator                     |
| Tooltip     | `<Tooltip>`     | `text`, `position`                        | Wraps child; shows on long-press      |

### 10.5 Navigation Components

| Component   | Tag             | Key Props                                                | Notes                    |
| ----------- | --------------- | -------------------------------------------------------- | ------------------------ |
| NavBar      | `<NavBar>`      | `title`, `leftAction`, `rightAction`, `bg`, `titleColor` | Fixed top navigation bar |
| TabBar      | `<TabBar>`      | `tabs`, `activeTab`, `onTabChange`, `bg`                 | Bottom tab bar           |
| Drawer      | `<Drawer>`      | `open`, `onClose`, `side`, `width`                       | Side drawer overlay      |
| Modal       | `<Modal>`       | `open`, `onClose`, `title`, `size`                       | Centered modal dialog    |
| BottomSheet | `<BottomSheet>` | `open`, `onClose`, `snapPoints`                          | Slide-up bottom sheet    |

### 10.6 Data Components

| Component  | Tag            | Key Props                                         | Notes                       |
| ---------- | -------------- | ------------------------------------------------- | --------------------------- |
| FlatList   | `<FlatList>`   | `data`, `renderItem`, `keyExtractor`, `separator` | `data` is a StateStore key  |
| Card       | `<Card>`       | `elevation`, `bg`, `radius`, `border`, `padding`  | Elevated container          |
| Table      | `<Table>`      | `columns`, `data`, `striped`, `border`            | Grid table                  |
| EmptyState | `<EmptyState>` | `icon`, `title`, `message`, `action`              | Placeholder for empty lists |

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

Usage in any screen file:

```xml
<UserCard name="Safi" avatar="safi.png" role="Lead Engineer" onPress="nav.profile" />
```

Prop substitution uses `{{propName}}` syntax. Default values: `props="name:Anonymous,role:Member"`.

### 11.2 Rust Registered Components

For components requiring custom rendering, complex state, or GPU-level operations, implement `Component` and register via macro:

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
        // additional custom draw commands
    }
    fn bounds(&self) -> LayoutRect { self.bounds }
}

register_component!("Chart", |props| ChartComponent {
    data_key: props.get_str("data", ""),
    color:    props.parse_color("color", Color::BLUE),
    bounds:   LayoutRect::zero(),
});
```

Usage:

```xml
<Chart data="{{analytics.weekly}}" color="#4F8EF7" height="200" />
```

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
| **Screen naming**       | lowercase-hyphen: `home-screen.xml`                  |
| **Component naming**    | PascalCase: `UserCard.xml`                           |
| **Comments**            | Standard XML: `<!-- -->`                             |
| **Max nesting depth**   | No hard limit; 20+ levels logs a performance warning |

### 12.2 Prop Value Types

| Type           | Format / Examples                                                               |
| -------------- | ------------------------------------------------------------------------------- |
| **String**     | `label="Sign In"`                                                               |
| **Number**     | `size="18"`, `padding="12"`, `opacity="0.8"`                                    |
| **Boolean**    | `disabled="true"`, `bold="false"`                                               |
| **Color**      | `"#RRGGBB"`, `"#AARRGGBB"`, `"rgba(255,100,0,0.5)"`, `"white"`, `"transparent"` |
| **Dimension**  | `"200"` (dp), `"50%"` (percent of parent), `"auto"`                             |
| **Binding**    | `"{{stateKey}}"`, resolved from StateStore at build time                        |
| **Event name** | `"auth.login"`, `"nav.back"`, `"modal.open"`                                    |

### 12.3 Special Props (All Components)

| Prop                 | Purpose                                                          |
| -------------------- | ---------------------------------------------------------------- |
| `id`                 | Unique identifier for StateStore bindings and EventBus targeting |
| `visible`            | Boolean; hides component but preserves layout space              |
| `opacity`            | 0.0–1.0 float; applied to entire subtree                         |
| `testID`             | String identifier for automated UI testing                       |
| `onMount`            | Event name fired when component enters the tree                  |
| `onUnmount`          | Event name fired when component leaves the tree                  |
| `accessibilityLabel` | Reserved for v2 accessibility support                            |
| `accessibilityRole`  | Reserved for v2 accessibility support                            |

---

## 13. Image Loading Pipeline

| Attribute          | Detail                                                           |
| ------------------ | ---------------------------------------------------------------- |
| **Formats**        | PNG, JPEG, WebP via the `image` Rust crate                       |
| **Decoding**       | Async, decoded on a background thread pool                       |
| **GPU upload**     | Main thread only, uploaded to SDL_GPU texture on decode complete |
| **Cache**          | LRU texture cache keyed by `src` string; configurable max size   |
| **Placeholder**    | Shows `Spinner` while loading; `EmptyState` on error             |
| **Network images** | `src` starting with `https://` triggers async HTTP fetch (v1.1)  |
| **Eviction**       | `SDL_EVENT_LOW_MEMORY` triggers full cache eviction              |

---

## 14. Error Handling and Fault Isolation

Every component's `build()` call is wrapped in a Rust `catch_unwind` boundary. If a component panics, the error is caught, logged, and a `DebugBox` is rendered in its place. The rest of the UI continues rendering normally.

Invalid prop values are handled by `PropUtils` which always returns a typed default, no component ever receives an unhandled `None` for a required prop.

Parse errors in hot-reload mode are shown as a full-screen red overlay with the file name, line number, and error message. The previous valid UI remains cached and displayed until the XML is fixed.

---

## 15. Build System and Integration

### 15.1 Cargo Structure

```toml
[package]
name    = "safi-ui"
version = "1.0.0"
edition = "2021"

[features]
default = []
dev     = ["hot-reload"]

[dependencies]
sdl3       = "0.1"
taffy      = "0.4"
fontdue    = "0.8"
rustybuzz  = "0.14"
roxmltree  = "0.20"
image      = "0.25"
glam       = "0.28"
serde      = { version = "1", features = ["derive"] }
hashbrown  = "0.14"
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

### 15.4 CMake Interop

For integration with existing C/C++ game engine projects, a C FFI header (`safi_ui.h`) is generated via `cbindgen`:

```cmake
add_library(safi_ui STATIC IMPORTED)
set_target_properties(safi_ui PROPERTIES
    IMPORTED_LOCATION
    "${CMAKE_SOURCE_DIR}/target/aarch64-linux-android/release/libsafi_ui.a"
)
target_link_libraries(my_game PRIVATE safi_ui)
```

---

## 16. Repository Structure

```
safi-ui/
├── Cargo.toml
├── README.md
├── LICENSE
├── build.rs                       ← shader compilation (glslc → SPIR-V + MSL)
├── cbindgen.toml                  ← C FFI header generation
├── shaders/
│   ├── rect.glsl
│   ├── border.glsl
│   ├── text.glsl
│   ├── image.glsl
│   └── shadow.glsl
├── src/
│   ├── lib.rs
│   ├── core/
│   │   ├── vnode.rs
│   │   ├── xml_parser.rs
│   │   ├── component_registry.rs
│   │   ├── layout_engine.rs       ← Taffy integration
│   │   ├── ui_context.rs
│   │   ├── widget_arena.rs
│   │   ├── command_buffer.rs
│   │   ├── dirty_tracker.rs
│   │   ├── event_bus.rs
│   │   ├── state_store.rs
│   │   ├── prop_utils.rs
│   │   ├── gesture_recognizer.rs
│   │   └── hot_reload.rs          ← cfg(feature = "dev") only
│   ├── components/
│   │   ├── layout/
│   │   ├── typography/
│   │   ├── input/
│   │   ├── display/
│   │   ├── navigation/
│   │   └── data/
│   ├── renderer/
│   │   ├── gpu_renderer.rs        ← SDL_GPU command submission + batching
│   │   ├── font_atlas.rs          ← fontdue + rustybuzz atlas builder
│   │   └── image_cache.rs         ← async load + LRU GPU texture cache
│   └── platform/
│       ├── android.rs             ← JNI bridge: safe area, keyboard height
│       └── ios.rs                 ← ObjC bridge: safe area, keyboard height
├── examples/
│   ├── android-hello/
│   ├── ios-hello/
│   └── todo-app/
├── assets/
│   └── ui/
│       ├── screens/
│       └── components/
├── docs/
│   ├── getting-started.md
│   ├── component-reference.md
│   └── custom-components.md
└── tests/
    ├── core/
    └── components/
```

---

## 17. Development Roadmap

| Phase   | Milestone          | Target     | Deliverables                                                                                                            |
| ------- | ------------------ | ---------- | ----------------------------------------------------------------------------------------------------------------------- |
| Phase 0 | Foundations        | Week 1–2   | Repo, Cargo setup, SDL3 window on Android + iOS, Vulkan/Metal surface confirmed, GitHub Actions CI                      |
| Phase 1 | Core Engine        | Week 3–6   | CommandBuffer, DirtyTracker, UIContext, WidgetArena, GestureRecognizer, SDL_GPU rect + text rendering on both platforms |
| Phase 2 | Layout + Parse     | Week 7–9   | Taffy integration, roxmltree XML parser, VNode tree, dp unit system, DPI scaling                                        |
| Phase 3 | Component Registry | Week 10–12 | ComponentRegistry, PropUtils, Component trait, View + Text + Button rendering correctly                                 |
| Phase 4 | Component Library  | Week 13–18 | All built-in components from Section 10, font atlas (fontdue + rustybuzz), image pipeline                               |
| Phase 5 | State + Events     | Week 19–21 | StateStore, EventBus, FlatList data binding, XML template components, `register_component!` macro                       |
| Phase 6 | Platform Polish    | Week 22–24 | Safe area, keyboard layout shift, lifecycle events, hot-reload dev feature, error overlay                               |
| Phase 7 | OSS Launch         | Week 25–26 | Docs, component reference, contribution guide, GitHub release, MIT license, 3 example apps                              |

---

## 18. Success Metrics

| Metric                                         | Target                             |
| ---------------------------------------------- | ---------------------------------- |
| **Cold parse time (1 screen, ~50 nodes)**      | < 5ms on Pixel 8 / iPhone 15       |
| **Layout compute time (50-node tree)**         | < 2ms per frame                    |
| **Frame render time (typical screen, dirty)**  | < 4ms GPU                          |
| **Frame CPU time (static screen, not dirty)**  | < 0.1ms (sleep loop)               |
| **Hot-reload latency (file save → new frame)** | < 100ms                            |
| **Binary size overhead vs bare SDL3**          | < 800KB stripped `.so` / framework |
| **GitHub stars at 3 months post-launch**       | > 300                              |
| **Built-in component count at v1 launch**      | >= 30                              |
| **Example apps shipping with library**         | >= 3 (hello, todo, dashboard)      |

---

## 19. Open Questions and Decisions

| Question                | Options                                                                       | Decision Owner                              |
| ----------------------- | ----------------------------------------------------------------------------- | ------------------------------------------- |
| Icon system             | Embedded font atlas (Material Icons) vs custom image atlas vs SVG via `resvg` | Safi, decide before Phase 4                 |
| FlatList virtualization | Full re-render vs windowed recycling for large lists                          | Safi, decide before Phase 5                 |
| Navigation stack        | Built-in `Navigator` component vs thin `NavManager` utility vs defer to v1.1  | Defer to v1.1                               |
| C FFI surface           | Auto-generate via `cbindgen` vs hand-authored minimal API                     | cbindgen at build time                      |
| Font bundling           | Bundle Inter + Noto Sans Arabic vs system font API bridge                     | Bundle defaults; override via `FontManager` |
| Android minimum API     | minSdk 24 (Vulkan guaranteed) vs minSdk 21 with runtime Vulkan check          | minSdk 24                                   |

---

## 20. External Dependencies

| Crate / Tool    | Version | License    | Purpose                                                  |
| --------------- | ------- | ---------- | -------------------------------------------------------- |
| `sdl3`          | 0.1+    | Zlib       | Window, input, GPU surface, lifecycle                    |
| `taffy`         | 0.4+    | MIT        | Pure Rust CSS Flexbox layout                             |
| `fontdue`       | 0.8+    | MIT        | Pure Rust font rasterization                             |
| `rustybuzz`     | 0.14+   | MIT        | Pure Rust text shaping (HarfBuzz port)                   |
| `roxmltree`     | 0.20+   | MIT        | Pure Rust XML parsing                                    |
| `image`         | 0.25+   | MIT        | Pure Rust image decoding (PNG, JPEG, WebP)               |
| `glam`          | 0.28+   | MIT        | SIMD math primitives (Vec2, Rect, Mat4)                  |
| `serde`         | 1.0+    | MIT        | Serialization (StateStore persistence v2)                |
| `cargo-ndk`     | 3.0+    | MIT        | Android NDK Rust cross-compilation                       |
| `cargo-mobile2` | 0.x+    | MIT/Apache | iOS Xcode project generation                             |
| `cbindgen`      | 0.26+   | MIT        | C FFI header generation                                  |
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

  <ScrollView flex="1" padding="16">

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

    <!-- Recent projects list -->
    <Text size="18" weight="bold" color="#fff">Recent Projects</Text>
    <Spacer size="8" />
    <FlatList data="projects.recent"
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

_Safi-UI · PRD v2.0 · Safi Studio · May 2026_
_Second Version Draft, May 2026_
