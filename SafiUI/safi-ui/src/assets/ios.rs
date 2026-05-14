//! iOS `Bundle.main` loader (PRD §9.2, §9.3).
//!
//! Reads files out of the `.app` bundle via `NSBundle`. The bundle
//! handle is acquired lazily on first use (rather than at construction)
//! because `NSBundle::mainBundle()` is cheap and avoids a startup-order
//! dependency with SDL3's `UIApplicationDelegate`.
//!
//! `NSBundle` and `NSData` are thread-safe per Apple's documentation,
//! so the loader is `Send + Sync` and can be handed to image-decode
//! worker threads (PRD §13).
//!
//! ## Path resolution
//!
//! Asset paths are forward-slash-delimited and rooted at the bundle's
//! `Resources/` directory (PRD §9.3): `assets/ui/screens/home.xml`.
//! `NSBundle::URLForResource:withExtension:subdirectory:` takes the
//! parts split out, so the loader splits the path:
//!
//! - subdirectory = everything up to the last `/`
//! - file basename = everything between the last `/` and the last `.`
//! - extension     = everything after the last `.` (or empty)

use objc2::rc::Retained;
use objc2_foundation::{NSBundle, NSData, NSString};

use super::{AssetError, AssetLoader};

/// Asset loader backed by iOS's `Bundle.main`.
pub struct IosAssetLoader {
    bundle: Retained<NSBundle>,
}

// SAFETY: `NSBundle` is documented as thread-safe and `Retained<T>`
// only forwards `Send`/`Sync` when the inner type is `Send`/`Sync`.
// We assert it here because objc2's auto-derivation is conservative
// and `NSBundle` does not advertise `Send` in its trait bounds. Same
// reasoning the `objc2-foundation` docs apply to `NSString` etc.
unsafe impl Send for IosAssetLoader {}
unsafe impl Sync for IosAssetLoader {}

impl IosAssetLoader {
    /// Construct a loader wrapping `NSBundle.mainBundle`.
    pub fn new() -> Self {
        Self {
            bundle: NSBundle::mainBundle(),
        }
    }

    /// Direct access to the underlying `NSBundle` for callers that
    /// need to look up info-plist keys or auxiliary executables.
    pub fn bundle(&self) -> &NSBundle {
        &self.bundle
    }

    /// Resolve a relative asset path to an absolute `NSString` path
    /// inside the bundle. Returns `None` if the resource doesn't exist.
    fn resolve(&self, rel: &str) -> Option<Retained<NSString>> {
        // Reject absolute paths and `..` traversal — parity with the
        // host filesystem loader.
        if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
            return None;
        }

        // Split `assets/ui/screens/home.xml` into:
        //   subdir = "assets/ui/screens"
        //   name   = "home"
        //   ext    = "xml"
        let (subdir, file) = match rel.rfind('/') {
            Some(idx) => (Some(&rel[..idx]), &rel[idx + 1..]),
            None => (None, rel),
        };
        let (name, ext) = match file.rfind('.') {
            Some(idx) => (&file[..idx], Some(&file[idx + 1..])),
            None => (file, None),
        };

        let ns_name = NSString::from_str(name);
        let ns_ext = ext.map(NSString::from_str);
        let ns_subdir = subdir.map(NSString::from_str);

        // `pathForResource:ofType:inDirectory:` returns the absolute
        // filesystem path inside the bundle, or nil for a missing
        // resource. The selector is marked safe by objc2-foundation.
        self.bundle.pathForResource_ofType_inDirectory(
            Some(&ns_name),
            ns_ext.as_deref(),
            ns_subdir.as_deref(),
        )
    }
}

impl Default for IosAssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader for IosAssetLoader {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        let abs_path = self
            .resolve(path)
            .ok_or_else(|| AssetError::NotFound(path.to_string()))?;

        let data: Retained<NSData> = NSData::dataWithContentsOfFile(&abs_path)
            .ok_or_else(|| AssetError::NotFound(path.to_string()))?;

        Ok(data.to_vec())
    }

    fn exists(&self, path: &str) -> bool {
        self.resolve(path).is_some()
    }
}
