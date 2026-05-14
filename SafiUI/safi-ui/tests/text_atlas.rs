//! `FontAtlas` + `TextShaper` host tests (todo 16).
//!
//! Several tests require a real TTF; we look for one in this order:
//!
//! 1. `SAFI_UI_TEST_FONT` env var pointing to a `.ttf`/`.otf` file
//! 2. `/System/Library/Fonts/Geneva.ttf` on macOS
//! 3. Skip the test (return early) on every other platform
//!
//! This keeps host CI green on Linux/Windows where no font ships
//! with safi-ui yet (the Inter bundle is a follow-up). The pure-API
//! tests (empty atlas, font-id arithmetic) run everywhere.

use safi_ui::text::{FontAtlas, FontId, TextShaper};

fn test_font_bytes() -> Option<Vec<u8>> {
    if let Ok(p) = std::env::var("SAFI_UI_TEST_FONT") {
        if let Ok(b) = std::fs::read(&p) {
            return Some(b);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(b) = std::fs::read("/System/Library/Fonts/Geneva.ttf") {
            return Some(b);
        }
    }
    None
}

#[test]
fn empty_atlas_has_zero_fonts() {
    let atlas = FontAtlas::new();
    assert_eq!(atlas.font_count(), 0);
    assert_eq!(atlas.cache_len(), 0);
}

#[test]
fn glyph_lookup_returns_none_when_font_id_out_of_range() {
    let atlas = FontAtlas::new();
    assert!(atlas.glyph(FontId::DEFAULT, 'A', 16.0).is_none());
}

#[test]
fn font_id_round_trips_through_font_handle() {
    use safi_ui::commands::FontHandle;
    let id = FontId(7);
    let h: FontHandle = id.into();
    let back: FontId = h.into();
    assert_eq!(back, id);
}

#[test]
fn shape_and_rasterize_returns_empty_for_unregistered_font() {
    let atlas = FontAtlas::new();
    let glyphs = atlas.shape_and_rasterize(FontId::DEFAULT, "hello", 16.0, 0.0, 16.0);
    assert!(glyphs.is_empty());
}

#[test]
fn register_real_font_and_rasterize_latin() {
    let Some(bytes) = test_font_bytes() else {
        eprintln!("text_atlas: no system font available; skipping");
        return;
    };
    let mut atlas = FontAtlas::new();
    let id = atlas.register(&bytes).expect("register font");
    assert_eq!(id, FontId::DEFAULT);

    let g = atlas.glyph(id, 'A', 32.0).expect("rasterize 'A' at 32px");
    assert!(g.width > 0, "'A' should have non-zero width");
    assert!(g.height > 0, "'A' should have non-zero height");
    assert!(!g.pixels.is_empty());
    assert_eq!(g.pixels.len(), g.width * g.height);
}

#[test]
fn cache_dedups_repeated_glyph_requests() {
    let Some(bytes) = test_font_bytes() else {
        return;
    };
    let mut atlas = FontAtlas::new();
    let id = atlas.register(&bytes).unwrap();
    for _ in 0..10 {
        let _ = atlas.glyph(id, 'A', 16.0);
    }
    // After 10 lookups of the same (font, glyph, size), cache holds
    // exactly one entry.
    assert_eq!(atlas.cache_len(), 1);
}

#[test]
fn shape_and_rasterize_advances_pen_across_string() {
    let Some(bytes) = test_font_bytes() else {
        return;
    };
    let mut atlas = FontAtlas::new();
    let id = atlas.register(&bytes).unwrap();
    let glyphs = atlas.shape_and_rasterize(id, "Hello", 24.0, 100.0, 200.0);
    assert_eq!(glyphs.len(), 5);
    // Pen monotonically advances right (Latin LTR) — verify x values
    // are non-decreasing.
    let mut prev = f32::NEG_INFINITY;
    for g in &glyphs {
        assert!(
            g.x >= prev,
            "glyph x={} should be >= prev x={} (LTR pen advance)",
            g.x,
            prev
        );
        prev = g.x;
    }
}

#[test]
fn text_shaper_constructs_from_bytes() {
    let Some(bytes) = test_font_bytes() else {
        return;
    };
    let shaper = TextShaper::from_bytes(bytes);
    let glyphs = shaper.shape("Hello");
    assert_eq!(glyphs.len(), 5, "expected 5 shaped glyphs for 'Hello'");
}

#[test]
fn text_shaper_returns_empty_on_bad_bytes() {
    let shaper = TextShaper::from_bytes(vec![0u8; 16]);
    let glyphs = shaper.shape("Hello");
    assert!(glyphs.is_empty());
}
