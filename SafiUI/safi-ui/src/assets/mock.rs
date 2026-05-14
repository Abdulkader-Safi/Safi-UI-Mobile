//! In-memory [`AssetLoader`] for tests (todo 12).
//!
//! No filesystem touch; backing store is a [`HashMap`]. Used by unit
//! tests that want to exercise asset-consuming code without writing
//! fixtures to disk.

use std::collections::HashMap;
use std::sync::RwLock;

use super::{AssetError, AssetLoader};

/// In-memory asset store. Cheap to clone-as-shared via `Arc`.
///
/// Interior `RwLock` lets tests `insert` after handing the loader to
/// the code under test, which is a common pattern when wiring up
/// "decode finishes asynchronously" flows.
pub struct MockAssetLoader {
    entries: RwLock<HashMap<String, Vec<u8>>>,
}

impl MockAssetLoader {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Convenience constructor for tests with a known set of assets.
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Vec<u8>>,
    {
        let map = pairs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self {
            entries: RwLock::new(map),
        }
    }

    /// Insert (or overwrite) an asset.
    pub fn insert(&self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.entries
            .write()
            .expect("MockAssetLoader poisoned")
            .insert(path.into(), bytes.into());
    }

    /// Number of entries currently in the store.
    pub fn len(&self) -> usize {
        self.entries.read().expect("MockAssetLoader poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for MockAssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader for MockAssetLoader {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        let guard = self.entries.read().expect("MockAssetLoader poisoned");
        guard
            .get(path)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(path.to_string()))
    }

    fn exists(&self, path: &str) -> bool {
        self.entries
            .read()
            .expect("MockAssetLoader poisoned")
            .contains_key(path)
    }
}
