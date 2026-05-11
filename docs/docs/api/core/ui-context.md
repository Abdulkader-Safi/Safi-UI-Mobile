# `UIContext`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

The central frame state passed by mutable reference to every `Component::build()` call.

```rust
pub struct UIContext {
    pub commands:   CommandBuffer,
    pub dirty:      DirtyTracker,
    pub focus:      FocusSystem,
    pub clips:      ClipStack,
    pub dpi_scale:  f32,
    pub safe_area:  EdgeInsets,
}
```

## Fields

| Field       | Purpose                                                                |
| ----------- | ---------------------------------------------------------------------- |
| `commands`  | Push draw commands here                                                |
| `dirty`     | Mark widgets dirty / read subscriptions                                |
| `focus`     | Claim or release keyboard focus                                        |
| `clips`     | Push / pop clip rects (paired with `Command::Clip` / `Command::ClipPop`) |
| `dpi_scale` | Read-only DPI from `SDL_GetDisplayContentScale()`                      |
| `safe_area` | Read-only platform safe-area insets                                    |

## Threading

`UIContext` is **strictly main-thread**. Holding `&mut UIContext` across an `await` or sending it to another thread is impossible by construction (the `&mut` reference doesn't outlive the build call).

## Snapshot / restore (dev mode only)

In dev builds, `UIContext` state (`commands.len`, `clips` depth, `focus` state) is snapshotted before each `build()` call and restored on panic. See [Error Handling](/guide/concepts/error-handling).
