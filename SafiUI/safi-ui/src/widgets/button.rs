//! `Button` widget — tappable rect+label (PRD §10.3).
//!
//! Renders a rounded background, optional border, and centered label.
//! Variants (`primary`, `secondary`, `ghost`, `danger`) map to
//! sensible default color palettes — apps override by setting `bg`,
//! `color`, `border` explicitly.
//!
//! ## Tap dispatch
//!
//! On a [`Gesture::Tap`] inside bounds, the button captures the
//! gesture (`on_gesture` returns `true`) and **records** the event
//! name. The runtime drains pending events in the frame loop and
//! forwards them to the `EventBus` once that lands (todo 24). Until
//! then, taps log to stderr so device verification still confirms
//! the gesture is firing.

use glam::Vec2;

use crate::commands::{Color as CmdColor, Command, FontHandle};
use crate::component::Component;
use crate::context::UIContext;
use crate::gestures::Gesture;
use crate::props::PropsExt;
use crate::vnode::{LayoutRect, Props};
use crate::widgets::view::into_cmd_color;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Variant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

impl Variant {
    fn from_str(s: &str) -> Self {
        match s {
            "secondary" => Self::Secondary,
            "ghost" => Self::Ghost,
            "danger" => Self::Danger,
            _ => Self::Primary,
        }
    }

    fn bg(self) -> CmdColor {
        match self {
            Self::Primary => CmdColor::rgb(0x4f, 0x8e, 0xf7),
            Self::Secondary => CmdColor::rgb(0x2a, 0x2a, 0x3e),
            Self::Ghost => CmdColor::TRANSPARENT,
            Self::Danger => CmdColor::rgb(0xe7, 0x4c, 0x3c),
        }
    }

    fn fg(self) -> CmdColor {
        match self {
            Self::Ghost => CmdColor::rgb(0x4f, 0x8e, 0xf7),
            _ => CmdColor::WHITE,
        }
    }

    fn border(self) -> Option<CmdColor> {
        match self {
            Self::Ghost => Some(CmdColor::rgb(0x4f, 0x8e, 0xf7)),
            _ => None,
        }
    }
}

pub struct Button {
    label: String,
    on_press: String,
    variant: Variant,
    disabled: bool,
    bg: CmdColor,
    fg: CmdColor,
    border: Option<CmdColor>,
    radius: f32,
    visible: bool,
    bounds: LayoutRect,
    pending_event: Option<String>,
}

impl Button {
    pub fn from_props(props: &Props) -> Self {
        let variant = Variant::from_str(&props.get_str("variant", "primary"));
        // Explicit overrides win over variant defaults.
        let bg = props
            .get("bg")
            .and_then(|s| crate::props::parse_color_str(s))
            .map_or_else(|| variant.bg(), into_cmd_color);
        let fg = props
            .get("color")
            .and_then(|s| crate::props::parse_color_str(s))
            .map_or_else(|| variant.fg(), into_cmd_color);
        let border = props
            .get("border")
            .and_then(|s| crate::props::parse_color_str(s))
            .map(into_cmd_color)
            .or_else(|| variant.border());
        Self {
            label: props.get_str("label", "Button"),
            on_press: props.get_str("onPress", ""),
            variant,
            disabled: props.parse_bool("disabled", false),
            bg,
            fg,
            border,
            radius: props.parse_f32("radius", 8.0),
            visible: props.parse_bool("visible", true),
            bounds: LayoutRect::default(),
            pending_event: None,
        }
    }

    pub fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = bounds;
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_press_event(&self) -> &str {
        &self.on_press
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// Take the pending event name, if any. The runtime drains this
    /// once per frame and dispatches via `EventBus` (todo 24).
    pub fn take_pending_event(&mut self) -> Option<String> {
        self.pending_event.take()
    }
}

impl Component for Button {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }

    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        if !self.visible {
            return;
        }
        let bg = if self.disabled { dim(self.bg) } else { self.bg };
        let fg = if self.disabled { dim(self.fg) } else { self.fg };

        if self.variant != Variant::Ghost {
            ctx.commands.push(Command::Rect {
                rect: bounds,
                color: bg,
                radius: self.radius,
            });
        }
        if let Some(b) = self.border {
            ctx.commands.push(Command::Border {
                rect: bounds,
                color: if self.disabled { dim(b) } else { b },
                radius: self.radius,
                thickness: 1.5,
            });
        }

        // Centered label. Vertical center: bounds.y + height/2 +
        // size*0.35 (approximate ascender). Real text metrics land
        // with todo 16; this is the smoke approximation.
        let size = 14.0;
        let pos = Vec2::new(
            bounds.x + 16.0,
            bounds.y + (bounds.height + size) * 0.5 - 2.0,
        );
        ctx.commands.push(Command::Text {
            pos,
            text: self.label.clone(),
            font: FontHandle::default(),
            size,
            color: fg,
        });
    }

    fn on_gesture(&mut self, gesture: Gesture) -> bool {
        if self.disabled || !self.visible {
            return false;
        }
        if let Gesture::Tap { pos } = gesture {
            if self.bounds.contains(pos) && !self.on_press.is_empty() {
                eprintln!(
                    "safi-ui::Button: tap fired '{}' (todo 24 wires real EventBus dispatch)",
                    self.on_press
                );
                self.pending_event = Some(self.on_press.clone());
                return true;
            }
        }
        false
    }
}

fn dim(c: CmdColor) -> CmdColor {
    CmdColor::rgba(c.r, c.g, c.b, c.a / 2)
}
