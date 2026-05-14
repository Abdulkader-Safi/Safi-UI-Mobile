//! `FilesystemAssetLoader` host tests (todo 12).

use std::fs;

use safi_ui::assets::{AssetError, AssetLoader, FilesystemAssetLoader};

/// Build a temp dir scoped to one test. `tempfile` isn't a workspace dep
/// yet (and pulling it in for one file would be heavy-handed), so we
/// hand-roll a per-test directory under `target/`.
fn tmpdir(label: &str) -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    p.push(format!("assets_filesystem_{label}"));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create tmpdir");
    p
}

#[test]
fn round_trip_load_bytes() {
    let root = tmpdir("round_trip");
    fs::create_dir_all(root.join("assets/images")).unwrap();
    fs::write(root.join("assets/images/logo.png"), b"\x89PNG\r\n\x1a\n").unwrap();

    let loader = FilesystemAssetLoader::new(&root);
    let bytes = loader.load_bytes("assets/images/logo.png").expect("load");
    assert_eq!(bytes, b"\x89PNG\r\n\x1a\n");
}

#[test]
fn missing_path_returns_not_found() {
    let root = tmpdir("missing");
    let loader = FilesystemAssetLoader::new(&root);
    let err = loader
        .load_bytes("assets/ui/screens/no-such.xml")
        .expect_err("should error");
    assert!(matches!(err, AssetError::NotFound(_)), "got {err:?}");
}

#[test]
fn exists_distinguishes_present_and_absent() {
    let root = tmpdir("exists");
    fs::create_dir_all(root.join("assets/ui/screens")).unwrap();
    fs::write(root.join("assets/ui/screens/home.xml"), b"<Screen/>").unwrap();

    let loader = FilesystemAssetLoader::new(&root);
    assert!(loader.exists("assets/ui/screens/home.xml"));
    assert!(!loader.exists("assets/ui/screens/other.xml"));
}

#[test]
fn rejects_absolute_paths() {
    let root = tmpdir("absolute");
    let loader = FilesystemAssetLoader::new(&root);
    // Absolute path outside the sandbox must not resolve.
    let abs = if cfg!(windows) {
        "C:/Windows/System32/cmd.exe"
    } else {
        "/etc/passwd"
    };
    let err = loader.load_bytes(abs).expect_err("should reject");
    assert!(matches!(err, AssetError::NotFound(_)));
    assert!(!loader.exists(abs));
}

#[test]
fn rejects_parent_dir_traversal() {
    let root = tmpdir("traversal");
    let loader = FilesystemAssetLoader::new(&root);
    let err = loader
        .load_bytes("assets/ui/screens/../../../etc/passwd")
        .expect_err("should reject");
    assert!(matches!(err, AssetError::NotFound(_)));
}

#[test]
fn loader_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FilesystemAssetLoader>();
}
