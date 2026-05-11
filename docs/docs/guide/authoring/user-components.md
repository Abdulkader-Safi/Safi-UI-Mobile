# User-Defined Components

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned user-component system.
:::

Safi-UI supports two ways to define your own components: **XML templates** for composition, and **Rust registration** for custom rendering.

## XML template components

Drop a `.xml` file into `assets/ui/components/` and the engine auto-discovers and registers it at startup.

`assets/ui/components/UserCard.xml`:

```xml
<Component name="UserCard" props="name,avatar,role,onPress">
  <Card elevation="4" bg="#1e1e2e">
    <Row align="center" gap="12" onPress="{{onPress}}">
      <Avatar src="{{avatar}}" size="48" />
      <Column flex="1">
        <Text size="16" weight="bold" color="#fff">{{name}}</Text>
        <Text size="12" color="#888">{{role}}</Text>
      </Column>
    </Row>
  </Card>
</Component>
```

Use it like any built-in:

```xml
<UserCard name="Safi" avatar="safi.png" role="Lead Engineer" onPress="nav.profile" />
```

### Default values

Defaults use `prop:value` syntax in the `props` attribute:

```xml
<Component name="UserCard" props="name:Anonymous,role:Member,avatar">
  ...
</Component>
```

## Rust registered components

For components that need custom rendering, complex internal state, or GPU-level operations, implement `Component` and register via macro. Custom component registration is **Rust-only** in v1 (no C FFI registration).

```rust
pub struct ChartComponent {
    data_key: String,
    color:    Color,
    bounds:   LayoutRect,
}

impl Component for ChartComponent {
    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        let data = StateStore::global().get(&self.data_key);
        ctx.commands.push(Command::Rect {
            rect:   bounds.into(),
            color:  self.color,
            radius: 8.0,
        });
        // additional custom draw commands
    }
    fn bounds(&self) -> LayoutRect { self.bounds }
}

register_component!("Chart", |props| ChartComponent {
    data_key: props.get_str("data", ""),
    color:    props.parse_color("color", Color::BLUE),
    bounds:   LayoutRect::zero(),
});
```

Use it from XML:

```xml
<Chart data="{{analytics.weekly}}" color="#4F8EF7" height="200" />
```

## Resolution order

When the parser encounters a tag:

1. Check `ComponentRegistry` for a Rust-registered component
2. Check `XmlTemplateLoader` for a user-defined `.xml` component file
3. Fall back to `DebugBox` (red outlined rectangle with the unknown tag name) in dev builds

## See also

- [Component trait](/api/core/component-trait)
- [`register_component!`](/api/macros#register_component)
