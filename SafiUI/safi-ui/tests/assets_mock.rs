//! `MockAssetLoader` trait conformance + interior-mutability tests (todo 12).

use std::sync::Arc;
use std::thread;

use safi_ui::assets::{AssetError, AssetLoader, MockAssetLoader};

#[test]
fn empty_loader_returns_not_found() {
    let loader = MockAssetLoader::new();
    let err = loader.load_bytes("anything").expect_err("missing");
    assert!(matches!(err, AssetError::NotFound(p) if p == "anything"));
    assert!(!loader.exists("anything"));
    assert!(loader.is_empty());
}

#[test]
fn from_pairs_round_trip() {
    let loader = MockAssetLoader::from_pairs([
        ("assets/ui/screens/home.xml", &b"<Screen/>"[..]),
        ("assets/images/logo.png", &b"\x89PNG"[..]),
    ]);
    assert_eq!(loader.len(), 2);
    assert_eq!(
        loader.load_bytes("assets/ui/screens/home.xml").unwrap(),
        b"<Screen/>"
    );
    assert!(loader.exists("assets/images/logo.png"));
}

#[test]
fn insert_after_construction() {
    let loader = MockAssetLoader::new();
    loader.insert("assets/late.bin", vec![1u8, 2, 3]);
    assert_eq!(loader.load_bytes("assets/late.bin").unwrap(), vec![1, 2, 3]);
}

#[test]
fn shared_across_threads() {
    // Sanity check on the Send + Sync bound — image decode (PRD §13) will
    // hand the loader to a worker pool.
    let loader = Arc::new(MockAssetLoader::from_pairs([(
        "assets/images/a.png",
        &b"a"[..],
    )]));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let l = Arc::clone(&loader);
            thread::spawn(move || l.load_bytes("assets/images/a.png").unwrap())
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap(), b"a");
    }
}
