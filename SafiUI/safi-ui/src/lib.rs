//! # Safi-UI
//!
//! Declarative XML-driven mobile UI framework in pure Rust.
//!
//! See [`PRD.md`](https://github.com/AbdulKaderSafi/safi-ui/blob/main/PRD.md)
//! for the full specification.

pub mod arena;
pub mod clip;
pub mod commands;
pub mod component;
pub mod context;
pub mod dirty;
pub mod edge_insets;
pub mod focus;
pub mod gestures;
pub mod gpu;
pub mod vnode;

pub use safi_ui_macros::vnode;
