//! `PropUtils` — typed prop parsing (PRD §6.14, §6.12).
//!
//! Every prop value in XML is a `String`. `PropUtils` turns those
//! strings into typed defaults the renderer can consume without ever
//! exposing components to `Option<None>` for required props (the
//! "typed-defaulted" model from PRD §6.14).
//!
//! The functions live as **methods on `Props`** via an extension trait
//! (`PropsExt`) — that's how the PRD's pseudo-code reads
//! (`props.get_str("label", "Button")`), and the trait keeps the call
//! sites short.
//!
//! ## Binding resolution
//!
//! `{{key}}` syntax in prop values resolves against any source
//! implementing [`BindingSource`] — typically [`StateStore`] in the
//! runtime, a [`HashMap<&str, &str>`] in tests. Missing keys resolve to
//! **empty string** (PRD §6.12) — never an error, never a panic.
//!
//! Composite bindings like `"Hello {{name}}!"` register a subscription
//! on *every* key referenced; [`resolve_composite_with_keys`] returns
//! the resolved string alongside the set of keys it touched so
//! `DirtyTracker` can subscribe the calling widget (PRD §6.12).
//!
//! [`StateStore`]: crate::state_store::StateStore

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use crate::vnode::Props;

/// Trait abstracting whatever holds key→value state — `StateStore` in
/// the runtime, a `HashMap<&str, &str>` (or `HashMap<String, String>`)
/// in tests. Returning `Option<String>` keeps the trait `Send + Sync`
/// without forcing implementors to expose internal borrows.
pub trait BindingSource {
    /// Look up `key`. Return `None` when missing — the resolution
    /// helpers convert that to an empty string per PRD §6.12.
    fn get_binding(&self, key: &str) -> Option<String>;
}

impl<S: BuildHasher> BindingSource for HashMap<String, String, S> {
    fn get_binding(&self, key: &str) -> Option<String> {
        self.get(key).cloned()
    }
}

impl<S: BuildHasher> BindingSource for HashMap<&str, &str, S> {
    fn get_binding(&self, key: &str) -> Option<String> {
        self.get(key).map(|v| (*v).to_string())
    }
}

/// RGBA color in 0..=255 channels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Dimension parsed from a prop string. `Dp(200.0)`, `Percent(50.0)`,
/// or `Auto` (PRD §12.2).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum Dimension {
    Dp(f32),
    Percent(f32),
    #[default]
    Auto,
}

/// Extension trait that hangs typed parsers off [`Props`]. Pulled into
/// scope wherever components parse props.
pub trait PropsExt {
    fn get_str(&self, key: &str, default: &str) -> String;
    fn parse_f32(&self, key: &str, default: f32) -> f32;
    fn parse_bool(&self, key: &str, default: bool) -> bool;
    fn parse_color(&self, key: &str, default: Color) -> Color;
    fn parse_dim(&self, key: &str, default: Dimension) -> Dimension;
    fn resolve_binding<S: BindingSource>(&self, key: &str, store: &S) -> String;
}

impl PropsExt for Props {
    fn get_str(&self, key: &str, default: &str) -> String {
        self.get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    fn parse_f32(&self, key: &str, default: f32) -> f32 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn parse_bool(&self, key: &str, default: bool) -> bool {
        self.get(key).map_or(default, |v| {
            matches!(v.as_str(), "true" | "TRUE" | "True" | "1")
        })
    }

    fn parse_color(&self, key: &str, default: Color) -> Color {
        self.get(key)
            .and_then(|v| parse_color_str(v))
            .unwrap_or(default)
    }

    fn parse_dim(&self, key: &str, default: Dimension) -> Dimension {
        self.get(key)
            .and_then(|v| parse_dim_str(v))
            .unwrap_or(default)
    }

    fn resolve_binding<S: BindingSource>(&self, key: &str, store: &S) -> String {
        let Some(raw) = self.get(key) else {
            return String::new();
        };
        resolve_composite(raw, store)
    }
}

/// Resolve every `{{key}}` reference in `template` against `store`.
/// Missing keys collapse to empty string (PRD §6.12). Returns the
/// fully-resolved string.
pub fn resolve_composite<S: BindingSource>(template: &str, store: &S) -> String {
    resolve_composite_with_keys(template, store).0
}

/// Same as [`resolve_composite`] but also returns the set of keys the
/// template referenced — `DirtyTracker` registers a subscription on
/// each so any of them changing invalidates the calling widget
/// (PRD §6.12).
pub fn resolve_composite_with_keys<S: BindingSource>(
    template: &str,
    store: &S,
) -> (String, HashSet<String>) {
    let mut out = String::with_capacity(template.len());
    let mut keys = HashSet::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        rest = &rest[start + 2..];
        let Some(end) = rest.find("}}") else {
            // Unterminated `{{` — fall through and emit the rest
            // verbatim. Matches the lenient policy (no panics on
            // prop content, ever).
            out.push_str("{{");
            out.push_str(rest);
            return (out, keys);
        };
        let key = rest[..end].trim();
        let resolved = store.get_binding(key).unwrap_or_default();
        out.push_str(&resolved);
        keys.insert(key.to_string());
        rest = &rest[end + 2..];
    }
    out.push_str(rest);
    (out, keys)
}

/// Public alias for callers that want to parse colors outside
/// [`PropsExt`] (renderer-side debug overlays, tests).
pub fn parse_color_str(raw: &str) -> Option<Color> {
    let trimmed = raw.trim();
    if let Some(named) = parse_named_color(trimmed) {
        return Some(named);
    }
    if let Some(rest) = trimmed.strip_prefix('#') {
        return parse_hex_color(rest);
    }
    if let Some(inner) = trimmed
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_rgba_func(inner);
    }
    if let Some(inner) = trimmed
        .strip_prefix("rgb(")
        .and_then(|s| s.strip_suffix(')'))
    {
        return parse_rgb_func(inner);
    }
    None
}

/// Public alias for [`PropsExt::parse_dim`].
pub fn parse_dim_str(raw: &str) -> Option<Dimension> {
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("auto") {
        return Some(Dimension::Auto);
    }
    if let Some(percent) = trimmed.strip_suffix('%') {
        return percent.trim().parse::<f32>().ok().map(Dimension::Percent);
    }
    // Allow trailing "dp" for clarity; "200" and "200dp" both parse.
    let numeric = trimmed.strip_suffix("dp").unwrap_or(trimmed);
    numeric.trim().parse::<f32>().ok().map(Dimension::Dp)
}

fn parse_named_color(raw: &str) -> Option<Color> {
    match raw {
        "transparent" | "TRANSPARENT" => Some(Color::TRANSPARENT),
        "white" | "WHITE" => Some(Color::WHITE),
        "black" | "BLACK" => Some(Color::BLACK),
        "red" => Some(Color::rgb(255, 0, 0)),
        "green" => Some(Color::rgb(0, 128, 0)),
        "blue" => Some(Color::rgb(0, 0, 255)),
        "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
        "yellow" => Some(Color::rgb(255, 255, 0)),
        "orange" => Some(Color::rgb(255, 165, 0)),
        "purple" => Some(Color::rgb(128, 0, 128)),
        _ => None,
    }
}

fn parse_hex_color(raw: &str) -> Option<Color> {
    let raw = raw.trim();
    match raw.len() {
        6 => {
            let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
            Some(Color::rgb(r, g, b))
        }
        8 => {
            let a = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let r = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let g = u8::from_str_radix(&raw[4..6], 16).ok()?;
            let b = u8::from_str_radix(&raw[6..8], 16).ok()?;
            Some(Color::rgba(r, g, b, a))
        }
        3 => {
            // `#RGB` shorthand → each nibble doubled.
            let r = hex_nibble(raw.as_bytes()[0])?;
            let g = hex_nibble(raw.as_bytes()[1])?;
            let b = hex_nibble(raw.as_bytes()[2])?;
            Some(Color::rgb(r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_rgb_func(inner: &str) -> Option<Color> {
    let mut parts = inner.split(',').map(str::trim);
    let r: u8 = parts.next()?.parse().ok()?;
    let g: u8 = parts.next()?.parse().ok()?;
    let b: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Color::rgb(r, g, b))
}

fn parse_rgba_func(inner: &str) -> Option<Color> {
    let mut parts = inner.split(',').map(str::trim);
    let r: u8 = parts.next()?.parse().ok()?;
    let g: u8 = parts.next()?.parse().ok()?;
    let b: u8 = parts.next()?.parse().ok()?;
    let a_raw: f32 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    // PRD: rgba(...) uses 0.0..=1.0 alpha (CSS convention). The clamp
    // ensures the cast is in-range; clippy's cast lints fire on every
    // `as u8` of a float, but here the value is 0..=255 by construction.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let a = (a_raw.clamp(0.0, 1.0) * 255.0).round() as u8;
    Some(Color::rgba(r, g, b, a))
}
