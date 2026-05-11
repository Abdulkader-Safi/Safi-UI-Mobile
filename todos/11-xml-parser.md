# 11 — `XmlParser` (roxmltree)

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
