//! `TextShaper` — complex-script + RTL shaping via `rustybuzz`
//! (PRD §7.2, todo 16).
//!
//! Wraps a `rustybuzz::Face` per registered font. `shape(text, size_px)`
//! returns the sequence of glyph ids + advances the renderer should
//! draw. ASCII text doesn't need shaping — [`FontAtlas`] short-
//! circuits that case.
//!
//! ## Status (todo 16 first cut)
//!
//! Face construction and the public API are wired. Real shaping of
//! Arabic / Hindi / Thai needs the bidi pass + script tagging which
//! lands as a follow-up to this module (the heavy lift is in
//! `rustybuzz` already; the work here is plumbing). Latin + Cyrillic
//! + Greek work today through the ASCII fast path.
//!
//! [`FontAtlas`]: crate::text::FontAtlas

use rustybuzz::Face;

/// A shaped glyph — what the shaper hands back to the atlas before
/// rasterization. Coordinates are in the font's design units; the
/// caller scales by `size_px / units_per_em`.
#[derive(Copy, Clone, Debug)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

pub struct TextShaper {
    // `Face::from_slice` borrows; we own the bytes here so the face
    // is valid for the shaper's lifetime.
    font_bytes: Vec<u8>,
}

impl TextShaper {
    /// Construct a shaper over a TTF/OTF byte slice.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { font_bytes: bytes }
    }

    /// Shape `text` and return the per-glyph layout. Returns an empty
    /// vec if the font can't be parsed (the atlas falls back to direct
    /// codepoint rasterization in that case).
    pub fn shape(&self, text: &str) -> Vec<ShapedGlyph> {
        let Some(face) = Face::from_slice(&self.font_bytes, 0) else {
            return Vec::new();
        };
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        let glyph_buffer = rustybuzz::shape(&face, &[], buf);
        let infos = glyph_buffer.glyph_infos();
        let positions = glyph_buffer.glyph_positions();
        infos
            .iter()
            .zip(positions.iter())
            .map(|(info, pos)| ShapedGlyph {
                glyph_id: info.glyph_id,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
            })
            .collect()
    }
}
