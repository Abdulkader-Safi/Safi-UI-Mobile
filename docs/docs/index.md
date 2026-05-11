---
pageType: home

hero:
  name: Safi-UI
  text: Native mobile UI in pure Rust
  tagline: Declarative XML, retained-mode rendering, native Vulkan and Metal via SDL_GPU. No managed runtime, no translation layers.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/start/introduction
    - theme: alt
      text: Implementation Status
      link: /status
features:
  - title: XML-Driven Authoring
    details: Author UI in structured XML files. Hot-reload in development with seamless state preservation. Users write XML, not Rust.
    icon: 📝
    link: /guide/authoring/xml-syntax
  - title: Native GPU, Zero Translation
    details: Native Vulkan on Android and Metal on iOS through SDL_GPU. No MoltenVK, no OpenGL ES, no managed runtime overhead.
    icon: ⚡
    link: /guide/concepts/command-buffer
  - title: Retained Mode with Per-Subtree Dirty Tracking
    details: The GPU is only invoked when something actually changes, and only affected subtrees rebuild. Built for mobile battery life.
    icon: 🔋
    link: /guide/concepts/dirty-tracking
  - title: Arena-Based Widget Storage
    details: Widgets reference each other by WidgetId, not by pointer. Solves Rust's borrow-checker UI problem and maps to game-engine ECS patterns.
    icon: 🧩
    link: /guide/concepts/widget-arena
  - title: CSS Flexbox via Taffy
    details: Familiar flex layout, pure-Rust implementation. Same model as React Native, no Yoga C dependency.
    icon: 📐
    link: /guide/concepts/vnode-tree
  - title: 30+ Built-in Components
    details: Layout, typography, input, display, navigation, and data components ship with v1. Extend via XML templates or Rust.
    icon: 🧱
    link: /api/components/
---
