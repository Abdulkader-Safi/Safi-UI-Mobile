//! `DebugBox` — fallback widget for unknown tags (PRD §5.4, §6.7).
//!
//! When [`ComponentRegistry::resolve`] can't find a factory for a tag,
//! it returns a `DebugBox`. The widget paints a 1dp red outlined
//! rectangle around its bounds and emits a `Command::Text` with the
//! unknown tag name inside, so the developer sees exactly what XML
//! they need to register.
//!
//! `DebugBox` is intentionally placed in its own module (not under
//! `registry`) so it can be referenced by other parts of the system
//! without pulling in the whole registry surface.
//!
//! [`ComponentRegistry::resolve`]: crate::registry::ComponentRegistry::resolve

use glam::Vec2;

use crate::commands::{Color, Command, FontHandle};
use crate::component::Component;
use crate::context::UIContext;
use crate::vnode::LayoutRect;

const OUTLINE_COLOR: Color = Color::rgba(255, 0, 0, 255);
const OUTLINE_THICKNESS: f32 = 1.0;
const TEXT_COLOR: Color = Color::rgba(255, 0, 0, 255);
const TEXT_SIZE_DP: f32 = 12.0;

pub struct DebugBox {
    tag: String,
    bounds: LayoutRect,
}

impl DebugBox {
    /// Build a debug box for the unknown `tag`. Layout populates
    /// bounds before `build` runs.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            bounds: LayoutRect::default(),
        }
    }

    /// Update bounds — called by the runtime between layout and build.
    /// Tests use this directly to set bounds before calling `build`.
    pub fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = bounds;
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }
}

impl Component for DebugBox {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }

    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        ctx.commands.push(Command::Border {
            rect: bounds,
            color: OUTLINE_COLOR,
            radius: 0.0,
            thickness: OUTLINE_THICKNESS,
        });
        // Text positioned 2dp in from the top-left so it doesn't run
        // into the border. Font handle 0 is the default — todo 16
        // (font atlas) replaces this with the real handle.
        ctx.commands.push(Command::Text {
            pos: Vec2::new(bounds.x + 4.0, bounds.y + 4.0),
            text: format!("<{}>", self.tag),
            font: FontHandle::default(),
            size: TEXT_SIZE_DP,
            color: TEXT_COLOR,
        });
    }
}
