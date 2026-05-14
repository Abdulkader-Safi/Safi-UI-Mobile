//! Android `AAssetManager` loader (PRD §9.1, §9.3).
//!
//! Reads files out of the APK's `assets/` directory via the NDK's
//! `AAssetManager` API. The handle is acquired once at startup from
//! the SDL3 Android activity (`SDL_AndroidGetActivity` →
//! `AAssetManager_fromJava`) and held for the process lifetime.
//!
//! `AAssetManager` is documented as thread-safe so the loader is
//! `Send + Sync` and can be cloned/handed to image-decode worker
//! threads (PRD §13). The `Asset` returned from `open()` is **not**
//! thread-safe; we read it to completion and drop it inside the
//! `load_bytes` call, never holding it across threads.

use std::ffi::CString;
use std::io::Read;
use std::ptr::NonNull;
use std::sync::Arc;

#[cfg(feature = "runtime")]
use jni::objects::JObject;
#[cfg(feature = "runtime")]
use jni::JNIEnv;
use ndk::asset::AssetManager;
use ndk_sys::AAssetManager;

use super::{AssetError, AssetLoader};

/// Error constructing an [`AndroidAssetLoader`] from the SDL3 activity.
#[cfg(feature = "runtime")]
#[derive(Debug)]
pub enum AndroidLoaderInitError {
    /// `SDL_GetAndroidJNIEnv` returned null. SDL3 not yet initialised on
    /// this thread.
    NoJniEnv,
    /// `SDL_GetAndroidActivity` returned null. App lifecycle race.
    NoActivity,
    /// A JNI call (typically `Activity.getAssets`) threw.
    Jni(jni::errors::Error),
    /// `AAssetManager_fromJava` returned null. The `AssetManager` jobject
    /// wasn't a real `android.content.res.AssetManager`.
    BadAssetManager,
}

#[cfg(feature = "runtime")]
impl std::fmt::Display for AndroidLoaderInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoJniEnv => f.write_str("SDL_GetAndroidJNIEnv returned null"),
            Self::NoActivity => f.write_str("SDL_GetAndroidActivity returned null"),
            Self::Jni(e) => write!(f, "JNI error obtaining AssetManager: {e}"),
            Self::BadAssetManager => f.write_str("AAssetManager_fromJava returned null"),
        }
    }
}

#[cfg(feature = "runtime")]
impl std::error::Error for AndroidLoaderInitError {}

#[cfg(feature = "runtime")]
impl From<jni::errors::Error> for AndroidLoaderInitError {
    fn from(e: jni::errors::Error) -> Self {
        Self::Jni(e)
    }
}

/// Asset loader backed by Android's `AAssetManager`.
///
/// Cloning yields a new handle to the same underlying manager — the
/// inner `Arc<AssetManager>` ensures the NDK pointer outlives every
/// clone. The image-decode worker pool gets its own clone rather than
/// borrowing through a lock.
#[derive(Clone)]
pub struct AndroidAssetLoader {
    manager: Arc<AssetManager>,
}

impl AndroidAssetLoader {
    /// Wrap a raw `AAssetManager*` pointer from SDL3 / JNI.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid, non-null pointer to an `AAssetManager`
    /// owned by the live Android `Activity`. The activity is expected
    /// to outlive the loader (in practice, the loader is owned by
    /// `App` which is held for the process lifetime, and the activity
    /// is owned by the OS).
    pub unsafe fn from_raw(ptr: *mut AAssetManager) -> Option<Self> {
        let nn = NonNull::new(ptr)?;
        // SAFETY: caller asserts pointer validity per the doc-comment.
        let manager = unsafe { AssetManager::from_ptr(nn) };
        Some(Self {
            manager: Arc::new(manager),
        })
    }

    /// Construct from the SDL3 Android activity. SDL3 must have been
    /// initialised before calling — `App::run` orders this after
    /// `sdl3::init`. Performs the JNI dance:
    ///
    /// 1. `SDL_GetAndroidJNIEnv()` → `JNIEnv*`
    /// 2. `SDL_GetAndroidActivity()` → `jobject` (the `Activity`)
    /// 3. `activity.getAssets()` → `jobject` (the Java `AssetManager`)
    /// 4. `AAssetManager_fromJava(env, jobject)` → `*mut AAssetManager`
    ///
    /// The returned loader holds the NDK-side handle; the Java-side
    /// local ref is released as the helper returns.
    ///
    /// Requires `feature = "runtime"` because it depends on the same
    /// `sdl3` crate that the App runtime pulls in.
    #[cfg(feature = "runtime")]
    pub fn from_sdl_activity() -> Result<Self, AndroidLoaderInitError> {
        // SAFETY: SDL3 promises both functions are safe to call from
        // any thread once SDL is initialised; both return null on
        // failure rather than aborting.
        let env_ptr =
            unsafe { sdl3::sys::system::SDL_GetAndroidJNIEnv() }.cast::<jni::sys::JNIEnv>();
        if env_ptr.is_null() {
            return Err(AndroidLoaderInitError::NoJniEnv);
        }
        let activity_raw = unsafe { sdl3::sys::system::SDL_GetAndroidActivity() };
        if activity_raw.is_null() {
            return Err(AndroidLoaderInitError::NoActivity);
        }

        // SAFETY: `env_ptr` came from SDL3 and is valid for this
        // thread (SDL3 attaches the JNI env per-thread internally).
        let mut env = unsafe { JNIEnv::from_raw(env_ptr) }?;
        // SAFETY: `activity_raw` is a non-null jobject owned by SDL3.
        // We take a borrow, not ownership — `Activity` outlives us.
        let activity = unsafe { JObject::from_raw(activity_raw.cast()) };

        let asset_manager = env
            .call_method(
                &activity,
                "getAssets",
                "()Landroid/content/res/AssetManager;",
                &[],
            )?
            .l()?;

        // Convert the Java `AssetManager` reference into an NDK
        // `AAssetManager*`. The pointer remains valid as long as the
        // Java reference is alive — SDL3 holds the activity (and
        // therefore the asset manager it returned) for the process
        // lifetime, so we can stash the C pointer in `App`.
        let aam_ptr = unsafe {
            ndk_sys::AAssetManager_fromJava(env.get_raw().cast(), asset_manager.as_raw().cast())
        };

        // SDL3 hands us a local ref to the activity; release it
        // before returning. `asset_manager` is also a local ref and
        // gets cleaned up by JNI when the frame pops, but explicitly
        // dropping it is harmless.
        let _ = env.delete_local_ref(activity);
        let _ = env.delete_local_ref(asset_manager);

        let nn = NonNull::new(aam_ptr).ok_or(AndroidLoaderInitError::BadAssetManager)?;
        // SAFETY: `AAssetManager_fromJava` returns a valid pointer
        // when non-null per the NDK contract.
        let manager = unsafe { AssetManager::from_ptr(nn) };
        Ok(Self {
            manager: Arc::new(manager),
        })
    }

    /// Direct access to the underlying [`AssetManager`] for callers
    /// that need NDK-native APIs (e.g. enumerating an asset directory).
    pub fn ndk_manager(&self) -> &AssetManager {
        &self.manager
    }

    fn open(&self, path: &str) -> Result<ndk::asset::Asset, AssetError> {
        let c_path = CString::new(path).map_err(|_| AssetError::NotFound(path.to_string()))?;
        self.manager
            .open(&c_path)
            .ok_or_else(|| AssetError::NotFound(path.to_string()))
    }
}

impl AssetLoader for AndroidAssetLoader {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError> {
        let mut asset = self.open(path)?;
        // `AAsset_getLength` is a 0-cost call; pre-sizing the Vec
        // avoids two-stage reallocation for the common large-asset
        // case (images, fonts).
        let mut buf = Vec::with_capacity(asset.length());
        asset.read_to_end(&mut buf).map_err(AssetError::Io)?;
        Ok(buf)
    }

    fn exists(&self, path: &str) -> bool {
        let Ok(c_path) = CString::new(path) else {
            return false;
        };
        // `AAssetManager_open` returns null when the asset is missing
        // and a valid `AAsset*` (which we immediately drop) when it
        // exists. There is no separate "stat" API on AAssetManager.
        self.manager.open(&c_path).is_some()
    }
}
