//! Asset access + DPI scaling (PRD §7.3, §9.3, todo 12).
//!
//! Two responsibilities live here because the PRD pairs them: a unified
//! [`AssetLoader`] trait that abstracts Android `AAssetManager`, iOS
//! `Bundle.main`, and host filesystem access; and a [`dpi::DpiScale`]
//! wrapper resolved from `SDL_GetDisplayContentScale` at startup.
//!
//! ## Scope (this session)
//!
//! - Trait + [`AssetError`] + path constants
//! - [`FilesystemAssetLoader`] for host tests and dev preview
//! - [`MockAssetLoader`] for unit tests
//! - [`dpi::DpiScale`] wrapper + math
//!
//! The Android JNI and iOS `ObjC` FFI loaders are present as `cfg`-gated
//! stubs (`unimplemented!()`); they will be filled in alongside todo 17
//! (image pipeline) where they're first exercised end-to-end.

use std::fmt;
use std::io;

pub mod dpi;
pub mod filesystem;
pub mod mock;

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

pub use dpi::DpiScale;
pub use filesystem::FilesystemAssetLoader;
pub use mock::MockAssetLoader;

#[cfg(target_os = "android")]
pub use android::AndroidAssetLoader;
#[cfg(target_os = "ios")]
pub use ios::IosAssetLoader;

/// Conventional path prefixes (PRD §9.3). Relative to the loader's root.
pub const SCREENS_DIR: &str = "assets/ui/screens";
/// User-defined XML components (PRD §11.1).
pub const COMPONENTS_DIR: &str = "assets/ui/components";
/// Image assets referenced by `<Image src="...">` (PRD §13).
pub const IMAGES_DIR: &str = "assets/images";

/// Unified asset access (PRD §9.3).
///
/// `Send + Sync` because background image decode (PRD §13) needs to hold
/// a handle on a worker thread. This is distinct from [`Component`],
/// which is deliberately main-thread-only (PRD §6.8).
///
/// [`Component`]: crate::component::Component
pub trait AssetLoader: Send + Sync {
    /// Load the entire asset into memory. Paths are relative to the
    /// loader's root and use forward slashes regardless of host OS
    /// (the loader rewrites separators internally where needed).
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError>;

    /// Cheap existence check. Implementations should not read the body.
    fn exists(&self, path: &str) -> bool;
}

/// Errors emitted by [`AssetLoader`] implementations.
///
/// Platform-specific variants (`Jni`, ``ObjC``) will be added when the
/// Android/iOS loaders are filled in. The current variants are enough
/// for the host implementations to be lossless.
#[derive(Debug)]
#[non_exhaustive]
pub enum AssetError {
    /// The requested path does not exist within the loader's root.
    NotFound(String),
    /// Underlying IO error (filesystem, JNI handle, `ObjC` bridge).
    Io(io::Error),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "asset not found: {p}"),
            Self::Io(e) => write!(f, "asset IO error: {e}"),
        }
    }
}

impl std::error::Error for AssetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::NotFound(_) => None,
        }
    }
}

impl From<io::Error> for AssetError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::NotFound {
            Self::NotFound(e.to_string())
        } else {
            Self::Io(e)
        }
    }
}
