//! Proc-macros for `safi-ui`.
//!
//! Hosts the `vnode!` macro (PRD §6.15) — a declarative XML-shaped DSL that
//! builds `safi_ui::vnode::VNode` trees in Rust. Re-exported from `safi-ui`.

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod codegen;
mod parse;

#[proc_macro]
pub fn vnode(input: TokenStream) -> TokenStream {
    let element = parse_macro_input!(input as parse::Element);
    let _ = element.tag_span;
    codegen::emit(&element).into()
}
