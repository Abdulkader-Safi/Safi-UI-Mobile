//! `Component` trait (PRD §6.8).
//!
//! Currently includes `bounds`, `hit_test`, and `on_gesture` — enough surface
//! for the widget arena (todo `04`) and gesture recognizer (todo `08`). Todo
//! `13` adds `build` and the lifecycle hooks (`on_mount` / `on_unmount` /
//! `on_layout`) to complete §6.8.

use glam::Vec2;

use crate::gestures::Gesture;
use crate::vnode::LayoutRect;

pub trait Component {
    fn bounds(&self) -> LayoutRect;

    /// Return `true` if `point` is inside this widget's interactive area.
    /// Default: point-in-bounds.
    fn hit_test(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }

    /// Handle a recognised gesture. Return `true` to consume it (stops the
    /// reverse-Z dispatch walk in [`crate::gestures::GestureRecognizer::flush`]).
    fn on_gesture(&mut self, _gesture: Gesture) -> bool {
        false
    }
}
