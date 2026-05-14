//! `DpiScale` math + SDL sanitisation tests (todo 12, PRD §7.3).

use safi_ui::assets::DpiScale;

const EPS: f32 = 1e-4;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

#[test]
fn identity_scale_round_trips() {
    let s = DpiScale::ONE;
    assert!(approx(s.dp_to_physical(100.0), 100.0));
    assert!(approx(s.physical_to_dp(100.0), 100.0));
}

#[test]
fn pixel_8_reference_value() {
    // PRD §7.3 reference: Pixel 8 @ 420dpi → scale 2.625.
    let s = DpiScale::from_sdl(2.625);
    assert!(approx(s.dp_to_physical(100.0), 262.5));
    assert!(approx(s.physical_to_dp(262.5), 100.0));
}

#[test]
fn iphone_15_pro_reference_value() {
    // PRD §7.3 reference: iPhone 15 Pro → scale 3.0.
    let s = DpiScale::from_sdl(3.0);
    assert!(approx(s.dp_to_physical(100.0), 300.0));
    assert!(approx(s.physical_to_dp(300.0), 100.0));
}

#[test]
fn from_sdl_clamps_zero_to_one() {
    // SDL can return 0.0 if the display has no content-scale information.
    assert_eq!(DpiScale::from_sdl(0.0), DpiScale::ONE);
    assert_eq!(DpiScale::from_sdl(-2.0), DpiScale::ONE);
    assert_eq!(DpiScale::from_sdl(f32::NAN), DpiScale::ONE);
    assert_eq!(DpiScale::from_sdl(f32::INFINITY), DpiScale::ONE);
}

#[test]
fn default_is_one() {
    assert_eq!(DpiScale::default(), DpiScale::ONE);
}

#[test]
fn raw_and_into_f32_match() {
    let s = DpiScale::from_sdl(2.625);
    assert!(approx(s.raw(), 2.625));
    let raw: f32 = s.into();
    assert!(approx(raw, 2.625));
}
