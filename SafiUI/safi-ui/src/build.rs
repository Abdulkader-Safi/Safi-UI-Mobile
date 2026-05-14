//! `VNode` → Component build walker (PRD §5.3, §6.7).
//!
//! Bridges the parse/layout side ([`VNode`] tree with bounds set by
//! [`LayoutEngine`]) to the render side (typed [`Command`] sequence
//! in [`UIContext::commands`]). Every frame's render path goes:
//!
//! ```text
//! VNode tree              (already laid out)
//!   └─► build_tree(reg, ctx)
//!         └─► for each node:
//!               ComponentRegistry::resolve(tag, props)
//!                 └─► Component::build(ctx, bounds)
//!                       └─► CommandBuffer pushes
//! ```
//!
//! The walker is **stateless** in this first cut — components are
//! constructed fresh on each call. That keeps the data flow obvious
//! and matches the PRD §5.3 picture exactly. The retained-mode
//! optimisation (cache instances in [`WidgetArena`] by id/key,
//! re-use across frames, fire lifecycle hooks only on tree
//! changes) lands when the dirty-tracker drives builds (todo 13b
//! follow-up integrating §6.4).
//!
//! [`VNode`]: crate::vnode::VNode
//! [`LayoutEngine`]: crate::layout::LayoutEngine
//! [`UIContext::commands`]: crate::context::UIContext::commands
//! [`Command`]: crate::commands::Command
//! [`WidgetArena`]: crate::arena::WidgetArena

use crate::context::UIContext;
use crate::props::{resolve_composite, BindingSource};
use crate::registry::ComponentRegistry;
use crate::vnode::VNode;

/// Walk `tree` and emit draw commands into `ctx`.
///
/// Equivalent to [`build_tree_with`] with an empty
/// [`BindingSource`] — `{{key}}` bindings in props resolve to empty
/// strings. Apps that want live state should call
/// [`build_tree_with`] passing a [`StateStore`].
///
/// [`StateStore`]: crate::state::StateStore
/// [`BindingSource`]: crate::props::BindingSource
pub fn build_tree(tree: &VNode, registry: &ComponentRegistry, ctx: &mut UIContext) {
    let empty = EmptyBindings;
    walk(tree, registry, &empty, ctx);
}

/// Walk `tree` resolving `{{key}}` bindings against `bindings` as it
/// goes. Every prop value is run through `resolve_composite` before
/// being handed to the widget factory, so `<Text value="Hello {{name}}!" />`
/// substitutes from the store at build time (PRD §6.12).
///
/// `node.text_content` is similarly resolved before being injected
/// into the synthetic `value` prop.
pub fn build_tree_with<S: BindingSource>(
    tree: &VNode,
    registry: &ComponentRegistry,
    bindings: &S,
    ctx: &mut UIContext,
) {
    walk(tree, registry, bindings, ctx);
}

fn walk<S: BindingSource>(
    node: &VNode,
    registry: &ComponentRegistry,
    bindings: &S,
    ctx: &mut UIContext,
) {
    // Resolve every prop value through the binding source. Static
    // strings pass through unchanged; `"{{user.name}}"` becomes the
    // current value.
    let mut props = crate::vnode::Props::with_capacity(node.props.len());
    for (k, v) in &node.props {
        props.insert(k.clone(), resolve_composite(v, bindings));
    }

    // Bridge: XML body text reaches widgets via a synthetic `value`
    // prop. Explicit `<Text value="…" />` wins over body text.
    if let Some(text) = &node.text_content {
        props
            .entry("value".to_string())
            .or_insert_with(|| resolve_composite(text, bindings));
    }

    let component = registry.resolve(&node.tag, &props);
    component.build(ctx, node.layout);

    // Children paint on top of their parent — depth-first paint
    // order matches the source order in XML.
    for child in &node.children {
        walk(child, registry, bindings, ctx);
    }
}

/// Internal no-op `BindingSource` used by [`build_tree`] when the
/// caller didn't supply one. Every lookup returns `None`, which
/// `resolve_composite` collapses to empty string per PRD §6.12.
struct EmptyBindings;

impl BindingSource for EmptyBindings {
    fn get_binding(&self, _key: &str) -> Option<String> {
        None
    }
}
