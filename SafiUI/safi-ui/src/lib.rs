//! # Safi-UI
//!
//! Declarative XML-driven mobile UI framework in pure Rust.
//!
//! See [`PRD.md`](https://github.com/AbdulKaderSafi/safi-ui/blob/main/PRD.md)
//! for the full specification.

#[cfg(feature = "runtime")]
pub mod app;
pub mod arena;
pub mod assets;
pub mod clip;
pub mod commands;
pub mod component;
pub mod context;
pub mod debug_box;
pub mod dirty;
pub mod edge_insets;
pub mod focus;
pub mod gestures;
pub mod gpu;
pub mod layout;
pub mod parse;
pub mod props;
pub mod registry;
pub mod vnode;
pub mod widgets;

#[cfg(feature = "runtime")]
pub use app::App;
pub use safi_ui_macros::vnode;

/// Emit the `SDL_main` C entry point for a Safi-UI app.
///
/// Expands to a `#[unsafe(no_mangle)] extern "C" fn SDL_main` that
/// constructs an [`App`] over the given root-tree factory and runs it.
/// Place this once at the top of your mobile cdylib's source — the entire
/// app then collapses to that call plus a `Fn() -> VNode` builder.
///
/// Requires the `runtime` feature.
///
/// # Example
///
/// ```ignore
/// safi_ui::app_main!(build_ui);
///
/// fn build_ui() -> safi_ui::vnode::VNode {
///     safi_ui::vnode! { <Screen bg="#0f0f1a" width="100%" height="100%" /> }
/// }
/// ```
#[cfg(feature = "runtime")]
#[macro_export]
macro_rules! app_main {
    ($build_root:expr) => {
        #[unsafe(no_mangle)]
        pub(crate) unsafe extern "C" fn SDL_main(
            _argc: ::std::ffi::c_int,
            _argv: *mut *mut ::std::ffi::c_char,
        ) -> ::std::ffi::c_int {
            match $crate::App::new($build_root).run() {
                ::std::result::Result::Ok(()) => 0,
                ::std::result::Result::Err(e) => {
                    $crate::app::log_fatal(&::std::format!("{e}"));
                    1
                }
            }
        }
    };
}
