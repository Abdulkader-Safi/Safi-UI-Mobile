# `ComponentRegistry`

:::tip Status: ✅ Shipped (todo 14)
`ComponentRegistry` + `register_component!` macro + `DebugBox` fallback. Three-step resolution per PRD §5.4 (the XML-template middle layer wires in with todo 27).
:::

The dispatch point from "XML tree of `VNode`s" to "tree of `Component` instances." Built-in widgets register at library init; consumer apps register custom widgets at startup; XML template components register lazily as their files are discovered (todo 27).

## Definition

```rust
use safi_ui::registry::{ComponentRegistry, ComponentFactory};
use safi_ui::component::Component;
use safi_ui::vnode::Props;

pub type ComponentFactory =
    Box<dyn Fn(&Props) -> Box<dyn Component> + Send + Sync>;

pub struct ComponentRegistry { /* … */ }

impl ComponentRegistry {
    pub fn new() -> Self;
    pub fn global() -> MutexGuard<'static, Self>;
    pub fn register(&mut self, tag: impl Into<String>, factory: ComponentFactory);
    pub fn resolve(&self, tag: &str, props: &Props) -> Box<dyn Component>;
    pub fn contains(&self, tag: &str) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

The factory type is `Send + Sync` because the registry itself can be touched from any thread during startup. The `Component` instances the factories return are not — they live on the main thread per PRD §6.8.

## Resolution order (PRD §5.4)

1. **Rust factory**: if a factory is registered under `tag`, the registry calls it with the parsed `Props`.
2. **XML template**: not handled by the registry directly. Callers that own both the registry and an `XmlTemplateLoader` consult the loader after `ComponentRegistry::contains(tag)` returns `false`. Lands with todo 27.
3. **`DebugBox` fallback**: any tag the registry doesn't recognise renders as a `DebugBox` — a 1dp red outlined rectangle with the unknown tag name as red text, so the developer sees exactly what XML they need to register.

```rust
let r = ComponentRegistry::new();
let unknown = r.resolve("Mystery", &Props::new());
// → Box<DebugBox> behind the trait object
// build() emits Border + Text("<Mystery>")
```

## Global vs per-test instance

`ComponentRegistry::global()` is a lazily-initialised process singleton, mutex-guarded — convenient for app code and the `register_component!` macro. Tests use `ComponentRegistry::new()` for isolation, so unrelated test fixtures don't bleed into each other.

## The `register_component!` macro

```rust
use safi_ui::{register_component, props::PropsExt};
use safi_ui::component::Component;
use safi_ui::vnode::{LayoutRect, Props};

struct MyChart { /* … */ }

impl Component for MyChart {
    fn bounds(&self) -> LayoutRect { /* … */ }
    fn build(&self, ctx: &mut safi_ui::context::UIContext, bounds: LayoutRect) { /* … */ }
}

register_component!("Chart", |props: &Props| -> Box<dyn Component> {
    Box::new(MyChart { /* read props … */ })
});
```

Then in XML:

```xml
<Chart data="{{analytics.weekly}}" color="#4F8EF7" height="200" />
```

## Duplicate registration

PRD §6.7: duplicate registrations log a warning to stderr **once per tag** (subsequent duplicates of the same tag are silent so test fixtures that re-register on every run don't spam) and last-write-wins.

## See also

- [`Component`](/api/core/component-trait) — the trait every factory returns
- [`PropUtils`](/api/core/prop-utils) — typed prop parsing for factory closures
- `DebugBox` — the fallback widget, defined in `safi_ui::debug_box`
- [PRD §5.4 / §6.7](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md)
