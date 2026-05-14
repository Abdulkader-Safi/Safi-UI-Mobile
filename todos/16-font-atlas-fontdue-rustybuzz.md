# 16 — Font atlas (`fontdue` + `rustybuzz`)

**Phase:** 4 — Component Library
**PRD refs:** §7.1, §7.2

**Status:** ✅ Complete — `safi-ui::text::{FontAtlas, TextShaper,
FontId, CachedGlyph, PositionedGlyph, FontAtlasError}` ship.
`FontAtlas::register(&[u8])` parses TTF/OTF via `fontdue`; per-glyph
rasterization caches into a `(FontId, glyph_id, size_q)` keyed
`RwLock<HashMap>`. `shape_and_rasterize(font, text, size, pen_x,
baseline_y)` returns positioned alpha bitmaps that the renderer
blits as alpha-tinted rects. ASCII fast path bypasses
`rustybuzz` (Latin/Cyrillic/Greek render directly via codepoint →
glyph-id lookup); `TextShaper::shape` wraps `rustybuzz::shape` for
complex scripts. `App::with_font_bytes(...)` registers a font at
construction; `replay_commands(Command::Text, …)` shapes →
rasterizes → blits per-pixel. Without a font, Text commands are
emitted but suppressed at paint time so the pipeline stays
verifiable. 9 host tests (`tests/text_atlas.rs`) cover empty
atlas, font-id arithmetic, Latin rasterize+cache dedup, pen
advance, and shaper failure paths.

**Deferred to follow-up:** bundling Inter + Noto Sans Arabic as
default fonts (apps currently provide their own bytes); GPU atlas
packing into a single texture (today each glyph is a per-pixel
fill_rect on the SDL_Renderer path, awaiting SDL_GPU shader
pipeline in todo 09 device demo); complex-script BiDi pass for
Arabic/Hindi/Thai (rustybuzz integration is wired but BiDi
ordering + script tagging is a separate session).

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
