# 16 — Font atlas (`fontdue` + `rustybuzz`)

**Phase:** 4 — Component Library
**PRD refs:** §7.1, §7.2

## Goal

Rasterise glyphs into a GPU texture atlas with shaping for complex scripts and RTL.

## Deliverables

- `safi-ui::text::FontAtlas` — `fontdue` rasterisation into a single GPU atlas texture
- `safi-ui::text::TextShaper` — `rustybuzz` shaping pipeline before rasterisation
- Bundled defaults: **Inter** (Latin) + **Noto Sans Arabic** (RTL)
- Atlas rebuild on DPI scale change (e.g. external display attached, font-size jumps)
- Glyph LRU eviction when atlas approaches capacity
- `Command::Text` rendering path samples the atlas with subpixel positioning

## Dependencies

- `09`, `12`, `13`

## Acceptance

- Latin, Arabic, Hindi, and Thai sample strings render correctly
- Atlas rebuild on DPI change does not crash; glyph cache repopulates lazily
- Sub-1ms shaping for a 200-character paragraph
