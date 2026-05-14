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
use crate::registry::ComponentRegistry;
use crate::vnode::VNode;

/// Walk `tree` and emit draw commands into `ctx`.
///
/// Each node's `tag` is resolved through `registry`; an unknown tag
/// falls through to [`DebugBox`] per PRD §5.4. The walker copies
/// `node.text_content` into a synthetic `value` prop before
/// resolution so widgets that look at `value` (notably [`Text`])
/// see XML body text without needing a special-cased extraction.
///
/// [`DebugBox`]: crate::debug_box::DebugBox
/// [`Text`]: crate::widgets::Text
pub fn build_tree(tree: &VNode, registry: &ComponentRegistry, ctx: &mut UIContext) {
    walk(tree, registry, ctx);
}

fn walk(node: &VNode, registry: &ComponentRegistry, ctx: &mut UIContext) {
    let mut props = node.props.clone();

    // Bridge: XML body text (`<Text>Hello</Text>`) reaches widgets
    // via a synthetic `value` prop. If the prop is already set the
    // explicit form (`<Text value="…" />`) wins.
    if let Some(text) = &node.text_content {
        props
            .entry("value".to_string())
            .or_insert_with(|| text.clone());
    }

    // Resolve and build. `bounds` is populated by `LayoutEngine`
    // before the walker runs — see `App::run`'s frame loop. Tests
    // that build directly off a `vnode!` tree must call
    // `LayoutEngine::compute` first or accept zero-sized commands.
    let component = registry.resolve(&node.tag, &props);
    component.build(ctx, node.layout);

    // Children paint on top of their parent — depth-first paint
    // order matches the source order in XML.
    for child in &node.children {
        walk(child, registry, ctx);
    }
}
