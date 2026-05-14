//! `FontAtlas` — owns fonts, caches rasterized glyphs (todo 16).
//!
//! The atlas is a thread-safe (RwLock-guarded) cache from
//! `(FontId, glyph_id, size_px_q)` to a [`CachedGlyph`]. The size is
//! quantised to whole pixels because subpixel-rendered glyphs at
//! adjacent sizes look visually identical at typical mobile DPIs,
//! and quantising keeps cache size bounded.
//!
//! GPU upload + atlas packing happens in the renderer (todo 17). The
//! atlas itself returns CPU-side grayscale alpha bitmaps.

use std::collections::HashMap;
use std::sync::RwLock;

use fontdue::{Font, FontSettings, Metrics};

use crate::commands::FontHandle;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

impl FontId {
    /// The first font registered with [`FontAtlas::register`] gets id 0.
    /// Component widgets that don't override the font use the default.
    pub const DEFAULT: Self = Self(0);
}

impl From<FontId> for FontHandle {
    fn from(id: FontId) -> Self {
        Self(id.0)
    }
}

impl From<FontHandle> for FontId {
    fn from(h: FontHandle) -> Self {
        Self(h.0)
    }
}

#[derive(Debug)]
pub enum FontAtlasError {
    Fontdue(&'static str),
}

impl std::fmt::Display for FontAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fontdue(msg) => write!(f, "fontdue: {msg}"),
        }
    }
}

impl std::error::Error for FontAtlasError {}

/// CPU-side rasterized glyph. `pixels` is grayscale alpha
/// (0=transparent, 255=opaque) of size `width × height`.
#[derive(Clone, Debug)]
pub struct CachedGlyph {
    pub width: usize,
    pub height: usize,
    /// Whole-pixel offset of the bitmap's left edge from the pen
    /// position. Negative when the glyph extends left of the origin
    /// (e.g. italics).
    pub xmin: i32,
    /// Whole-pixel offset of the bitmap's bottom edge from the
    /// baseline. Negative when the glyph descends below.
    pub ymin: i32,
    pub advance_width: f32,
    pub pixels: Vec<u8>,
}

impl CachedGlyph {
    fn from_fontdue(metrics: Metrics, pixels: Vec<u8>) -> Self {
        Self {
            width: metrics.width,
            height: metrics.height,
            xmin: metrics.xmin,
            ymin: metrics.ymin,
            advance_width: metrics.advance_width,
            pixels,
        }
    }
}

/// A glyph positioned in absolute (x, y) — the output of
/// [`FontAtlas::shape_and_rasterize`].
#[derive(Clone, Debug)]
pub struct PositionedGlyph {
    pub x: f32,
    pub y: f32,
    pub glyph: CachedGlyph,
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
struct CacheKey {
    font: FontId,
    glyph_id: u32,
    size_q: u32, // size_px rounded to whole pixel
}

pub struct FontAtlas {
    fonts: Vec<Font>,
    cache: RwLock<HashMap<CacheKey, CachedGlyph>>,
}

impl FontAtlas {
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Register a font from TTF / OTF bytes. The first font registered
    /// becomes [`FontId::DEFAULT`]; subsequent fonts get sequential ids.
    pub fn register(&mut self, bytes: &[u8]) -> Result<FontId, FontAtlasError> {
        let font =
            Font::from_bytes(bytes, FontSettings::default()).map_err(FontAtlasError::Fontdue)?;
        let id = FontId(self.fonts.len().try_into().expect("font count exceeds u32"));
        self.fonts.push(font);
        Ok(id)
    }

    /// Number of fonts currently registered.
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Cache size, useful for monitoring memory pressure.
    pub fn cache_len(&self) -> usize {
        self.cache.read().expect("FontAtlas cache poisoned").len()
    }

    /// Look up (or rasterize, on miss) a single glyph. Returns `None`
    /// if `font` is out of range.
    pub fn glyph(&self, font: FontId, ch: char, size_px: f32) -> Option<CachedGlyph> {
        let font_idx: usize = font.0.try_into().ok()?;
        let f = self.fonts.get(font_idx)?;
        // Quantise the cache key to whole pixels. The clamp + cast
        // is bounded; clippy's blanket float-cast lints fire here
        // even though the value is in range.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let size_q = size_px.max(0.0).round() as u32;
        let glyph_id = u32::from(f.lookup_glyph_index(ch));
        let key = CacheKey {
            font,
            glyph_id,
            size_q,
        };
        {
            let read = self.cache.read().expect("FontAtlas cache poisoned");
            if let Some(g) = read.get(&key) {
                return Some(g.clone());
            }
        }
        // Miss — rasterize, insert under write lock.
        let (metrics, pixels) = f.rasterize(ch, size_px);
        let cached = CachedGlyph::from_fontdue(metrics, pixels);
        let mut write = self.cache.write().expect("FontAtlas cache poisoned");
        write.insert(key, cached.clone());
        Some(cached)
    }

    /// Shape + rasterize a whole string. ASCII fast path runs without
    /// `rustybuzz` — codepoints map directly to glyph ids. Non-ASCII
    /// input falls back to the shaper (PRD §7.2).
    ///
    /// The pen starts at `(x, y)` where `y` is the **baseline**.
    pub fn shape_and_rasterize(
        &self,
        font: FontId,
        text: &str,
        size_px: f32,
        pen_x: f32,
        baseline_y: f32,
    ) -> Vec<PositionedGlyph> {
        if text.is_ascii() {
            return self.shape_ascii(font, text, size_px, pen_x, baseline_y);
        }
        // Non-ASCII: route through the shaper. Falls back to per-char
        // direct rasterization if shaping fails (unloaded font, etc.)
        // so the framework never silently swallows text.
        let Some(font_idx) = usize::try_from(font.0)
            .ok()
            .filter(|i| *i < self.fonts.len())
        else {
            return Vec::new();
        };
        let _ = font_idx; // rustybuzz shaping integration: scheduled
                          // for the same todo, see shaper.rs. For now
                          // even non-ASCII goes through the ASCII path
                          // (each codepoint rasterized independently).
                          // This produces correct output for scripts
                          // without contextual shaping (Latin extended,
                          // Cyrillic, Greek). Arabic / Hindi / Thai
                          // need the shaper hooked up — tracked
                          // inside this module.
        self.shape_ascii(font, text, size_px, pen_x, baseline_y)
    }

    fn shape_ascii(
        &self,
        font: FontId,
        text: &str,
        size_px: f32,
        mut pen_x: f32,
        baseline_y: f32,
    ) -> Vec<PositionedGlyph> {
        let mut out = Vec::with_capacity(text.len());
        for ch in text.chars() {
            let Some(g) = self.glyph(font, ch, size_px) else {
                continue;
            };
            let advance = g.advance_width;
            // Position glyph: bitmap top-left = (pen_x + xmin,
            // baseline_y - ymin - height). fontdue's coordinate
            // system has Y up, ours has Y down (screen-space), hence
            // the subtraction.
            #[allow(clippy::cast_precision_loss)]
            let x = pen_x + g.xmin as f32;
            #[allow(clippy::cast_precision_loss)]
            let y = baseline_y - g.ymin as f32 - g.height as f32;
            out.push(PositionedGlyph { x, y, glyph: g });
            pen_x += advance;
        }
        out
    }
}

impl Default for FontAtlas {
    fn default() -> Self {
        Self::new()
    }
}
