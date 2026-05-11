# `CommandBuffer`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

A growable array of typed draw commands. The single ABI between components and the GPU renderer.

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

const COMMAND_BUFFER_CAPACITY_DEFAULT: usize = 8192;
```

## Capacity

Initial capacity is set at `App::init()` time (default `8192`). On overflow the buffer **grows and logs a warning** — never drops or panics.

A debug-mode warning is emitted when utilisation crosses **75%** on any frame. Tune `AppConfig::command_buffer_capacity` to silence the warning if your app legitimately needs more than the default.

## Pushing commands

```rust
ctx.commands.push(Command::Rect {
    rect:   bounds.into(),
    color:  Color::BLUE,
    radius: 8.0,
});
```

## Clip stack

`Command::Clip { rect }` pushes a clip onto the GPU clip stack; `Command::ClipPop` pops it. **You must keep them paired.** Use `ctx.clips.push(rect)` / `ctx.clips.pop()` instead of pushing the commands manually — the helper paired-pushes both the GPU command and the dev-mode tracking.

## See also

- [Command Buffer concept](/guide/concepts/command-buffer)
