# 03 — `VNode` struct and `vnode!` macro

**Status:** ✅ Completed. `VNode`/`Props`/`LayoutRect` land in `safi-ui/src/vnode.rs`; `vnode!` proc-macro implemented in `safi-ui-macros` (parser + codegen) and re-exported from `safi-ui`. 10 runtime tests + 5 trybuild compile-fail snapshots all pass; `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean.

**Phase:** 0 — Foundations
**PRD refs:** §6.1, §6.15

## Goal

Land the `VNode` data structure and the proc-macro DSL that builds `VNode` trees in Rust. Phase 1 work depends on this because the XML parser doesn't exist yet.

## Deliverables

- `safi-ui::vnode::{VNode, Props, LayoutRect}` types matching §6.1 exactly
- `safi-ui-macros::vnode!` proc-macro:
  - Mirrors XML syntax: `<Tag prop="value">...</Tag>`
  - String literals only as prop values
  - Bare string literal in body → `text_content`
  - Bindings preserved verbatim: `value="{{user.name}}"`
  - Unknown tags compile fine (resolved at runtime)
  - Malformed syntax fails at compile time with helpful errors
- Re-export `vnode!` from `safi-ui` crate root
- Unit tests covering: nested children, props, bindings, text content, edge cases (empty tag, self-closing)

## Dependencies

- `00`

## Acceptance

- `vnode! { <Column><Text>"hi"</Text></Column> }` produces the expected tree
- Compile-error snapshot tests for malformed input
- ≥ 90% line coverage on `safi-ui-macros`
