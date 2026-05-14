//! `Component` lifecycle host tests (todo 13, PRD §6.8).
//!
//! Verifies that the trait's default methods do what the spec says
//! (zero-side-effect no-ops) and that override hooks fire as expected.
//! Bounds-change-only `on_layout` is policed at the runtime level (todo
//! 13 wires layout cycle to call `on_layout` only on delta); here we
//! verify the trait contract by overriding hooks and observing the
//! ordering test harness invokes.

use std::cell::Cell;

use glam::Vec2;
use safi_ui::component::Component;
use safi_ui::context::UIContext;
use safi_ui::gestures::Gesture;
use safi_ui::vnode::LayoutRect;

struct Probe {
    mount_count: Cell<u32>,
    unmount_count: Cell<u32>,
    layout_count: Cell<u32>,
    last_bounds: Cell<LayoutRect>,
    bounds: LayoutRect,
}

impl Probe {
    fn new() -> Self {
        Self {
            mount_count: Cell::new(0),
            unmount_count: Cell::new(0),
            layout_count: Cell::new(0),
            last_bounds: Cell::new(LayoutRect::default()),
            bounds: LayoutRect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
            },
        }
    }
}

impl Component for Probe {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }

    fn on_mount(&mut self, _ctx: &mut UIContext) {
        self.mount_count.set(self.mount_count.get() + 1);
    }

    fn on_unmount(&mut self, _ctx: &mut UIContext) {
        self.unmount_count.set(self.unmount_count.get() + 1);
    }

    fn on_layout(&mut self, bounds: LayoutRect) {
        self.layout_count.set(self.layout_count.get() + 1);
        self.last_bounds.set(bounds);
    }
}

#[test]
fn defaults_are_noops() {
    struct Minimal;
    impl Component for Minimal {
        fn bounds(&self) -> LayoutRect {
            LayoutRect::default()
        }
    }
    let mut c = Minimal;
    let mut ctx = UIContext::test_default();
    // None of these should panic or produce side effects we can observe.
    c.on_mount(&mut ctx);
    c.on_unmount(&mut ctx);
    c.on_layout(LayoutRect {
        x: 1.0,
        y: 1.0,
        width: 1.0,
        height: 1.0,
    });
    // `build` is also defaulted to a no-op — should leave the command
    // buffer untouched.
    let before = ctx.commands.len();
    c.build(&mut ctx, LayoutRect::default());
    assert_eq!(ctx.commands.len(), before);
}

#[test]
fn lifecycle_hooks_fire_when_overridden() {
    let mut p = Probe::new();
    let mut ctx = UIContext::test_default();
    p.on_mount(&mut ctx);
    p.on_mount(&mut ctx);
    p.on_unmount(&mut ctx);
    p.on_layout(LayoutRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 2.0,
    });
    assert_eq!(p.mount_count.get(), 2);
    assert_eq!(p.unmount_count.get(), 1);
    assert_eq!(p.layout_count.get(), 1);
    assert_eq!(
        p.last_bounds.get(),
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 2.0,
        }
    );
}

#[test]
fn hit_test_default_is_point_in_bounds() {
    let p = Probe::new();
    assert!(p.hit_test(Vec2::new(50.0, 30.0))); // inside
    assert!(!p.hit_test(Vec2::new(0.0, 0.0))); // outside
    assert!(!p.hit_test(Vec2::new(200.0, 200.0))); // way outside
}

#[test]
fn on_gesture_default_does_not_consume() {
    struct Minimal;
    impl Component for Minimal {
        fn bounds(&self) -> LayoutRect {
            LayoutRect::default()
        }
    }
    let mut c = Minimal;
    assert!(!c.on_gesture(Gesture::Tap {
        pos: Vec2::new(0.0, 0.0)
    }));
}

#[test]
fn trait_is_object_safe() {
    // Compile-time check — if Component lost object-safety
    // (e.g. by gaining a generic method), this would fail to compile.
    fn _take_dyn(_c: &dyn Component) {}
}

#[test]
fn no_send_sync_bound() {
    // PRD §6.8: Component is deliberately not Send + Sync. Demonstrate
    // by holding a non-Send field (Rc) and still implementing the trait.
    use std::rc::Rc;
    struct NotSend {
        _state: Rc<()>,
    }
    impl Component for NotSend {
        fn bounds(&self) -> LayoutRect {
            LayoutRect::default()
        }
    }
    let _c = NotSend {
        _state: Rc::new(()),
    };
}
