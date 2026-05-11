# Component Lifecycle

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned lifecycle hooks.
:::

Every `Component` has three lifecycle hooks: `on_mount`, `on_unmount`, and `on_layout`.

## Hook signatures

```rust
fn on_mount(&mut self, _ctx: &mut UIContext)    {}  // fires whenever entering the live tree
fn on_unmount(&mut self, _ctx: &mut UIContext)  {}  // fires whenever leaving the live tree
fn on_layout(&mut self, _bounds: LayoutRect)    {}  // fires only when bounds change
```

## Lifecycle rules

| Event                                             | Fires?                                             |
| ------------------------------------------------- | -------------------------------------------------- |
| First appearance in any frame                     | `on_mount`                                         |
| Removed from tree                                 | `on_unmount`                                       |
| Hot-reload remounts an unmatched node             | `on_unmount` then `on_mount` (new instance)        |
| Hot-reload preserves a matched node (stable `id`) | Neither — instance is reused                       |
| FlatList recycles item out of window              | `on_unmount`                                       |
| FlatList recycles item back into window           | `on_mount`                                         |
| `visible="false"` toggled on                      | **Neither** — `build()` is skipped, instance lives |
| `visible="false"` toggled off                     | **Neither** — `build()` resumes                    |
| Layout pass produces same bounds as last frame    | Nothing (no `on_layout`)                           |
| Layout pass produces different bounds             | `on_layout(new_bounds)`                            |

The key principle: `on_mount` / `on_unmount` correspond to **entering and leaving the live tree** — they are not "first ever" / "last ever" events. `on_layout` is **change-detected**, so it does not fire every frame on static layouts.

## Use `on_layout` for measure-dependent positioning

Tooltips, popovers, and any UI that needs to position itself relative to its actual rendered bounds should use `on_layout`. Because it only fires on bounds change, it does not thrash.

```rust
fn on_layout(&mut self, bounds: LayoutRect) {
    // Reposition tooltip arrow to point at the new bounds
    self.arrow_x = bounds.x + bounds.width / 2.0;
}
```

## See also

- [Component trait](/api/core/component-trait)
- [Hot-Reload](/guide/concepts/hot-reload)
