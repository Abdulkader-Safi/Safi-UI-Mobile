# `PropUtils`

:::tip Status: ✅ Shipped (todo 13)
Typed prop parsing + `{{key}}` binding resolution. Methods hang off `Props` via the `PropsExt` extension trait.
:::

Typed prop parsing helpers used by every component. Always returns a typed default — no component ever receives an unhandled `None`.

## Usage

```rust
use safi_ui::props::{Color, Dimension, PropsExt};

let label   = props.get_str("label", "Button");
let size    = props.parse_f32("size", 14.0);
let visible = props.parse_bool("visible", true);
let color   = props.parse_color("color", Color::WHITE);
let width   = props.parse_dim("width", Dimension::Auto);

// Resolves {{key}} bindings (missing key → "")
let text = props.resolve_binding("label", &state_store);
```

## Helpers

| Helper                                 | Returns      | On missing / invalid |
| -------------------------------------- | ------------ | -------------------- |
| `get_str(name, default)`               | `String`     | `default`            |
| `parse_f32(name, default)`             | `f32`        | `default`            |
| `parse_bool(name, default)`            | `bool`       | `default`            |
| `parse_color(name, default)`           | `Color`      | `default`            |
| `parse_dim(name, default)`             | `Dimension`  | `default`            |
| `resolve_binding(name, source)`        | `String`     | `""`                 |

`parse_bool` accepts `"true"`, `"TRUE"`, `"True"`, and `"1"` as truthy. Everything else (including `"false"`, `"0"`, `"no"`) is falsy.

## Color formats

`parse_color` (and the standalone `parse_color_str`) accepts:

| Format       | Example                | Notes                                              |
| ------------ | ---------------------- | -------------------------------------------------- |
| `#RGB`       | `#f00`                 | Each nibble doubled (`#f00` → `#ff0000`)           |
| `#RRGGBB`    | `#0f0f1a`              | Standard 24-bit                                    |
| `#AARRGGBB`  | `#80ff0000`            | Alpha first (8-digit hex)                          |
| `rgb(r,g,b)` | `rgb(255, 200, 100)`   | Each channel 0–255                                 |
| `rgba(r,g,b,a)` | `rgba(255, 100, 0, 0.5)` | RGB 0–255, alpha 0.0–1.0 (CSS convention)        |
| Named        | `transparent`, `white`, `black`, `red`, `green`, `blue`, `gray`/`grey`, `yellow`, `orange`, `purple` | |

Whitespace is trimmed. Malformed input collapses to the supplied default — no panics.

## Dimension formats

`parse_dim` (and the standalone `parse_dim_str`) accepts:

| Format    | Example   | Variant              |
| --------- | --------- | -------------------- |
| Number    | `"200"`   | `Dimension::Dp(200)` |
| `dp` suffix | `"200dp"` | `Dimension::Dp(200)` |
| Percent   | `"50%"`   | `Dimension::Percent(50)` |
| `auto`    | `"auto"` (case-insensitive) | `Dimension::Auto` |

## Bindings

Any prop can contain a `{{key}}` binding. `resolve_binding` resolves it from any [`BindingSource`] — `StateStore` in the runtime, a `HashMap<String, String>` or `HashMap<&str, &str>` in tests. Missing keys resolve to `""` (PRD §6.12) — never an error, never a panic.

Composite bindings substitute multiple keys in one template:

```rust
let mut state: HashMap<String, String> = HashMap::new();
state.insert("first".into(), "Abdul".into());
state.insert("last".into(),  "Safi".into());

// "Hello Abdul Safi!"
let greeting = props.resolve_binding("greeting", &state);
```

`resolve_composite_with_keys` returns both the resolved string and the set of keys touched, so `DirtyTracker` can subscribe the calling widget to every referenced key.

Unterminated `{{` is emitted verbatim — never crashes.

## See also

- [`Component`](/api/core/component-trait) — consumes parsed props in its `build` method
- [`StateStore`](/api/core/state-store) — runtime `BindingSource` implementation (todo 23)
- [PRD §6.14 / §6.12](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md)
