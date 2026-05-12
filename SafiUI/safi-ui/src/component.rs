//! Minimal `Component` trait stub.
//!
//! Just enough surface for `WidgetArena` (todo `04`) to store `Box<dyn Component>`.
//! Todo `13` expands this to the full PRD §6.8 trait (`build`, `on_gesture`,
//! lifecycle hooks, `hit_test`).

use crate::vnode::LayoutRect;

pub trait Component {
    fn bounds(&self) -> LayoutRect;
}
