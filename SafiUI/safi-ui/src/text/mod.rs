//! Text shaping + rasterization (PRD §7.1, §7.2; todo 16).
//!
//! Two responsibilities:
//!
//! - [`FontAtlas`] — owns one or more `fontdue::Font` instances and
//!   caches rasterized glyphs by `(font_id, glyph_id, size_px)`.
//!   Pure CPU side; the GPU texture upload + atlas-packing lands
//!   alongside todo 17's image pipeline (the typed handle is already
//!   threaded through `Command::Text`).
//! - [`TextShaper`] — wraps `rustybuzz` for complex-script + RTL
//!   shaping. ASCII goes through a fast direct-codepoint path that
//!   skips the shaper entirely.
//!
//! ## What the renderer does each frame
//!
//! 1. For each `Command::Text`, call [`FontAtlas::shape_and_rasterize`]
//!    with the text + size + font handle.
//! 2. Receive a list of [`PositionedGlyph`]s — each entry carries an
//!    absolute (x, y) and a reference to the cached pixel data.
//! 3. Blit each glyph's grayscale alpha onto the canvas, tinted by
//!    `Command::Text::color`.
//!
//! The atlas itself is build- and run-portable: nothing here touches
//! SDL3 / Metal / Vulkan. The `SdlGpuRenderer` (todo 09 device demo)
//! and todo 17's texture cache subscribe to atlas-evicted events to
//! drop GPU pages. Until those land, the host renderer (`replay_commands`
//! in `app.rs`) can either skip Text or blit on the CPU.

pub mod atlas;
pub mod shaper;

pub use atlas::{CachedGlyph, FontAtlas, FontAtlasError, FontId, PositionedGlyph};
pub use shaper::TextShaper;
