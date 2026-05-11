# VNode Tree

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned `VNode` model.
:::

Every XML element is parsed into a **`VNode`**, the single data structure that flows through the parse, layout, and build phases.

## Structure

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

## All props are strings

All prop values arrive as `String`. Components parse them through [`PropUtils`](/api/core/prop-utils) helpers. This keeps the parser simple and consistent regardless of source (XML file, `vnode!` macro, hot-reload).

## Coordinate units

All XML coordinates are in **dp** (density-independent pixels). Conversion to physical pixels happens at the `GpuRenderer` boundary using `dpi_scale` from `SDL_GetDisplayContentScale()`.

## Identity and keying

Stateful components (`Input`, `ScrollView`, `FlatList` items, `BottomSheet`, etc.) **require** an `id` prop. A separate `key` prop (React-style, scoped to siblings) is distinct from `id` (globally unique) and is used by FlatList recycling to preserve item state across data reorders.

| Prop  | Scope           | Used for                                              |
| ----- | --------------- | ----------------------------------------------------- |
| `id`  | Globally unique | StateStore bindings, EventBus targeting, hot-reload   |
| `key` | Sibling-scoped  | FlatList item state preservation across data reorders |

See [Hot-Reload](/guide/concepts/hot-reload) for how `id` enables seamless state preservation, and [`<FlatList>`](/api/components/data) for `key` semantics.

## Constructing trees programmatically

Outside XML files, trees are built via the [`vnode!` macro](/api/macros#vnode) (planned). Direct struct construction is technically possible but bypasses validation.
