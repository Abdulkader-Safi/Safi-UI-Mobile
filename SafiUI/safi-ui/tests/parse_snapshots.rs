//! Integration tests for `safi_ui::parse::XmlParser`.
//!
//! Lives outside the crate so the `vnode!` proc-macro's absolute
//! `::safi_ui::...` paths resolve correctly.

use std::path::PathBuf;
use std::time::Instant;

use safi_ui::parse::XmlParser;
use safi_ui::vnode;

fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/parse");
    p.push(name);
    p
}

fn count_nodes(v: &safi_ui::vnode::VNode) -> usize {
    1 + v.children.iter().map(count_nodes).sum::<usize>()
}

#[test]
fn valid_screen_parses_and_has_50ish_nodes() {
    let v = XmlParser::parse_path(&fixture("valid_screen.xml")).expect("parse");
    assert_eq!(v.tag, "Screen");
    let n = count_nodes(&v);
    assert!(
        n >= 50,
        "fixture should exercise >=50 nodes for the perf gate, got {n}"
    );
}

#[test]
fn xml_and_vnode_macro_produce_identical_trees() {
    let from_xml = XmlParser::parse_str(
        r##"<Screen bg="#0f0f1a"><Button id="cta" label="Tap me" /></Screen>"##,
        "inline.xml",
    )
    .expect("parse");
    let from_macro = vnode! {
        <Screen bg="#0f0f1a">
            <Button id="cta" label="Tap me" />
        </Screen>
    };
    assert_eq!(from_xml, from_macro);
}

#[test]
fn cold_parse_under_5ms_on_50_node_fixture() {
    let path = fixture("valid_screen.xml");
    // Warm: read once so the FS cache is hot. The PRD §17.1 target is the
    // parse time itself, not file IO.
    let _ = XmlParser::parse_path(&path).expect("warm parse");

    // Median of 11 runs to dampen host jitter.
    let mut samples: Vec<u128> = (0..11)
        .map(|_| {
            let t = Instant::now();
            let v = XmlParser::parse_path(&path).expect("parse");
            let elapsed_us = t.elapsed().as_micros();
            std::hint::black_box(v);
            elapsed_us
        })
        .collect();
    samples.sort_unstable();
    let median_us = samples[samples.len() / 2];
    assert!(
        median_us < 5_000,
        "parse median {median_us}µs exceeds 5ms budget (samples = {samples:?})"
    );
}
