# Error Handling and Fault Isolation

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned error model.
:::

## Panic isolation

Panic isolation is **dev builds only** (`#[cfg(debug_assertions)]`). In dev builds, `UIContext` state is snapshotted before each `build()` call (`CommandBuffer.len`, `ClipStack` depth, `FocusSystem` state) and restored on panic. A `DebugBox` is rendered using the **intended layout bounds**. The rest of the UI continues rendering normally.

| Build profile | `build()` panic outcome                                         |
| ------------- | --------------------------------------------------------------- |
| Dev           | Snapshot + restore `UIContext`; render `DebugBox`; UI continues |
| Release       | Crash handler fires (analytics flush), then process aborts      |

Release builds do **not** wrap `build()` in `catch_unwind`. A panic inside `Component::build` unwinds into the Safi-UI frame loop and the process aborts (matching `panic = "abort"` recommended for mobile binary-size). The global crash handler registered via the dev hook is **also invoked in release** before abort, so apps can flush crash analytics — but it cannot resume rendering.

## Dev error overlay

| Error type                    | Behaviour                                                                   |
| ----------------------------- | --------------------------------------------------------------------------- |
| Runtime panic in a component  | Inline `DebugBox` at intended layout bounds                                 |
| XML parse error in hot-reload | Full-screen red overlay showing **all** errors across all reloaded files    |
| Overlay interaction           | Dismissible by tap — previous valid UI visible behind it                    |
| Opt-out                       | Apps can register a custom crash UI in dev and suppress the default overlay |

## Invalid props

`PropUtils` always returns a typed default — no component ever receives an unhandled `None` for a required prop. Missing or malformed values silently fall back to the documented default. See [`PropUtils` API](/api/core/prop-utils).

## Unknown XML tags

Tags with no matching `ComponentRegistry` entry and no matching XML template render as a `DebugBox` (red outlined rectangle showing the unknown tag name) in dev builds. In release builds, the tag is treated as an empty `View`.
