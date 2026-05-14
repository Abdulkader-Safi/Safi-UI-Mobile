//! End-to-end pipeline test (todo 15 + build walker).
//!
//! Proves the end-user flow: write XML → it parses → layout
//! computes bounds → registry resolves tags → widgets emit the
//! right Command sequence. This is the "framework is usable" gate.
//!
//! NOTE on raw string literals: hex color values like `"#000000"`
//! collide with the `r#"…"#` close delimiter. We use `r##"…"##`
//! throughout this file so the `"#` inside XML is not interpreted
//! as the close.

#![allow(clippy::needless_raw_string_hashes)]

use safi_ui::build::build_tree;
use safi_ui::commands::Command;
use safi_ui::context::UIContext;
use safi_ui::layout::LayoutEngine;
use safi_ui::parse::XmlParser;
use safi_ui::registry::ComponentRegistry;
use safi_ui::widgets::register_builtins;
use taffy::{AvailableSpace, Size};

fn definite(w: f32, h: f32) -> Size<AvailableSpace> {
    Size {
        width: AvailableSpace::Definite(w),
        height: AvailableSpace::Definite(h),
    }
}

fn build_pipeline(xml: &str) -> Vec<Command> {
    let mut tree = XmlParser::parse_str(xml, "test.xml").expect("parse");
    let mut layout = LayoutEngine::new();
    layout.compute(&mut tree, definite(480.0, 800.0));

    let mut registry = ComponentRegistry::new();
    register_builtins(&mut registry);

    let mut ctx = UIContext::test_default();
    build_tree(&tree, &registry, &mut ctx);
    ctx.commands.as_slice().to_vec()
}

#[test]
fn screen_with_bg_emits_one_rect() {
    let cmds = build_pipeline(r##"<Screen bg="#0f0f1a" width="100%" height="100%" />"##);
    assert_eq!(cmds.len(), 1);
    let Command::Rect { color, .. } = cmds[0] else {
        panic!("expected Rect");
    };
    assert_eq!(color, safi_ui::commands::Color::rgba(0x0f, 0x0f, 0x1a, 255));
}

#[test]
fn screen_with_button_emits_rect_rect_text() {
    let cmds = build_pipeline(
        r##"<Screen bg="#000000" width="100%" height="100%">
            <Button label="Sign in" onPress="auth.login" />
        </Screen>"##,
    );
    assert!(cmds.len() >= 3, "got {} commands", cmds.len());
    assert!(matches!(cmds[0], Command::Rect { .. }), "screen rect");
    assert!(matches!(cmds[1], Command::Rect { .. }), "button rect");
    let Command::Text { ref text, .. } = cmds[2] else {
        panic!("expected button label Text");
    };
    assert_eq!(text, "Sign in");
}

#[test]
fn nested_layout_renders_depth_first() {
    let cmds = build_pipeline(
        r##"<Screen bg="#101010" width="100%" height="100%">
            <Column>
                <View bg="#27ae60" width="200" height="50" />
                <View bg="#e74c3c" width="200" height="50" />
            </Column>
        </Screen>"##,
    );
    let rects: Vec<_> = cmds
        .iter()
        .filter_map(|c| match c {
            Command::Rect { color, .. } => Some(*color),
            _ => None,
        })
        .collect();
    assert_eq!(rects.len(), 3);
    assert_eq!(
        rects[1],
        safi_ui::commands::Color::rgba(0x27, 0xae, 0x60, 255)
    );
    assert_eq!(
        rects[2],
        safi_ui::commands::Color::rgba(0xe7, 0x4c, 0x3c, 255)
    );
}

#[test]
fn unknown_tag_falls_back_to_debug_box() {
    let cmds = build_pipeline(
        r##"<Screen bg="#000000" width="100%" height="100%">
            <Mystery />
        </Screen>"##,
    );
    assert!(cmds.iter().any(|c| matches!(c, Command::Border { .. })));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::Text { text, .. } if text.contains("Mystery"))));
}

#[test]
fn text_body_content_resolves_via_value_prop() {
    let cmds = build_pipeline(
        r##"<Screen bg="#000000" width="100%" height="100%">
            <Text size="18" color="#ffffff">Hello, Safi-UI</Text>
        </Screen>"##,
    );
    let Some(Command::Text { text, .. }) = cmds
        .iter()
        .find(|c| matches!(c, Command::Text { text, .. } if text == "Hello, Safi-UI"))
    else {
        panic!("expected text 'Hello, Safi-UI', got {cmds:#?}");
    };
    assert_eq!(text, "Hello, Safi-UI");
}

#[test]
fn explicit_value_prop_wins_over_body_text() {
    let cmds = build_pipeline(
        r##"<Screen width="100%" height="100%">
            <Text value="EXPLICIT" />
        </Screen>"##,
    );
    let Some(Command::Text { text, .. }) = cmds.iter().find(|c| matches!(c, Command::Text { .. }))
    else {
        panic!("expected Text command");
    };
    assert_eq!(text, "EXPLICIT");
}
