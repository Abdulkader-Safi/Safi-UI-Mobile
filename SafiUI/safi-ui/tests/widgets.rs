//! `View`, `Text`, `Button` widget host tests (todo 15).
//!
//! Verifies the XML → registry → component → command-buffer pipeline
//! end-to-end on host. Each widget is exercised both directly and via
//! a freshly-built `ComponentRegistry` with `register_builtins`.

use std::collections::HashMap;

use glam::Vec2;
use safi_ui::commands::{Color as CmdColor, Command};
use safi_ui::component::Component;
use safi_ui::context::UIContext;
use safi_ui::gestures::Gesture;
use safi_ui::registry::ComponentRegistry;
use safi_ui::vnode::{LayoutRect, Props};
use safi_ui::widgets::{button::Variant, register_builtins, Button, Text, View};

fn props(pairs: &[(&str, &str)]) -> Props {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn bounds() -> LayoutRect {
    LayoutRect {
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 64.0,
    }
}

// ----- View -------------------------------------------------------------

#[test]
fn view_with_bg_emits_rect() {
    let mut v = View::from_props(&props(&[("bg", "#0f0f1a"), ("radius", "8")]));
    v.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    v.build(&mut ctx, bounds());
    let cmds = ctx.commands.as_slice();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(
        cmds[0],
        Command::Rect { color, radius, .. }
        if color == CmdColor::rgba(0x0f, 0x0f, 0x1a, 255) && (radius - 8.0).abs() < 1e-6
    ));
}

#[test]
fn view_with_border_emits_border_after_rect() {
    let mut v = View::from_props(&props(&[
        ("bg", "#000000"),
        ("border", "#ff0000"),
        ("borderWidth", "2"),
        ("radius", "4"),
    ]));
    v.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    v.build(&mut ctx, bounds());
    let cmds = ctx.commands.as_slice();
    assert_eq!(cmds.len(), 2);
    assert!(matches!(cmds[0], Command::Rect { .. }));
    assert!(matches!(
        cmds[1],
        Command::Border { thickness, .. } if (thickness - 2.0).abs() < 1e-6
    ));
}

#[test]
fn view_visible_false_emits_nothing() {
    let mut v = View::from_props(&props(&[("bg", "#fff"), ("visible", "false")]));
    v.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    v.build(&mut ctx, bounds());
    assert!(ctx.commands.as_slice().is_empty());
}

#[test]
fn view_opacity_modulates_bg_alpha() {
    let mut v = View::from_props(&props(&[("bg", "#ffffff"), ("opacity", "0.5")]));
    v.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    v.build(&mut ctx, bounds());
    let Command::Rect { color, .. } = ctx.commands.as_slice()[0] else {
        panic!("expected Rect");
    };
    assert_eq!(color.a, 128); // 255 * 0.5 ≈ 128
}

#[test]
fn view_without_bg_emits_no_rect() {
    let mut v = View::from_props(&Props::new());
    v.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    v.build(&mut ctx, bounds());
    assert!(ctx.commands.as_slice().is_empty());
}

// ----- Text -------------------------------------------------------------

#[test]
fn text_emits_text_command_with_value() {
    let mut t = Text::from_props("Text", &props(&[("value", "Hello"), ("size", "18")]));
    t.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    t.build(&mut ctx, bounds());
    let cmds = ctx.commands.as_slice();
    assert_eq!(cmds.len(), 1);
    let Command::Text { ref text, size, .. } = cmds[0] else {
        panic!("expected Text command");
    };
    assert_eq!(text, "Hello");
    assert!((size - 18.0).abs() < 1e-6);
}

#[test]
fn text_resolves_binding_before_build() {
    let mut t = Text::from_props("Text", &props(&[("value", "Hi {{name}}!")]));
    let mut store: HashMap<String, String> = HashMap::new();
    store.insert("name".to_string(), "Safi".to_string());
    t.resolve(&store);

    let mut ctx = UIContext::test_default();
    t.build(&mut ctx, bounds());
    let Command::Text { ref text, .. } = ctx.commands.as_slice()[0] else {
        panic!("expected Text");
    };
    assert_eq!(text, "Hi Safi!");
}

#[test]
fn text_empty_template_emits_nothing() {
    let mut t = Text::from_props("Text", &Props::new());
    t.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    t.build(&mut ctx, bounds());
    assert!(ctx.commands.as_slice().is_empty());
}

#[test]
fn heading_level_sets_default_size() {
    let t = Text::from_props("Heading", &props(&[("level", "1"), ("value", "Big")]));
    let mut ctx = UIContext::test_default();
    let mut t_owned = t;
    t_owned.set_bounds(bounds());
    t_owned.build(&mut ctx, bounds());
    let Command::Text { size, .. } = ctx.commands.as_slice()[0] else {
        panic!("expected Text");
    };
    assert!(
        (size - 32.0).abs() < 1e-6,
        "h1 should default to 32dp, got {size}"
    );
}

#[test]
fn label_uppercases_text() {
    let mut t = Text::from_props("Label", &props(&[("value", "submit")]));
    t.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    t.build(&mut ctx, bounds());
    let Command::Text { ref text, .. } = ctx.commands.as_slice()[0] else {
        panic!();
    };
    assert_eq!(text, "SUBMIT");
}

// ----- Button -----------------------------------------------------------

#[test]
fn button_emits_rect_then_text() {
    let mut b = Button::from_props(&props(&[("label", "Sign in"), ("onPress", "auth.login")]));
    b.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    b.build(&mut ctx, bounds());
    let cmds = ctx.commands.as_slice();
    assert_eq!(cmds.len(), 2);
    assert!(matches!(cmds[0], Command::Rect { .. }));
    let Command::Text { ref text, .. } = cmds[1] else {
        panic!();
    };
    assert_eq!(text, "Sign in");
}

#[test]
fn button_ghost_variant_emits_border_not_rect() {
    let mut b = Button::from_props(&props(&[("variant", "ghost"), ("label", "X")]));
    b.set_bounds(bounds());
    let mut ctx = UIContext::test_default();
    b.build(&mut ctx, bounds());
    let cmds = ctx.commands.as_slice();
    // Ghost: border + text, no rect.
    assert_eq!(cmds.len(), 2);
    assert!(matches!(cmds[0], Command::Border { .. }));
    assert!(matches!(cmds[1], Command::Text { .. }));
}

#[test]
fn button_tap_inside_bounds_captures_event() {
    let mut b = Button::from_props(&props(&[("label", "X"), ("onPress", "demo.tap")]));
    b.set_bounds(bounds());
    let consumed = b.on_gesture(Gesture::Tap {
        pos: Vec2::new(50.0, 40.0), // inside `bounds()`
    });
    assert!(consumed);
    assert_eq!(b.take_pending_event().as_deref(), Some("demo.tap"));
}

#[test]
fn button_tap_outside_bounds_passes_through() {
    let mut b = Button::from_props(&props(&[("label", "X"), ("onPress", "demo.tap")]));
    b.set_bounds(bounds());
    let consumed = b.on_gesture(Gesture::Tap {
        pos: Vec2::new(500.0, 500.0),
    });
    assert!(!consumed);
    assert!(b.take_pending_event().is_none());
}

#[test]
fn button_disabled_does_not_fire() {
    let mut b = Button::from_props(&props(&[
        ("label", "X"),
        ("onPress", "demo.tap"),
        ("disabled", "true"),
    ]));
    b.set_bounds(bounds());
    let consumed = b.on_gesture(Gesture::Tap {
        pos: Vec2::new(50.0, 40.0),
    });
    assert!(!consumed);
    assert!(b.take_pending_event().is_none());
}

#[test]
fn button_variant_parses() {
    let primary = Button::from_props(&props(&[("variant", "primary")]));
    let secondary = Button::from_props(&props(&[("variant", "secondary")]));
    let ghost = Button::from_props(&props(&[("variant", "ghost")]));
    let danger = Button::from_props(&props(&[("variant", "danger")]));
    let default_p = Button::from_props(&Props::new());
    assert_eq!(primary.variant(), Variant::Primary);
    assert_eq!(secondary.variant(), Variant::Secondary);
    assert_eq!(ghost.variant(), Variant::Ghost);
    assert_eq!(danger.variant(), Variant::Danger);
    assert_eq!(default_p.variant(), Variant::Primary);
}

// ----- register_builtins ------------------------------------------------

#[test]
fn register_builtins_wires_all_canonical_tags() {
    let mut r = ComponentRegistry::new();
    register_builtins(&mut r);
    for tag in [
        "Screen", "View", "Row", "Column", "Stack", "Spacer", "Text", "Heading", "Label", "Button",
    ] {
        assert!(r.contains(tag), "registry should contain <{tag}>");
    }
}

#[test]
fn registry_resolves_unknown_tag_to_debug_box() {
    let mut r = ComponentRegistry::new();
    register_builtins(&mut r);
    let unknown = r.resolve("Mystery", &Props::new());
    let mut ctx = UIContext::test_default();
    unknown.build(&mut ctx, bounds());
    let cmds = ctx.commands.as_slice();
    // DebugBox emits Border + Text("<Mystery>")
    assert_eq!(cmds.len(), 2);
    assert!(matches!(cmds[0], Command::Border { .. }));
    let Command::Text { ref text, .. } = cmds[1] else {
        panic!("expected Text");
    };
    assert!(text.contains("Mystery"));
}
