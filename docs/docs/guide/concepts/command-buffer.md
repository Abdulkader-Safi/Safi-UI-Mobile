# Command Buffer

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned command-buffer design.
:::

The `CommandBuffer` is a growable array of typed draw commands. It sits between the build phase (components) and the render phase (`GpuRenderer`). **No component ever talks to the GPU directly.**

## Why this pattern

Borrowed from MicroUI: a flat list of typed commands enables efficient GPU batching and decouples UI logic from rendering completely. A typical screen of 50–80 components produces 5–15 actual GPU draw calls.

## Command types

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
```

## Capacity and growth

The initial capacity is set at `App::init()` time (default: `8192` commands). On overflow the buffer **grows and logs a warning** — it never drops or panics. A debug-mode warning is emitted when utilisation crosses 75% on any frame, so teams can size the initial allocation appropriately.

```rust
const COMMAND_BUFFER_CAPACITY_DEFAULT: usize = 8192;
```

## Frame lifecycle

1. At the start of a dirty frame, the affected ranges of the buffer are cleared
2. `Component::build()` calls push commands via `ctx.commands.push(...)`
3. `GpuRenderer::flush()` walks the buffer, batches consecutive compatible commands, and submits draw calls

## Range tracking for partial rebuilds

Per-subtree dirty tracking (see [Dirty Tracking](/guide/concepts/dirty-tracking)) means only changed subtrees rebuild their commands. The renderer tracks which command-buffer ranges changed to avoid full buffer rebuilds on partial updates.

## See also

- [`CommandBuffer` API](/api/core/command-buffer)
- [Dirty Tracking](/guide/concepts/dirty-tracking)
