# Typography Components

:::warning Status: Specification (v1.0)
None of these are implemented yet.
:::

| Component | Tag         | Key Props                                                  | Notes                      |
| --------- | ----------- | ---------------------------------------------------------- | -------------------------- |
| Text      | `<Text>`    | `size`, `color`, `weight`, `align`, `italic`, `lineHeight` | Supports `{{binding}}`     |
| Heading   | `<Heading>` | `level` (1–6), `color`                                     | Pre-sized by heading level |
| Label     | `<Label>`   | `size`, `color`, `uppercase`                               | Small uppercase form label |
| Code      | `<Code>`    | `language`, `bg`                                           | Monospace text block       |

## `<Text>`

The primary text component.

| Prop         | Type             | Default  | Notes                                       |
| ------------ | ---------------- | -------- | ------------------------------------------- |
| `size`       | Number           | `14`     | dp                                          |
| `color`      | Color            | `#000`   |                                             |
| `weight`     | `100`–`900` or named | `400` | `normal`, `bold`, etc.                  |
| `align`      | `left` \| `center` \| `right` \| `justify` | `left` |                              |
| `italic`     | Boolean          | `false`  |                                             |
| `lineHeight` | Number           | `1.4`    | Multiplier on `size`                        |

```xml
<Text size="16" weight="bold" color="#fff">{{user.name}}</Text>
```

Bindings are supported in the text content body — `{{user.name}}` resolves the same way it does in props.

## `<Heading>`

Pre-sized typography for hierarchy.

| Level | Size (dp) | Default weight |
| ----- | --------- | -------------- |
| 1     | 32        | 700            |
| 2     | 24        | 600            |
| 3     | 20        | 600            |
| 4     | 18        | 500            |
| 5     | 16        | 500            |
| 6     | 14        | 500            |

## `<Label>`

A small uppercase form label.

| Prop        | Type    | Default |
| ----------- | ------- | ------- |
| `size`      | Number  | `12`    |
| `color`     | Color   | `#666`  |
| `uppercase` | Boolean | `true`  |

## `<Code>`

Monospace text block for code snippets in docs / about screens.

| Prop       | Type   | Default        |
| ---------- | ------ | -------------- |
| `language` | String | none           |
| `bg`       | Color  | `#1e1e2e`      |

Syntax highlighting is **not** in v1; `language` is reserved for future use.
