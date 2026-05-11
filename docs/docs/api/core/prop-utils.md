# `PropUtils`

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

Typed prop parsing helpers used by every component. Always returns a typed default — no component ever receives an unhandled `None`.

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

// Resolves {{key}} bindings (missing key → "")
let text = props.resolve_binding("label", state_store);
```

## Helpers

| Helper                                                       | Returns               | On missing / invalid |
| ------------------------------------------------------------ | --------------------- | -------------------- |
| `get_str(name, default)`                                     | `String`              | `default`            |
| `parse_f32(name, default)`                                   | `f32`                 | `default`            |
| `parse_i32(name, default)`                                   | `i32`                 | `default`            |
| `parse_bool(name, default)`                                  | `bool`                | `default`            |
| `parse_color(name, default)`                                 | `Color`               | `default`            |
| `parse_dim(name, default)`                                   | `Dimension`           | `default`            |
| `parse_enum(name, default, options)`                         | enum variant          | `default`            |
| `resolve_binding(name, state)`                               | `String`              | `""`                 |

## Color formats

`parse_color` accepts:

- `#RRGGBB`
- `#AARRGGBB`
- `rgba(r,g,b,a)` with `r,g,b` in 0–255 and `a` in 0.0–1.0
- Named colors: `white`, `black`, `transparent`, plus the basic CSS palette

## Dimension formats

`parse_dim` accepts:

- `"200"` — dp
- `"50%"` — percent of parent
- `"auto"` — content-sized

## Bindings

Any prop can contain a `{{key}}` binding. `resolve_binding` resolves it from the active `StateStore`. Missing keys resolve to `""`. Composite bindings (`"Hello {{name}}!"`) register subscriptions on every key referenced.
