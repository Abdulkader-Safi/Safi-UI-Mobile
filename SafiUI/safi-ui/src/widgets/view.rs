//! `View` widget — generic box container (PRD §10.1).
//!
//! The most fundamental built-in. Used as the factory for every
//! "box-shaped" tag (`Screen`, `View`, `Row`, `Column`, `Stack`,
//! `Spacer`) — the visual surface is identical, the only difference
//! is what `LayoutEngine` does with the tag-derived flex direction.
//!
//! ## Props
//!
//! - `bg` — background color (any [`parse_color_str`] variant)
//! - `radius` — corner radius in dp
//! - `border` — border color; thickness pulled from `borderWidth` (dp)
//! - `borderWidth` — border thickness in dp (default 0 = no border)
//! - `opacity` — 0.0..=1.0 multiplier on `bg` alpha
//! - `visible` — boolean; when false, `build` is a no-op (no paint,
//!   no children paint) but layout space and instance state stay
//!
//! [`parse_color_str`]: crate::props::parse_color_str

use crate::commands::{Color as CmdColor, Command};
use crate::component::Component;
use crate::context::UIContext;
use crate::props::{Color as PropColor, PropsExt};
use crate::vnode::{LayoutRect, Props};

pub struct View {
    bg: Option<CmdColor>,
    radius: f32,
    border: Option<CmdColor>,
    border_width: f32,
    opacity: f32,
    visible: bool,
    bounds: LayoutRect,
}

impl View {
    pub fn from_props(props: &Props) -> Self {
        let bg = props
            .get("bg")
            .and_then(|s| crate::props::parse_color_str(s))
            .map(into_cmd_color);
        let border = props
            .get("border")
            .and_then(|s| crate::props::parse_color_str(s))
            .map(into_cmd_color);
        Self {
            bg,
            radius: props.parse_f32("radius", 0.0),
            border,
            border_width: props.parse_f32("borderWidth", 0.0),
            opacity: props.parse_f32("opacity", 1.0).clamp(0.0, 1.0),
            visible: props.parse_bool("visible", true),
            bounds: LayoutRect::default(),
        }
    }

    pub fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = bounds;
    }

    fn alpha_mod(&self, c: CmdColor) -> CmdColor {
        // Multiply the channel-α by opacity. Clamped because parse
        // already clamped opacity, but f32 → u8 cast needs an
        // in-range guarantee.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let a = (f32::from(c.a) * self.opacity).clamp(0.0, 255.0).round() as u8;
        CmdColor { a, ..c }
    }
}

impl Component for View {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }

    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        if !self.visible {
            return;
        }
        if let Some(c) = self.bg {
            ctx.commands.push(Command::Rect {
                rect: bounds,
                color: self.alpha_mod(c),
                radius: self.radius,
            });
        }
        if let Some(c) = self.border {
            if self.border_width > 0.0 {
                ctx.commands.push(Command::Border {
                    rect: bounds,
                    color: self.alpha_mod(c),
                    radius: self.radius,
                    thickness: self.border_width,
                });
            }
        }
    }
}

/// Convert from the prop-side color type to the renderer-side
/// command color. Identical layout but the types live in different
/// modules — props for parsing, commands for the GPU pipeline.
pub(crate) fn into_cmd_color(p: PropColor) -> CmdColor {
    CmdColor::rgba(p.r, p.g, p.b, p.a)
}
