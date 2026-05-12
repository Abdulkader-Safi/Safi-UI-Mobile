use std::cell::RefCell;

use glam::Vec2;
use safi_ui::arena::{InsertSpec, WidgetArena, WidgetId};
use safi_ui::component::Component;
use safi_ui::gestures::{
    Gesture, GestureRecognizer, SwipeDirection, SWIPE_VELOCITY_THRESHOLD_DP_PER_SEC,
};
use safi_ui::vnode::LayoutRect;

struct TestWidget {
    bounds: LayoutRect,
    received: RefCell<Vec<Gesture>>,
    consume: bool,
}

impl TestWidget {
    fn new(bounds: LayoutRect, consume: bool) -> Self {
        Self {
            bounds,
            received: RefCell::new(Vec::new()),
            consume,
        }
    }
}

impl Component for TestWidget {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }

    fn on_gesture(&mut self, g: Gesture) -> bool {
        self.received.borrow_mut().push(g);
        self.consume
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> LayoutRect {
    LayoutRect {
        x,
        y,
        width: w,
        height: h,
    }
}

fn arena_one(bounds: LayoutRect, consume: bool) -> (WidgetArena, WidgetId) {
    let mut a = WidgetArena::new();
    let id = a.insert(InsertSpec {
        widget: Box::new(TestWidget::new(bounds, consume)),
        parent: None,
    });
    (a, id)
}

fn approx_eq(a: f32, b: f32, tolerance_pct: f32) -> bool {
    (a - b).abs() <= b.abs() * tolerance_pct
}

// ---- boundary timing: tap vs long-press ----

#[test]
fn tap_at_199ms_fires_tap() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_up(1, Vec2::ZERO, 199);

    let pending: Vec<_> = r.pending().copied().collect();
    assert_eq!(pending.len(), 1);
    assert!(matches!(pending[0].0, Gesture::Tap { .. }));
}

#[test]
fn up_at_201ms_fires_nothing() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_up(1, Vec2::ZERO, 201);
    assert_eq!(r.pending().count(), 0);
}

#[test]
fn long_press_at_499ms_tick_does_not_fire() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.tick(499);
    assert_eq!(r.pending().count(), 0);
}

#[test]
fn long_press_at_501ms_tick_fires() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.tick(501);
    let pending: Vec<_> = r.pending().copied().collect();
    assert_eq!(pending.len(), 1);
    assert!(matches!(pending[0].0, Gesture::LongPress { .. }));
}

#[test]
fn long_press_fires_only_once() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.tick(501);
    r.tick(700);
    let count = r
        .pending()
        .filter(|(g, _)| matches!(g, Gesture::LongPress { .. }))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn long_press_then_up_emits_no_tap() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.tick(600);
    r.finger_up(1, Vec2::ZERO, 700);
    let kinds: Vec<_> = r
        .pending()
        .map(|(g, _)| std::mem::discriminant(g))
        .collect();
    assert_eq!(kinds.len(), 1);
    assert!(matches!(
        r.pending().next().unwrap().0,
        Gesture::LongPress { .. }
    ));
}

// ---- pan / swipe ----

#[test]
fn motion_under_threshold_does_not_pan() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_motion(1, Vec2::new(5.0, 0.0), 50);
    assert_eq!(r.pending().count(), 0);
}

#[test]
fn motion_over_threshold_emits_pan_for_each_subsequent_motion() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_motion(1, Vec2::new(15.0, 0.0), 50);
    r.finger_motion(1, Vec2::new(20.0, 0.0), 100);
    let pans = r
        .pending()
        .filter(|(g, _)| matches!(g, Gesture::Pan { .. }))
        .count();
    assert_eq!(pans, 2);
}

#[test]
fn up_during_slow_pan_emits_no_swipe() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    // Cross threshold slowly: 15dp over 500ms = 30 dp/s (well below swipe).
    r.finger_motion(1, Vec2::new(15.0, 0.0), 500);
    r.finger_up(1, Vec2::new(15.0, 0.0), 600);
    let swipes = r
        .pending()
        .filter(|(g, _)| matches!(g, Gesture::Swipe { .. }))
        .count();
    assert_eq!(swipes, 0);
}

#[test]
fn fast_pan_release_emits_swipe_right() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    // 100 dp in 100 ms = 1000 dp/s, above 800 threshold.
    r.finger_motion(1, Vec2::new(100.0, 0.0), 100);
    r.finger_up(1, Vec2::new(100.0, 0.0), 110);
    let (last_gesture, _) = r.pending().last().expect("at least one gesture");
    match last_gesture {
        Gesture::Swipe {
            direction,
            velocity,
        } => {
            assert_eq!(*direction, SwipeDirection::Right);
            assert!(
                *velocity >= SWIPE_VELOCITY_THRESHOLD_DP_PER_SEC,
                "velocity {velocity} should be >= {SWIPE_VELOCITY_THRESHOLD_DP_PER_SEC}"
            );
        }
        other => panic!("expected Swipe, got {other:?}"),
    }
}

#[test]
fn swipe_direction_picks_dominant_axis() {
    // Right: (50, 5)
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_motion(1, Vec2::new(50.0, 5.0), 50);
    r.finger_up(1, Vec2::new(50.0, 5.0), 60);
    let swipe = r
        .pending()
        .find(|(g, _)| matches!(g, Gesture::Swipe { .. }));
    if let Some((Gesture::Swipe { direction, .. }, _)) = swipe {
        assert_eq!(*direction, SwipeDirection::Right);
    } else {
        panic!("expected Right swipe");
    }

    // Down: (5, 50)
    let mut r = GestureRecognizer::new();
    r.finger_down(2, Vec2::ZERO, 0);
    r.finger_motion(2, Vec2::new(5.0, 50.0), 50);
    r.finger_up(2, Vec2::new(5.0, 50.0), 60);
    let swipe = r
        .pending()
        .find(|(g, _)| matches!(g, Gesture::Swipe { .. }));
    if let Some((Gesture::Swipe { direction, .. }, _)) = swipe {
        assert_eq!(*direction, SwipeDirection::Down);
    } else {
        panic!("expected Down swipe");
    }
}

// ---- velocity accuracy (≤ 5%) ----

#[test]
fn velocity_matches_synthetic_input_within_5_percent() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_motion(1, Vec2::new(100.0, 0.0), 100);
    let pan = r
        .pending()
        .find(|(g, _)| matches!(g, Gesture::Pan { .. }))
        .expect("pan");
    if let (Gesture::Pan { velocity, .. }, _) = pan {
        assert!(
            approx_eq(velocity.x, 1000.0, 0.05),
            "velocity.x = {} not within 5% of 1000",
            velocity.x
        );
        assert_eq!(velocity.y.to_bits(), 0.0_f32.to_bits());
    }
}

// ---- reverse-Z hit test ----

struct LoggingWidget {
    bounds: LayoutRect,
    name: &'static str,
    log: std::rc::Rc<RefCell<Vec<&'static str>>>,
}
impl Component for LoggingWidget {
    fn bounds(&self) -> LayoutRect {
        self.bounds
    }
    fn on_gesture(&mut self, _g: Gesture) -> bool {
        self.log.borrow_mut().push(self.name);
        true
    }
}

#[test]
fn deepest_widget_wins_hit_test() {
    use std::rc::Rc;

    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));

    let mut arena = WidgetArena::new();
    let root = arena.insert(InsertSpec {
        widget: Box::new(LoggingWidget {
            bounds: rect(0.0, 0.0, 100.0, 100.0),
            name: "root",
            log: log.clone(),
        }),
        parent: None,
    });
    let _child = arena.insert(InsertSpec {
        widget: Box::new(LoggingWidget {
            bounds: rect(0.0, 0.0, 100.0, 100.0),
            name: "child",
            log: log.clone(),
        }),
        parent: Some(root),
    });

    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::new(50.0, 50.0), 0);
    r.finger_up(1, Vec2::new(50.0, 50.0), 50);
    let n = r.flush(&mut arena);

    assert_eq!(n, 1);
    assert_eq!(log.borrow().clone(), vec!["child"]);
}

// ---- multi-touch ----

#[test]
fn two_fingers_tap_independently() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_down(2, Vec2::new(50.0, 0.0), 10);
    r.finger_up(1, Vec2::ZERO, 100);
    r.finger_up(2, Vec2::new(50.0, 0.0), 110);

    let taps = r
        .pending()
        .filter(|(g, _)| matches!(g, Gesture::Tap { .. }))
        .count();
    assert_eq!(taps, 2);
}

#[test]
fn finger_cancel_drops_state() {
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_motion(1, Vec2::new(20.0, 0.0), 50);
    r.finger_cancel(1);
    assert_eq!(r.finger_count(), 0);
    // up after cancel is a no-op.
    r.finger_up(1, Vec2::new(20.0, 0.0), 100);
    let taps = r
        .pending()
        .filter(|(g, _)| matches!(g, Gesture::Tap { .. }))
        .count();
    assert_eq!(taps, 0);
}

// ---- API smoke ----

#[test]
fn unknown_finger_motion_is_noop() {
    let mut r = GestureRecognizer::new();
    r.finger_motion(99, Vec2::new(10.0, 0.0), 5);
    assert_eq!(r.pending().count(), 0);
    assert_eq!(r.finger_count(), 0);
}

#[test]
fn flush_with_no_hits_returns_zero_and_drains_queue() {
    let mut arena = WidgetArena::new();
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::ZERO, 0);
    r.finger_up(1, Vec2::ZERO, 50);
    assert_eq!(r.pending().count(), 1);

    let n = r.flush(&mut arena);
    assert_eq!(n, 0);
    assert_eq!(r.pending().count(), 0, "queue drained even on miss");
}

#[test]
fn flush_dispatches_into_widget() {
    let (mut arena, _id) = arena_one(rect(0.0, 0.0, 100.0, 100.0), true);
    let mut r = GestureRecognizer::new();
    r.finger_down(1, Vec2::new(50.0, 50.0), 0);
    r.finger_up(1, Vec2::new(50.0, 50.0), 50);
    let n = r.flush(&mut arena);
    assert_eq!(n, 1);
}

// ---- LayoutRect::contains smoke ----

#[test]
fn layout_rect_contains_half_open() {
    let r = rect(10.0, 20.0, 30.0, 40.0);
    assert!(r.contains(Vec2::new(10.0, 20.0))); // top-left included
    assert!(r.contains(Vec2::new(39.9, 59.9))); // inside
    assert!(!r.contains(Vec2::new(40.0, 30.0))); // right edge excluded
    assert!(!r.contains(Vec2::new(20.0, 60.0))); // bottom edge excluded
    assert!(!r.contains(Vec2::new(0.0, 0.0))); // outside
}
