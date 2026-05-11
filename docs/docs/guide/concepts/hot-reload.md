# Hot-Reload

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned hot-reload behaviour.
:::

In development builds (`features = ["dev"]`), the `HotReloadWatcher` monitors XML files for modification. When a change is detected, the affected screen is re-parsed and re-laid-out **on the next frame** with **seamless state preservation**.

## How it works

- **Android:** `inotify` via JNI bridge
- **iOS:** `kqueue` via Objective-C bridge
- **Release builds:** the watcher is compiled out entirely with `#[cfg(feature = "dev")]` — zero runtime overhead

## Spec

| Attribute              | Detail                                                           |
| ---------------------- | ---------------------------------------------------------------- |
| **Feature flag**       | `safi-ui = { features = ["dev"] }` in `Cargo.toml`               |
| **Trigger**            | File modification timestamp change on any watched `.xml` file    |
| **Watch scope**        | `assets/ui/` directory tree (recursive)                          |
| **Reload latency**     | < 100ms from file save to new frame                              |
| **Reload scope**       | Affected screen only                                             |
| **State preservation** | Seamless via stable `id` mapping — no visible flash              |
| **Error display**      | See [Error Handling](/guide/concepts/error-handling)             |
| **Release builds**     | `HotReloadWatcher` compiled out entirely — zero runtime overhead |

## State preservation

State is preserved across re-parse by mapping old `WidgetId`s to new `WidgetId`s via the stable `id` prop. Anything that depended on `WidgetId` continues working:

- Scroll offsets (`<ScrollView id="…">`)
- Input cursor positions (`<Input id="…">`)
- Open/closed states (`<BottomSheet id="…">`, `<Modal id="…">`, `<Drawer id="…">`)
- Focus
- In-flight gesture state
- `StateStore` values (preserved unconditionally)

After re-parse, event subscriptions are re-registered and the `DirtyTracker` marks affected subtrees dirty.

## Identity matching rules

| Old node has `id`? | New node with same `id`? | Outcome                                        |
| ------------------ | ------------------------ | ---------------------------------------------- |
| Yes                | Yes                      | Instance reused, state preserved               |
| Yes                | No                       | Old instance unmounted (`on_unmount` fires)    |
| No                 | —                        | Treated as new instance (no state to preserve) |
| —                  | Yes (no old match)       | Fresh instance, `on_mount` fires               |

This is why **stateful components require an `id`**: without it, hot-reload cannot preserve their state.

## See also

- [Lifecycle](/guide/concepts/lifecycle)
- [VNode Tree](/guide/concepts/vnode-tree)
