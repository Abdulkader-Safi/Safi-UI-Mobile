# `AssetLoader`

:::tip Status: ✅ Shipped (todo 12)
Full implementation across all platforms: trait, host filesystem, mock, real Android `AAssetManager` (via NDK + JNI), real iOS `NSBundle.mainBundle` (via objc2).
:::

Unified asset access (PRD §9.3). Returns raw bytes, leaves decoding to the caller.

## Trait

```rust
pub trait AssetLoader: Send + Sync {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError>;
    fn exists(&self, path: &str) -> bool;
}
```

`Send + Sync` because the background image-decode thread pool (PRD §13) needs to hold a loader handle. `Component` keeps its main-thread-only bound (PRD §6.8).

## Errors

```rust
#[non_exhaustive]
pub enum AssetError {
    NotFound(String),
    Io(std::io::Error),
}
```

`AssetError` implements `std::error::Error` and `From<io::Error>` (mapping `ErrorKind::NotFound` to the `NotFound` variant). It is `#[non_exhaustive]` so platform-specific variants can be added without a breaking release.

The Android-specific `AndroidLoaderInitError` is returned only by `AndroidAssetLoader::from_sdl_activity` — runtime `load_bytes` / `exists` calls collapse to `AssetError::NotFound` regardless of underlying cause.

## Path constants

| Constant         | Value                    |
| ---------------- | ------------------------ |
| `SCREENS_DIR`    | `"assets/ui/screens"`    |
| `COMPONENTS_DIR` | `"assets/ui/components"` |
| `IMAGES_DIR`     | `"assets/images"`        |

Paths passed to `load_bytes` / `exists` are forward-slash-delimited regardless of host OS. Implementations rewrite separators where needed.

## Implementations

### `FilesystemAssetLoader`

Reads from a host directory. Used by host tests, the dev preview workflow, and the future `safi preview` CLI command.

```rust
use safi_ui::assets::FilesystemAssetLoader;

let loader = FilesystemAssetLoader::new("./");
let bytes = loader.load_bytes("assets/ui/screens/home.xml")?;
```

Rejects absolute paths and any path containing `..` segments — matches the sandboxed access model on Android and iOS.

### `MockAssetLoader`

In-memory store backed by `HashMap<String, Vec<u8>>` with interior `RwLock` so tests can insert after handing the loader to the code under test.

```rust
use safi_ui::assets::MockAssetLoader;

let loader = MockAssetLoader::from_pairs([
    ("assets/ui/screens/home.xml", &b"<Screen/>"[..]),
]);
loader.insert("assets/images/late.png", vec![1, 2, 3]);
```

### `AndroidAssetLoader`

Backed by the NDK's `AAssetManager`. Constructed by `App::run` via `AndroidAssetLoader::from_sdl_activity()`, which:

1. Calls `SDL_GetAndroidJNIEnv()` for the `JNIEnv*`
2. Calls `SDL_GetAndroidActivity()` for the `Activity` jobject
3. Calls `activity.getAssets()` via JNI for the Java `AssetManager`
4. Converts to a native `AAssetManager*` via `AAssetManager_fromJava` from `ndk-sys`

The loader wraps the resulting `AssetManager` in an `Arc` so worker-thread clones share the underlying NDK handle. `AAssetManager` is documented as thread-safe; the `AAsset*` returned by `open()` is not — `load_bytes` reads each asset to completion and drops it before returning.

For raw-pointer construction (testing or advanced cases), `AndroidAssetLoader::from_raw(*mut AAssetManager)` is available behind an `unsafe` boundary.

### `IosAssetLoader`

Backed by `NSBundle.mainBundle`. Constructed eagerly in `App::run` via `IosAssetLoader::new()` — no failure path because `NSBundle.mainBundle` always returns a valid bundle.

Path resolution splits the dot-extension and last `/` into the three arguments `NSBundle.pathForResource(ofType:inDirectory:)` expects, then `NSData.dataWithContentsOfFile:` reads bytes off disk. Returns `AssetError::NotFound` if the resource isn't in the bundle or the file can't be read.

Both `NSBundle` and `NSData` are documented as thread-safe; the loader is `Send + Sync` accordingly.

## See also

- [`DpiScale`](./asset-loader#dpi-scaling) — paired with `AssetLoader` because §7.3 and §9.3 are intertwined in the PRD
- [Image pipeline (todo 17)](/guide/platform/images) — the next consumer of `AssetLoader::load_bytes`
- [PRD §9.3](https://github.com/Abdulkader-Safi/Safi-UI-Mobile/blob/main/PRD.md) — Unified `AssetLoader`

## DPI scaling

`safi_ui::assets::DpiScale` wraps the raw `f32` from `SDL_GetDisplayContentScale`.

```rust
use safi_ui::assets::DpiScale;

let scale = DpiScale::from_sdl(3.0);          // iPhone 15 Pro
let phys  = scale.dp_to_physical(100.0);      // 300.0
let dp    = scale.physical_to_dp(300.0);      // 100.0
```

`DpiScale::from_sdl` clamps `0.0`, negatives, `NaN`, and `±∞` to `1.0`. The `App` runtime resolves it once at startup and logs the result as `safi-ui::app: dpi_scale = …`.
