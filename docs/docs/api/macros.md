# Macros

:::warning Status: Specification (v1.0)
Not yet implemented.
:::

Two macros are exported from `safi-ui`. Both are proc-macros and live in the `safi-ui-macros` crate, re-exported from the main crate.

## `vnode!`

A declarative macro that builds a `VNode` tree directly in Rust. Used by Phase 1 work (before the XML parser exists), by tests, and by anyone embedding Safi-UI without an `assets/ui/` directory. Syntactically it mirrors XML.

```rust
let tree: VNode = vnode! {
    <Screen bg="#0f0f1a" safeArea="true">
        <Column gap="12" padding="16">
            <Heading level="2" color="#fff">"Hello"</Heading>
            <Button id="cta" label="Tap me" onPress="demo.tap" />
        </Column>
    </Screen>
};
```

| Attribute      | Detail                                                                                                          |
| -------------- | --------------------------------------------------------------------------------------------------------------- |
| Output type    | `VNode` (same struct produced by `XmlParser::parse`)                                                            |
| Prop values    | String literals only (matches the runtime model where all props are `String`)                                   |
| Text content   | A bare string literal becomes `VNode::text_content`                                                             |
| Bindings       | Written verbatim: `value="{{user.name}}"`                                                                       |
| Compile errors | Unknown tags compile fine (resolved at runtime via `ComponentRegistry`); malformed syntax fails at compile time |
| Hot-reload     | Trees produced by `vnode!` are not hot-reloadable (no source file to watch)                                     |

## `register_component!`

Convenience macro for registering a Rust-implemented component with the global `ComponentRegistry`.

```rust
register_component!("Chart", |props| ChartComponent {
    data_key: props.get_str("data", ""),
    color:    props.parse_color("color", Color::BLUE),
    bounds:   LayoutRect::zero(),
});
```

Equivalent to:

```rust
ComponentRegistry::global().register(
    "Chart",
    |props| Box::new(ChartComponent { /* ... */ }),
);
```

Duplicate registrations log a warning; the last registration wins. Resolution order is documented in [User-Defined Components](/guide/authoring/user-components#resolution-order).

## See also

- [`VNode`](/api/core/vnode)
- [`Component` trait](/api/core/component-trait)
- [User-Defined Components](/guide/authoring/user-components)
