//! Host-filesystem [`AssetLoader`] (todo 12).
//!
//! Used by host tests, the dev preview workflow, and (eventually)
//! `safi preview` from the CLI tool (PRD §20.2). The mobile build
//! never instantiates this — it goes through the Android / iOS
//! loaders instead.

use std::fs;
use std::path::{Path, PathBuf};

use super::{AssetError, AssetLoader};

/// An [`AssetLoader`] rooted at a directory on the host filesystem.
///
/// Paths passed to `load_bytes`/`exists` are joined onto `root`. Absolute
/// paths or paths containing `..` segments are rejected to keep the
/// loader from escaping its sandbox — this matches the asset model on
/// Android and iOS, where neither `AAssetManager` nor `Bundle.main`
/// allows traversal outside the app bundle.
pub struct FilesystemAssetLoader {
    root: PathBuf,
}

impl FilesystemAssetLoader {
    /// Create a new loader rooted at `root`. `root` is not required to
    /// exist at construction time — errors surface lazily on `load_bytes`
    /// so tests can pass a path that will be populated later.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The configured root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn resolve(&self, rel: &str) -> Result<PathBuf, AssetError> {
        let p = Path::new(rel);
        if p.is_absolute()
            || p.components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(AssetError::NotFound(format!("rejected unsafe path: {rel}")));
        }
        Ok(self.root.join(p))
    }
}

impl AssetLoader for FilesystemAssetLoader {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        let abs = self.resolve(path)?;
        fs::read(&abs).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AssetError::NotFound(path.to_string())
            } else {
                AssetError::Io(e)
            }
        })
    }

    fn exists(&self, path: &str) -> bool {
        let Ok(abs) = self.resolve(path) else {
            return false;
        };
        abs.is_file()
    }
}
