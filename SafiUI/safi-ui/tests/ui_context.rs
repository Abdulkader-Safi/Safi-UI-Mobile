use safi_ui::arena::{InsertSpec, WidgetArena, WidgetId};
use safi_ui::commands::{Command, Rect};
use safi_ui::component::Component;
use safi_ui::context::UIContext;
use safi_ui::dirty::SizingMode;
use safi_ui::edge_insets::EdgeInsets;
use safi_ui::focus::FocusSystem;
use safi_ui::vnode::LayoutRect;

struct TestWidget;
impl Component for TestWidget {
    fn bounds(&self) -> LayoutRect {
        LayoutRect::default()
    }
}

fn boxed() -> Box<dyn Component> {
    Box::new(TestWidget)
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

fn arena_with_two_resolved_roots(ctx: &mut UIContext) -> (WidgetArena, WidgetId, WidgetId) {
    let mut a = WidgetArena::new();
    let id_a = a.insert(InsertSpec {
        widget: boxed(),
        parent: None,
    });
    let id_b = a.insert(InsertSpec {
        widget: boxed(),
        parent: None,
    });
    ctx.dirty.set_sizing(id_a, SizingMode::Resolved);
    ctx.dirty.set_sizing(id_b, SizingMode::Resolved);
    (a, id_a, id_b)
}

// ---- UIContext::test_default ----

#[test]
fn test_default_initialises_fields() {
    let ctx = UIContext::test_default();
    assert_eq!(ctx.dpi_scale.to_bits(), 1.0_f32.to_bits());
    assert_eq!(ctx.safe_area, EdgeInsets::ZERO);
    assert!(ctx.commands.is_empty());
    assert!(!ctx.dirty.needs_redraw());
    assert_eq!(ctx.focus.owner(), None);
    assert_eq!(ctx.clips.depth(), 0);
}

// ---- ClipStack via UIContext ----

#[test]
fn push_clip_emits_clip_command_and_grows_depth() {
    let mut ctx = UIContext::test_default();
    let r = rect(0.0, 0.0, 100.0, 100.0);
    ctx.push_clip(r);
    assert_eq!(ctx.clips.depth(), 1);
    assert_eq!(ctx.commands.len(), 1);
    assert_eq!(ctx.commands.as_slice()[0], Command::Clip { rect: r });
}

#[test]
fn pop_clip_emits_clippop_command_and_shrinks_depth() {
    let mut ctx = UIContext::test_default();
    ctx.push_clip(rect(0.0, 0.0, 50.0, 50.0));
    ctx.pop_clip();
    assert_eq!(ctx.clips.depth(), 0);
    assert_eq!(ctx.commands.as_slice()[1], Command::ClipPop);
}

#[test]
fn nested_clips_balance() {
    let mut ctx = UIContext::test_default();
    let r1 = rect(0.0, 0.0, 100.0, 100.0);
    let r2 = rect(10.0, 10.0, 80.0, 80.0);
    ctx.push_clip(r1);
    ctx.push_clip(r2);
    assert_eq!(ctx.clips.top(), Some(r2));
    ctx.pop_clip();
    ctx.pop_clip();

    assert_eq!(ctx.clips.depth(), 0);
    let cmds = ctx.commands.as_slice();
    assert_eq!(cmds.len(), 4);
    assert!(matches!(cmds[0], Command::Clip { .. }));
    assert!(matches!(cmds[1], Command::Clip { .. }));
    assert_eq!(cmds[2], Command::ClipPop);
    assert_eq!(cmds[3], Command::ClipPop);
}

#[cfg(debug_assertions)]
#[test]
fn pop_clip_on_empty_panics_in_debug() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut ctx = UIContext::test_default();
        ctx.pop_clip();
    }));
    assert!(result.is_err(), "pop_clip on empty must panic in debug");
}

// ---- FocusSystem ----

#[test]
fn set_owner_returns_previous() {
    let mut f = FocusSystem::new();
    assert_eq!(f.set_owner(Some(1)), None);
    assert_eq!(f.set_owner(Some(2)), Some(1));
    assert_eq!(f.set_owner(None), Some(2));
}

#[test]
fn tab_order_round_trip() {
    let mut f = FocusSystem::new();
    f.register_tabbable(1);
    f.register_tabbable(2);
    f.register_tabbable(3);
    f.register_tabbable(2); // dedup no-op
    assert_eq!(f.tab_order(), &[1, 2, 3]);
    f.unregister_tabbable(2);
    assert_eq!(f.tab_order(), &[1, 3]);
}

#[test]
fn next_in_tab_order_wraps() {
    let mut f = FocusSystem::new();
    f.register_tabbable(10);
    f.register_tabbable(20);
    f.register_tabbable(30);
    assert_eq!(f.next_in_tab_order(), Some(10), "no owner -> first");
    f.set_owner(Some(30));
    assert_eq!(f.next_in_tab_order(), Some(10), "wrap to first");
    f.set_owner(Some(10));
    assert_eq!(f.next_in_tab_order(), Some(20));
}

#[test]
fn prev_in_tab_order_wraps() {
    let mut f = FocusSystem::new();
    f.register_tabbable(10);
    f.register_tabbable(20);
    f.register_tabbable(30);
    assert_eq!(f.prev_in_tab_order(), Some(30), "no owner -> last");
    f.set_owner(Some(10));
    assert_eq!(f.prev_in_tab_order(), Some(30), "wrap to last");
    f.set_owner(Some(20));
    assert_eq!(f.prev_in_tab_order(), Some(10));
}

#[test]
fn tab_order_on_empty_returns_none() {
    let f = FocusSystem::new();
    assert_eq!(f.next_in_tab_order(), None);
    assert_eq!(f.prev_in_tab_order(), None);
}

// ---- Focus -> dirty cascade (acceptance #2) ----

#[test]
fn request_focus_marks_new_dirty() {
    let mut ctx = UIContext::test_default();
    let (arena, a, _b) = arena_with_two_resolved_roots(&mut ctx);
    ctx.request_focus(&arena, Some(a));
    assert!(ctx.dirty.is_dirty(a));
}

#[test]
fn request_focus_marks_previous_and_new_dirty() {
    let mut ctx = UIContext::test_default();
    let (arena, a, b) = arena_with_two_resolved_roots(&mut ctx);
    ctx.request_focus(&arena, Some(a));
    ctx.dirty.on_frame_complete();
    ctx.request_focus(&arena, Some(b));
    assert!(ctx.dirty.is_dirty(a), "previous owner dirty");
    assert!(ctx.dirty.is_dirty(b), "new owner dirty");
}

#[test]
fn clear_focus_marks_previous_dirty() {
    let mut ctx = UIContext::test_default();
    let (arena, a, _b) = arena_with_two_resolved_roots(&mut ctx);
    ctx.request_focus(&arena, Some(a));
    ctx.dirty.on_frame_complete();
    ctx.clear_focus(&arena);
    assert!(ctx.dirty.is_dirty(a));
    assert_eq!(ctx.focus.owner(), None);
}

#[test]
fn request_focus_same_widget_is_dirty_once() {
    let mut ctx = UIContext::test_default();
    let (arena, a, _b) = arena_with_two_resolved_roots(&mut ctx);
    ctx.request_focus(&arena, Some(a));
    ctx.dirty.on_frame_complete();
    ctx.request_focus(&arena, Some(a));
    let count = ctx.dirty.dirty_widgets().count();
    assert_eq!(count, 1);
}

#[test]
fn request_focus_with_unknown_id_does_not_dirty() {
    let mut ctx = UIContext::test_default();
    let arena = WidgetArena::new();
    ctx.request_focus(&arena, Some(9999));
    assert_eq!(ctx.focus.owner(), Some(9999));
    assert!(!ctx.dirty.needs_redraw());
}

// ---- EdgeInsets helpers ----

#[test]
fn edge_insets_all_and_symmetric() {
    let a = EdgeInsets::all(8.0);
    assert_eq!(
        a,
        EdgeInsets {
            top: 8.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0,
        }
    );
    let s = EdgeInsets::symmetric(4.0, 8.0);
    assert_eq!(
        s,
        EdgeInsets {
            top: 8.0,
            right: 4.0,
            bottom: 8.0,
            left: 4.0,
        }
    );
    assert_eq!(EdgeInsets::ZERO, EdgeInsets::default());
}
