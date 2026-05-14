//! `PropUtils` host tests (todo 13, PRD §6.14, §6.12).
//!
//! Coverage targets:
//! - 100% of `parse_color` syntactic variants (#RGB, #RRGGBB, #AARRGGBB,
//!   rgba(...), rgb(...), named, "transparent", malformed)
//! - 100% of `parse_dim` variants (Dp, Percent, Auto, dp suffix,
//!   malformed)
//! - Missing-key bindings → empty string (PRD §6.12)
//! - Composite-binding subscription set covers every key referenced

use std::collections::HashMap;

use safi_ui::props::{
    parse_color_str, parse_dim_str, resolve_composite, resolve_composite_with_keys, Color,
    Dimension, PropsExt,
};

fn props(pairs: &[(&str, &str)]) -> safi_ui::vnode::Props {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

// ----- get_str / parse_f32 / parse_bool ---------------------------------

#[test]
fn get_str_returns_default_when_missing() {
    let p = props(&[]);
    assert_eq!(p.get_str("label", "Button"), "Button");
}

#[test]
fn get_str_returns_value_when_present() {
    let p = props(&[("label", "Submit")]);
    assert_eq!(p.get_str("label", "Button"), "Submit");
}

#[test]
fn parse_f32_default_on_missing_and_bad() {
    let p = props(&[("good", "12.5"), ("bad", "abc")]);
    assert!((p.parse_f32("good", 0.0) - 12.5).abs() < 1e-6);
    assert!((p.parse_f32("bad", 99.0) - 99.0).abs() < 1e-6);
    assert!((p.parse_f32("missing", 7.0) - 7.0).abs() < 1e-6);
}

#[test]
fn parse_bool_accepts_canonical_truthy() {
    let p = props(&[
        ("a", "true"),
        ("b", "TRUE"),
        ("c", "1"),
        ("d", "false"),
        ("e", "no"),
    ]);
    assert!(p.parse_bool("a", false));
    assert!(p.parse_bool("b", false));
    assert!(p.parse_bool("c", false));
    assert!(!p.parse_bool("d", true));
    assert!(!p.parse_bool("e", true));
    assert!(p.parse_bool("missing", true));
}

// ----- parse_color ------------------------------------------------------

#[test]
fn parse_color_rrggbb() {
    assert_eq!(parse_color_str("#0f0f1a"), Some(Color::rgb(15, 15, 26)));
    assert_eq!(parse_color_str("#FFFFFF"), Some(Color::WHITE));
    assert_eq!(parse_color_str("#000000"), Some(Color::BLACK));
}

#[test]
fn parse_color_aarrggbb() {
    // AARRGGBB — alpha first
    assert_eq!(
        parse_color_str("#80ff0000"),
        Some(Color::rgba(255, 0, 0, 0x80))
    );
}

#[test]
fn parse_color_rgb_shorthand() {
    // #RGB → each nibble × 17
    assert_eq!(parse_color_str("#f00"), Some(Color::rgb(255, 0, 0)));
    assert_eq!(parse_color_str("#abc"), Some(Color::rgb(170, 187, 204)));
}

#[test]
fn parse_color_rgba_func() {
    assert_eq!(
        parse_color_str("rgba(255, 100, 0, 0.5)"),
        Some(Color::rgba(255, 100, 0, 128))
    );
    assert_eq!(
        parse_color_str("rgba(0,0,0,1.0)"),
        Some(Color::rgba(0, 0, 0, 255))
    );
}

#[test]
fn parse_color_rgb_func() {
    assert_eq!(
        parse_color_str("rgb(255, 200, 100)"),
        Some(Color::rgb(255, 200, 100))
    );
}

#[test]
fn parse_color_named() {
    assert_eq!(parse_color_str("transparent"), Some(Color::TRANSPARENT));
    assert_eq!(parse_color_str("white"), Some(Color::WHITE));
    assert_eq!(parse_color_str("black"), Some(Color::BLACK));
    assert_eq!(parse_color_str("red"), Some(Color::rgb(255, 0, 0)));
    assert_eq!(parse_color_str("gray"), Some(Color::rgb(128, 128, 128)));
    assert_eq!(parse_color_str("grey"), Some(Color::rgb(128, 128, 128)));
}

#[test]
fn parse_color_handles_whitespace() {
    assert_eq!(parse_color_str("  #f00  "), Some(Color::rgb(255, 0, 0)));
    assert_eq!(
        parse_color_str(" rgba( 1, 2, 3, 0.5 ) "),
        Some(Color::rgba(1, 2, 3, 128))
    );
}

#[test]
fn parse_color_rejects_malformed() {
    assert_eq!(parse_color_str(""), None);
    assert_eq!(parse_color_str("#zzzzzz"), None);
    assert_eq!(parse_color_str("#12345"), None); // 5-digit hex is invalid
    assert_eq!(parse_color_str("rgba(1, 2, 3)"), None); // missing alpha
    assert_eq!(parse_color_str("rgba(1, 2, 3, 0.5, 0)"), None); // 5 parts
    assert_eq!(parse_color_str("rgb(256, 0, 0)"), None); // 256 > u8 max
    assert_eq!(parse_color_str("not-a-color"), None);
}

#[test]
fn parse_color_via_props_uses_default_on_miss() {
    let p = props(&[("good", "#ff0000")]);
    assert_eq!(p.parse_color("good", Color::BLACK), Color::rgb(255, 0, 0));
    assert_eq!(p.parse_color("missing", Color::WHITE), Color::WHITE);
    let bad = props(&[("c", "garbage")]);
    assert_eq!(bad.parse_color("c", Color::BLACK), Color::BLACK);
}

// ----- parse_dim --------------------------------------------------------

#[test]
fn parse_dim_dp() {
    assert_eq!(parse_dim_str("200"), Some(Dimension::Dp(200.0)));
    assert_eq!(parse_dim_str("200dp"), Some(Dimension::Dp(200.0)));
    assert_eq!(parse_dim_str(" 12.5dp "), Some(Dimension::Dp(12.5)));
}

#[test]
fn parse_dim_percent() {
    assert_eq!(parse_dim_str("50%"), Some(Dimension::Percent(50.0)));
    assert_eq!(parse_dim_str(" 100% "), Some(Dimension::Percent(100.0)));
    assert_eq!(parse_dim_str("33.33%"), Some(Dimension::Percent(33.33)));
}

#[test]
fn parse_dim_auto() {
    assert_eq!(parse_dim_str("auto"), Some(Dimension::Auto));
    assert_eq!(parse_dim_str("AUTO"), Some(Dimension::Auto));
}

#[test]
fn parse_dim_rejects_malformed() {
    assert_eq!(parse_dim_str(""), None);
    assert_eq!(parse_dim_str("abc"), None);
    assert_eq!(parse_dim_str("12pixels"), None);
    assert_eq!(parse_dim_str("%50"), None);
}

#[test]
fn parse_dim_via_props_uses_default_on_miss() {
    let p = props(&[("w", "50%")]);
    assert_eq!(p.parse_dim("w", Dimension::Auto), Dimension::Percent(50.0));
    assert_eq!(
        p.parse_dim("missing", Dimension::Dp(7.0)),
        Dimension::Dp(7.0)
    );
}

// ----- Bindings ---------------------------------------------------------

#[test]
fn missing_binding_resolves_to_empty_string() {
    let store: HashMap<String, String> = HashMap::new();
    let p = props(&[("label", "{{user.name}}")]);
    assert_eq!(p.resolve_binding("label", &store), "");
}

#[test]
fn present_binding_substitutes_value() {
    let mut store: HashMap<String, String> = HashMap::new();
    store.insert("user.name".to_string(), "Safi".to_string());
    let p = props(&[("label", "{{user.name}}")]);
    assert_eq!(p.resolve_binding("label", &store), "Safi");
}

#[test]
fn composite_binding_concatenates() {
    let mut store: HashMap<String, String> = HashMap::new();
    store.insert("first".to_string(), "Abdul".to_string());
    store.insert("last".to_string(), "Safi".to_string());
    let p = props(&[("greeting", "Hello {{first}} {{last}}!")]);
    assert_eq!(p.resolve_binding("greeting", &store), "Hello Abdul Safi!");
}

#[test]
fn composite_binding_with_keys_returns_all_referenced_keys() {
    let mut store: HashMap<String, String> = HashMap::new();
    store.insert("first".to_string(), "A".to_string());
    let (resolved, keys) = resolve_composite_with_keys("{{first}} {{last}}", &store);
    assert_eq!(resolved, "A ");
    assert!(keys.contains("first"));
    assert!(keys.contains("last"));
    assert_eq!(keys.len(), 2);
}

#[test]
fn resolve_composite_with_no_bindings_passes_through() {
    let store: HashMap<String, String> = HashMap::new();
    assert_eq!(resolve_composite("plain text", &store), "plain text");
}

#[test]
fn resolve_composite_handles_unterminated_brace() {
    let store: HashMap<String, String> = HashMap::new();
    // Unterminated `{{` is not an error — we emit the rest verbatim.
    assert_eq!(resolve_composite("Hello {{name", &store), "Hello {{name");
}

#[test]
fn resolve_composite_trims_key_whitespace() {
    let mut store: HashMap<String, String> = HashMap::new();
    store.insert("k".to_string(), "v".to_string());
    assert_eq!(resolve_composite("{{ k }}", &store), "v");
}

#[test]
fn resolve_binding_via_missing_prop_returns_empty() {
    let store: HashMap<String, String> = HashMap::new();
    let p = props(&[]);
    assert_eq!(p.resolve_binding("missing", &store), "");
}
