//! `Component` trait (PRD §6.8).
//!
//! Every widget — built-in or user-defined — implements `Component`.
//! The trait deliberately carries **no `Send + Sync` bound** because
//! component instances live on the main thread only; cross-thread
//! coordination happens via `EventBus::post_async` (todo 24) and the
//! image-decode channel (todo 17). Without the bound, custom components
//! can hold `Rc<…>` / `RefCell<…>` / SDL handles without ceremony.
//!
//! ## Lifecycle semantics
//!
//! - `build` — emit draw commands. Called once per dirty subtree per
//!   frame (per-subtree dirty tracking, PRD §6.4). Main-thread only.
//! - `on_mount` — fires **every time** the component enters the live
//!   tree: first appearance, post-hot-reload remount of an unmatched
//!   node, or recycle-back-into-window for a virtualised `FlatList` item.
//! - `on_unmount` — fires on every leave, including `FlatList`
//!   recycle-out. `visible="false"` does **not** trigger this — it
//!   skips `build` only and preserves instance state.
//! - `on_layout` — fires only when bounds change (delta vs last laid-out
//!   frame). Quiet on static layouts. Use this to position tooltips or
//!   measure-dependent children without thrashing every frame.

use glam::Vec2;

use crate::context::UIContext;
use crate::gestures::Gesture;
use crate::vnode::LayoutRect;

pub trait Component {
    /// Bounds in layout (dp) space, populated by `LayoutEngine`.
    fn bounds(&self) -> LayoutRect;

    /// Emit draw commands for this widget's current state.
    ///
    /// The default no-op makes the trait easier to adopt incrementally:
    /// a stateful widget that needs lifecycle wiring but not rendering
    /// can omit it. Built-in widgets (`View`, `Text`, `Button`, …) all
    /// override.
    fn build(&self, _ctx: &mut UIContext, _bounds: LayoutRect) {}

    /// Return `true` if `point` is inside this widget's interactive
    /// area. Default: point-in-bounds.
    fn hit_test(&self, point: Vec2) -> bool {
        self.bounds().contains(point)
    }

    /// Handle a recognised gesture. Return `true` to consume it (stops
    /// the reverse-Z dispatch walk in
    /// [`crate::gestures::GestureRecognizer::flush`]).
    fn on_gesture(&mut self, _gesture: Gesture) -> bool {
        false
    }

    /// Fires whenever the component enters the live tree. Use for one-
    /// shot setup tied to visibility — registering event handlers,
    /// kicking off prefetches.
    fn on_mount(&mut self, _ctx: &mut UIContext) {}

    /// Fires whenever the component leaves the live tree (including
    /// `FlatList` recycle-out). Symmetric counterpart to `on_mount`.
    fn on_unmount(&mut self, _ctx: &mut UIContext) {}

    /// Fires **only when bounds change** vs the last laid-out frame.
    /// Does not fire every frame on static layouts. Useful for
    /// positioning tooltips, measuring child intrinsic sizes, etc.
    fn on_layout(&mut self, _bounds: LayoutRect) {}
}
