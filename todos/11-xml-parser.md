# 11 — `XmlParser` (roxmltree)

**Status:** ✅ `safi_ui::parse::{XmlParser, ParseError}` with `parse_path` + `parse_str`. `id`/`key` lifted out of `props` (matches `safi-ui-macros::codegen`). Comments stripped, pure-whitespace text skipped, mixed content rejected with line/column. Non-UTF8 paths and malformed XML produce actionable `ParseError`s. Stderr warnings at >512 KB and >20 nesting levels. 11 unit tests + 3 integration tests (incl. 53-node fixture round-trip and a 5 ms p99 perf gate — median observed ~50µs on host). `vnode!` and `XmlParser::parse_str` produce identical `VNode`s for equivalent source. `cargo fmt` / `cargo clippy --workspace --all-targets -- -D warnings` / full `cargo test -p safi-ui` clean.

**Phase:** 2 — Layout + Parse
**PRD refs:** §6.5, §12

## Goal

Parse `.xml` files into `VNode` trees with helpful error reporting.

## Deliverables

- `safi-ui::parse::{XmlParser, ParseError}`
- Inputs: `parse_path(&Path)` and `parse_str(&str, source_name)`
- UTF-8 enforcement; non-UTF8 is a `ParseError`
- `ParseError` carries `source_name`, `line`, `column`, message
- Preserves the `id` and `key` props as first-class fields on `VNode`
- Comments stripped silently
- Performance warning when a single file exceeds 512 KB or nesting passes 20 levels

## Dependencies

- `03`

## Acceptance

- Cold parse < 5ms for a 50-node screen on Pixel 8 (PRD §17.1)
- Snapshot tests for valid + malformed XML
- Error messages include file name + line number
