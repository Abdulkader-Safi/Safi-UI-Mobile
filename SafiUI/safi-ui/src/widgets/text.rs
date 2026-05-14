//! `Text` widget — base typography (PRD §10.2).
//!
//! Renders a single text run. Supports `{{binding}}` substitution at
//! `build` time so `<Text>{{user.name}}</Text>` works without
//! recomputing the `VNode`.
//!
//! ## Tag-based defaults
//!
//! The same factory backs `Text`, `Heading`, and `Label`. The tag the
//! `VNode` used at parse time is passed in via `from_props(tag, props)`
//! so the widget can pick the right size default:
//!
//! | Tag       | Default size | Style                                      |
//! | --------- | ------------ | ------------------------------------------ |
//! | `Text`    | 14dp         | Regular                                    |
//! | `Heading` | by `level`   | h1=32, h2=24, h3=20, h4=18, h5=16, h6=14   |
//! | `Label`   | 12dp         | Uppercase (string transform at build time) |
//!
//! Explicit `size` / `color` / `weight` props override the defaults.

use glam::Vec2;

use crate::commands::{Command, FontHandle};
use crate::component::Component;
use crate::context::UIContext;
use crate::props::PropsExt;
use crate::vnode::{LayoutRect, Props};
use crate::widgets::view::into_cmd_color;

pub struct Text {
    template: String,
    size: f32,
    color: crate::commands::Color,
    uppercase: bool,
    visible: bool,
    bounds: LayoutRect,
}

impl Text {
    pub fn from_props(tag: &str, props: &Props) -> Self {
        // Resolve size with tag-aware defaults.
        let default_size = default_size_for(tag, props);
        let size = props.parse_f32("size", default_size);

        // Color: default black for Text, gray for Label, near-black
        // for Heading. Apps overriding via `color="…"` win.
        let default_color = match tag {
            "Label" => crate::props::Color::rgb(96, 96, 96),
            _ => crate::props::Color::BLACK,
        };
        let color = into_cmd_color(props.parse_color("color", default_color));

        // The text body sits either in `text_content` (XML body) or
        // an explicit `value="…"` prop. Either form is supported.
        // The `Text` widget itself only sees the template — the VNode
        // hands `text_content` to us via the `value` prop in
        // `register_builtins` (today the parser sets text_content but
        // the widget reads via props; the bridge happens in
        // `App::run`'s build walk when todo 13 lands the build path).
        let template = props
            .get("value")
            .cloned()
            .or_else(|| props.get("text").cloned())
            .unwrap_or_default();

        Self {
            template,
            size,
            color,
            uppercase: tag == "Label",
            visible: props.parse_bool("visible", true),
            bounds: LayoutRect::default(),
        }
    }

    pub fn set_bounds(&mut self, bounds: LayoutRect) {
        self.bounds = bounds;
    }

    /// Resolve any `{{key}}` bindings in the template against
    /// `store`. Plain strings pass through. Apps call this before
    /// `build` if their `BindingSource` (`StateStore`) is available;
    /// otherwise `build` emits the template verbatim with any `{{ }}`
    /// markers intact, which is the right behaviour for previewing.
    pub fn resolve<S: crate::props::BindingSource>(&mut self, store: &S) {
        self.template = crate::props::resolve_composite(&self.template, store);
    }

    fn rendered_text(&self) -> String {
        if self.uppercase {
            self.template.to_uppercase()
        } else {
            self.template.clone()
        }
    }
}

impl Component for Text {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }

    fn build(&self, ctx: &mut UIContext, bounds: LayoutRect) {
        if !self.visible || self.template.is_empty() {
            return;
        }
        // Position the baseline at top-left + 1em padding for now —
        // proper baseline computation lands with todo 16 (font atlas).
        ctx.commands.push(Command::Text {
            pos: Vec2::new(bounds.x, bounds.y + self.size),
            text: self.rendered_text(),
            font: FontHandle::default(),
            size: self.size,
            color: self.color,
        });
    }
}

fn default_size_for(tag: &str, props: &Props) -> f32 {
    match tag {
        "Heading" => {
            // Heading level 1–6 → size table. Clamp first so the
            // f32→u8 cast is in-range; clippy's cast lints fire on
            // every float cast, this one is bounded.
            let raw = props.parse_f32("level", 2.0).clamp(1.0, 6.0);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let level = raw as u8;
            match level {
                1 => 32.0,
                3 => 20.0,
                4 => 18.0,
                5 => 16.0,
                6 => 14.0,
                _ => 24.0, // h2 default
            }
        }
        "Label" => 12.0,
        _ => 14.0,
    }
}
