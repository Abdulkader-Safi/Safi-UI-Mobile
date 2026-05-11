# Prop Bindings

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned binding semantics.
:::

Any prop value can reference a `StateStore` key with the `{{key}}` syntax. The value is resolved every time the owning subtree rebuilds.

## Simple bindings

```xml
<Text>{{user.name}}</Text>
<Image src="{{user.avatar}}" />
<Button label="{{cta.label}}" onPress="auth.login" />
```

## Composite bindings

Multiple keys can be interpolated into a single string:

```xml
<Text>Hello {{first}} {{last}}!</Text>
<Text>{{count}} items in cart</Text>
```

Composite bindings register a [`DirtyTracker`](/guide/concepts/dirty-tracking) subscription on **every** key referenced in the template. Any one of them changing invalidates the owning subtree.

## Bindings in any prop

Bindings work in **any** prop, not just text. This includes layout, color, event names, and image sources.

```xml
<Button label="Save"
        onPress="{{dynamicAction}}"
        bg="{{theme.primary}}"
        width="{{layout.buttonWidth}}" />
```

## Missing keys resolve to empty string

A binding that references a key not present in `StateStore` resolves to `""`. This is **never** an error. Numeric and boolean parsers fall back to their documented defaults when given an empty string.

```xml
<!-- if user.age is unset, this renders width=auto (the default) -->
<View width="{{user.age}}" />
```

## Type coercion

Bound values are always strings. `PropUtils::parse_*` coerces them at consumption time:

| Helper                   | Empty / missing string fallback |
| ------------------------ | ------------------------------- |
| `parse_f32(name, def)`   | `def`                           |
| `parse_i32(name, def)`   | `def`                           |
| `parse_bool(name, def)`  | `def`                           |
| `parse_color(name, def)` | `def`                           |
| `parse_dim(name, def)`   | `def`                           |

## Updating bound values

```rust
StateStore::global().set("user.name", "Safi");
// Every component subscribed to user.name re-builds on the next frame
```

See [State and Events](/guide/concepts/state-and-events) for how to update state from a background thread.
