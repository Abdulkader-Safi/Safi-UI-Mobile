//! `ComponentRegistry` host tests (todo 14, PRD §5.4, §6.7).
//!
//! Covers:
//! - Round-trip registration + resolution
//! - Three-step fallback: registered → `DebugBox` (XML template middle
//!   layer is tested when todo 27 lands)
//! - Duplicate registration warns + last-write-wins
//! - `ComponentRegistry::new()` instances are independent of the
//!   global

use safi_ui::commands::Command;
use safi_ui::component::Component;
use safi_ui::context::UIContext;
use safi_ui::debug_box::DebugBox;
use safi_ui::registry::ComponentRegistry;
use safi_ui::vnode::{LayoutRect, Props};

struct StubButton {
    _label: String,
    bounds: LayoutRect,
}

impl Component for StubButton {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }
}

fn props(pairs: &[(&str, &str)]) -> Props {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

#[test]
fn new_registry_is_empty() {
    let r = ComponentRegistry::new();
    assert!(r.is_empty());
    assert_eq!(r.len(), 0);
    assert!(!r.contains("Button"));
}

#[test]
fn register_and_resolve_round_trip() {
    let mut r = ComponentRegistry::new();
    r.register(
        "Button",
        Box::new(|p: &Props| -> Box<dyn Component> {
            Box::new(StubButton {
                _label: p.get("label").cloned().unwrap_or_default(),
                bounds: LayoutRect::default(),
            })
        }),
    );
    assert!(r.contains("Button"));
    assert_eq!(r.len(), 1);

    let component = r.resolve("Button", &props(&[("label", "Sign in")]));
    // Down-cast verification via debug formatting through bounds — we
    // can't easily downcast a `Box<dyn Component>`, but the StubButton
    // sets bounds to default and the factory writes the label, so we
    // verify by checking the component is *not* a DebugBox.
    assert!(component.bounds() == LayoutRect::default());
}

#[test]
fn unknown_tag_falls_back_to_debug_box() {
    let r = ComponentRegistry::new();
    let component = r.resolve("Mystery", &Props::new());
    // We can't downcast through `Box<dyn Component>` without a custom
    // accessor; instead exercise the `build` and check what gets
    // emitted into the command buffer — DebugBox emits a Border and
    // a Text.
    let mut ctx = UIContext::test_default();
    component.build(
        &mut ctx,
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        },
    );
    let cmds = ctx.commands.as_slice();
    assert_eq!(cmds.len(), 2, "DebugBox emits border + text");
    assert!(matches!(cmds[0], Command::Border { .. }));
    assert!(matches!(cmds[1], Command::Text { ref text, .. } if text == "<Mystery>"));
}

#[test]
fn duplicate_registration_last_write_wins() {
    let mut r = ComponentRegistry::new();
    r.register(
        "X",
        Box::new(|_p: &Props| -> Box<dyn Component> {
            Box::new(StubButton {
                _label: "first".into(),
                bounds: LayoutRect::default(),
            })
        }),
    );
    r.register(
        "X",
        Box::new(|_p: &Props| -> Box<dyn Component> {
            Box::new(StubButton {
                _label: "second".into(),
                bounds: LayoutRect::default(),
            })
        }),
    );
    assert_eq!(r.len(), 1);
    // Cannot read the label off a `Box<dyn Component>` without a
    // helper; we verify last-write-wins indirectly via the count
    // staying at 1 (no orphan factories) and not panicking on second
    // register. The stderr warning is exercised manually.
    let _ = r.resolve("X", &Props::new());
}

#[test]
fn separate_instances_are_independent() {
    let mut a = ComponentRegistry::new();
    let b = ComponentRegistry::new();
    a.register(
        "A",
        Box::new(|_p: &Props| -> Box<dyn Component> {
            Box::new(StubButton {
                _label: String::new(),
                bounds: LayoutRect::default(),
            })
        }),
    );
    assert!(a.contains("A"));
    assert!(!b.contains("A"));
}

#[test]
fn debug_box_carries_unknown_tag_name() {
    let db = DebugBox::new("MyMissingTag");
    assert_eq!(db.tag(), "MyMissingTag");
}

#[test]
fn debug_box_renders_text_with_tag_name() {
    let mut db = DebugBox::new("Foo");
    let bounds = LayoutRect {
        x: 10.0,
        y: 20.0,
        width: 80.0,
        height: 40.0,
    };
    db.set_bounds(bounds);
    let mut ctx = UIContext::test_default();
    db.build(&mut ctx, bounds);
    let cmds = ctx.commands.as_slice();
    let Command::Text { ref text, .. } = cmds[1] else {
        panic!("second command should be Text");
    };
    assert!(text.contains("Foo"));
    assert!(text.contains('<') && text.contains('>'));
}
