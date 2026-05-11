# `VNode`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

The single data structure that flows through parse → layout → build.

```rust
pub struct VNode {
    pub tag:          String,
    pub props:        Props,
    pub children:     Vec<VNode>,
    pub text_content: Option<String>,
    pub layout:       LayoutRect,
    pub id:           Option<String>,
    pub key:          Option<String>,
}

pub type Props = HashMap<String, String>;

pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

## Fields

| Field          | Notes                                             |
| -------------- | ------------------------------------------------- |
| `tag`          | XML tag name; resolved by `ComponentRegistry`     |
| `props`        | All XML attributes as strings                     |
| `children`     | Child nodes in source order                       |
| `text_content` | Bare text content of the element, if any          |
| `layout`       | Filled in by `LayoutEngine` after the layout pass |
| `id`           | Required for stateful components                  |
| `key`          | Required on FlatList items                        |

## Construction

- **From XML files:** `XmlParser::parse(path)`
- **Programmatically:** [`vnode!`](/api/macros#vnode) macro
- Direct struct construction is allowed but not recommended (bypasses validation)

## See also

- [VNode Tree concept](/guide/concepts/vnode-tree)
