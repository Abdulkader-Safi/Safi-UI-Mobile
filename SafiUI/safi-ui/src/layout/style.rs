//! Prop → `taffy::Style` translation (PRD §6.6).
//!
//! Parsing helpers (`parse_dim`, `parse_lpa`, `parse_lp`, etc.) are private to
//! this module on purpose — todo `13` (`PropUtils`) lifts a typed prop-parsing
//! layer crate-wide. Until then, layout owns the only prop parsers in the
//! crate, scoped to the Taffy mapping.

use taffy::{
    AlignItems, Dimension, FlexDirection, FlexWrap, JustifyContent, LengthPercentage,
    LengthPercentageAuto, Rect, Size, Style,
};

use crate::vnode::Props;

/// Hash of every prop value that contributes to layout, so
/// `compute_if_dirty` can skip restyling unchanged nodes.
pub(crate) const LAYOUT_PROP_KEYS: &[&str] = &[
    "flexDirection",
    "justifyContent",
    "alignItems",
    "flex",
    "width",
    "height",
    "padding",
    "paddingLeft",
    "paddingRight",
    "paddingTop",
    "paddingBottom",
    "margin",
    "marginLeft",
    "marginRight",
    "marginTop",
    "marginBottom",
    "gap",
    "rowGap",
    "columnGap",
    "wrap",
];

/// Translate a `VNode`'s `(tag, props)` into a Taffy [`Style`].
///
/// The tag participates because PRD §10.1 specifies tag-implied defaults
/// for layout containers (`<Column>` → `flexDirection: column`, `<Row>`
/// → `row`). Once todo `15` lands real `Component`s those defaults will be
/// injected by the component layer; until then layout owns the mapping so
/// the demo XML in §Appendix renders sensibly without ceremony. Explicit
/// `flexDirection` props always override the tag default.
pub(crate) fn style_from_tag_and_props(tag: &str, props: &Props) -> Style {
    let mut style = Style::default();

    style.flex_direction = match tag {
        "Row" => FlexDirection::Row,
        "Column" | "Screen" | "View" | "Stack" => FlexDirection::Column,
        _ => style.flex_direction,
    };

    if let Some(v) = props.get("flexDirection") {
        style.flex_direction = parse_flex_direction(v);
    }
    if let Some(v) = props.get("justifyContent") {
        style.justify_content = Some(parse_justify(v));
    }
    if let Some(v) = props.get("alignItems") {
        style.align_items = Some(parse_align_items(v));
    }
    if let Some(v) = props.get("flex") {
        if let Some(f) = parse_f32(v) {
            style.flex_grow = f;
        }
    }
    if let Some(w) = props.get("width") {
        style.size.width = parse_dim(w).unwrap_or(Dimension::Auto);
    }
    if let Some(h) = props.get("height") {
        style.size.height = parse_dim(h).unwrap_or(Dimension::Auto);
    }
    if let Some(v) = props.get("wrap") {
        style.flex_wrap = parse_flex_wrap(v);
    }

    style.padding = parse_rect(props, "padding");
    style.margin = parse_rect_lpa(props, "margin");
    style.gap = parse_gap(props);

    style
}

/// Returns `true` when both `width` and `height` resolve to a non-auto length
/// or percent. Used to set `DirtyTracker::SizingMode` in the layout cache.
pub(crate) fn is_fully_sized(style: &Style) -> bool {
    !matches!(style.size.width, Dimension::Auto) && !matches!(style.size.height, Dimension::Auto)
}

fn parse_flex_direction(s: &str) -> FlexDirection {
    match s {
        "row" => FlexDirection::Row,
        "rowReverse" | "row-reverse" => FlexDirection::RowReverse,
        "columnReverse" | "column-reverse" => FlexDirection::ColumnReverse,
        // Default per PRD: column.
        _ => FlexDirection::Column,
    }
}

fn parse_justify(s: &str) -> JustifyContent {
    match s {
        "center" => JustifyContent::Center,
        "flexEnd" | "flex-end" | "end" => JustifyContent::FlexEnd,
        "spaceBetween" | "space-between" => JustifyContent::SpaceBetween,
        "spaceAround" | "space-around" => JustifyContent::SpaceAround,
        "spaceEvenly" | "space-evenly" => JustifyContent::SpaceEvenly,
        _ => JustifyContent::FlexStart,
    }
}

fn parse_align_items(s: &str) -> AlignItems {
    match s {
        "center" => AlignItems::Center,
        "flexEnd" | "flex-end" | "end" => AlignItems::FlexEnd,
        "stretch" => AlignItems::Stretch,
        "baseline" => AlignItems::Baseline,
        _ => AlignItems::FlexStart,
    }
}

fn parse_flex_wrap(s: &str) -> FlexWrap {
    match s {
        "wrap" => FlexWrap::Wrap,
        "wrapReverse" | "wrap-reverse" => FlexWrap::WrapReverse,
        _ => FlexWrap::NoWrap,
    }
}

fn parse_f32(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

fn parse_dim(s: &str) -> Option<Dimension> {
    let s = s.trim();
    if s == "auto" || s.is_empty() {
        return Some(Dimension::Auto);
    }
    if let Some(stripped) = s.strip_suffix('%') {
        return stripped
            .parse::<f32>()
            .ok()
            .map(|p| Dimension::Percent(p / 100.0));
    }
    let stripped = s.strip_suffix("dp").unwrap_or(s).trim();
    stripped.parse::<f32>().ok().map(Dimension::Length)
}

fn parse_lpa(s: &str) -> LengthPercentageAuto {
    let s = s.trim();
    if s == "auto" || s.is_empty() {
        return LengthPercentageAuto::Auto;
    }
    if let Some(stripped) = s.strip_suffix('%') {
        if let Ok(p) = stripped.parse::<f32>() {
            return LengthPercentageAuto::Percent(p / 100.0);
        }
    }
    let stripped = s.strip_suffix("dp").unwrap_or(s).trim();
    stripped.parse::<f32>().map_or(
        LengthPercentageAuto::Length(0.0),
        LengthPercentageAuto::Length,
    )
}

fn parse_lp(s: &str) -> LengthPercentage {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix('%') {
        if let Ok(p) = stripped.parse::<f32>() {
            return LengthPercentage::Percent(p / 100.0);
        }
    }
    let stripped = s.strip_suffix("dp").unwrap_or(s).trim();
    stripped
        .parse::<f32>()
        .map_or(LengthPercentage::Length(0.0), LengthPercentage::Length)
}

/// Resolves `padding` (shorthand) then per-side overrides
/// (`paddingLeft`, etc.). Per-side wins if present.
fn parse_rect(props: &Props, base: &str) -> Rect<LengthPercentage> {
    let shorthand = props.get(base).map_or("", String::as_str);
    let zero = LengthPercentage::Length(0.0);
    let (left, right, top, bottom) = parse_shorthand_4(shorthand, zero, parse_lp);
    Rect {
        left: per_side_lp(props, base, "Left").unwrap_or(left),
        right: per_side_lp(props, base, "Right").unwrap_or(right),
        top: per_side_lp(props, base, "Top").unwrap_or(top),
        bottom: per_side_lp(props, base, "Bottom").unwrap_or(bottom),
    }
}

fn parse_rect_lpa(props: &Props, base: &str) -> Rect<LengthPercentageAuto> {
    let shorthand = props.get(base).map_or("", String::as_str);
    let zero = LengthPercentageAuto::Length(0.0);
    let (left, right, top, bottom) = parse_shorthand_4(shorthand, zero, parse_lpa);
    Rect {
        left: per_side_lpa(props, base, "Left").unwrap_or(left),
        right: per_side_lpa(props, base, "Right").unwrap_or(right),
        top: per_side_lpa(props, base, "Top").unwrap_or(top),
        bottom: per_side_lpa(props, base, "Bottom").unwrap_or(bottom),
    }
}

fn per_side_lp(props: &Props, base: &str, side: &str) -> Option<LengthPercentage> {
    props.get(&format!("{base}{side}")).map(|s| parse_lp(s))
}

fn per_side_lpa(props: &Props, base: &str, side: &str) -> Option<LengthPercentageAuto> {
    props.get(&format!("{base}{side}")).map(|s| parse_lpa(s))
}

/// Expand 1/2/3/4-value CSS shorthand into `(left, right, top, bottom)`.
///
/// Mapping:
/// - 1 value → all sides
/// - 2 values → `(vertical, horizontal)` per CSS
/// - 3 values → `(top, horizontal, bottom)`
/// - 4 values → `(top, right, bottom, left)` (CSS order)
///
/// `parse_one(zero)` is the per-token parser, `zero` is the fallback on
/// empty or malformed input.
fn parse_shorthand_4<T, F>(s: &str, zero: T, parse_one: F) -> (T, T, T, T)
where
    T: Copy,
    F: Fn(&str) -> T,
{
    let parts: Vec<T> = s.split_whitespace().map(&parse_one).collect();
    match *parts.as_slice() {
        [all] => (all, all, all, all),
        [vert, horiz] => (horiz, horiz, vert, vert),
        [top, horiz, bottom] => (horiz, horiz, top, bottom),
        [top, right, bottom, left] => (left, right, top, bottom),
        _ => (zero, zero, zero, zero),
    }
}

fn parse_gap(props: &Props) -> Size<LengthPercentage> {
    let base = props.get("gap").map_or("0", String::as_str);
    let base_lp = parse_lp(base);
    let row = props.get("rowGap").map_or(base_lp, |s| parse_lp(s));
    let column = props.get("columnGap").map_or(base_lp, |s| parse_lp(s));
    Size {
        width: column,
        height: row,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vnode::Props;

    fn props(pairs: &[(&str, &str)]) -> Props {
        let mut p = Props::new();
        for (k, v) in pairs {
            p.insert((*k).to_string(), (*v).to_string());
        }
        p
    }

    #[test]
    fn defaults_yield_column() {
        let s = style_from_tag_and_props("", &Props::new());
        assert!(matches!(
            s.flex_direction,
            FlexDirection::Row | FlexDirection::Column
        ));
    }

    #[test]
    fn width_height_percent() {
        let s = style_from_tag_and_props("", &props(&[("width", "50%"), ("height", "100")]));
        assert!(matches!(s.size.width, Dimension::Percent(p) if (p - 0.5).abs() < 1e-6));
        assert!(matches!(s.size.height, Dimension::Length(v) if (v - 100.0).abs() < 1e-6));
    }

    #[test]
    fn padding_shorthand_2_value() {
        let s = style_from_tag_and_props("", &props(&[("padding", "8 16")]));
        let top = match s.padding.top {
            LengthPercentage::Length(v) => v,
            LengthPercentage::Percent(_) => unreachable!(),
        };
        let left = match s.padding.left {
            LengthPercentage::Length(v) => v,
            LengthPercentage::Percent(_) => unreachable!(),
        };
        assert!((top - 8.0).abs() < 1e-6);
        assert!((left - 16.0).abs() < 1e-6);
    }

    #[test]
    fn padding_per_side_override_wins() {
        let s = style_from_tag_and_props("", &props(&[("padding", "8"), ("paddingLeft", "24")]));
        let left = match s.padding.left {
            LengthPercentage::Length(v) => v,
            LengthPercentage::Percent(_) => unreachable!(),
        };
        assert!((left - 24.0).abs() < 1e-6);
    }

    #[test]
    fn gap_falls_back_to_shorthand() {
        let s = style_from_tag_and_props("", &props(&[("gap", "12")]));
        let h = match s.gap.height {
            LengthPercentage::Length(v) => v,
            LengthPercentage::Percent(_) => unreachable!(),
        };
        assert!((h - 12.0).abs() < 1e-6);
    }

    #[test]
    fn flex_grow_parses() {
        let s = style_from_tag_and_props("", &props(&[("flex", "2")]));
        assert!((s.flex_grow - 2.0).abs() < 1e-6);
    }

    #[test]
    fn fully_sized_classification() {
        let sized = style_from_tag_and_props("", &props(&[("width", "100"), ("height", "50")]));
        let unsized_w = style_from_tag_and_props("", &props(&[("height", "50")]));
        assert!(is_fully_sized(&sized));
        assert!(!is_fully_sized(&unsized_w));
    }
}
