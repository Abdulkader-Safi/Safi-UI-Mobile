# Assets and AssetLoader

:::tip Status: ✅ Shipped (todo 12)
Full implementation — `AssetLoader` trait, `FilesystemAssetLoader` (host), `MockAssetLoader` (tests), `AndroidAssetLoader` (real `AAssetManager` via JNI), `IosAssetLoader` (real `NSBundle.mainBundle` via objc2). DPI scaling resolved from `SDL_GetDisplayContentScale` at `App::run` startup.
:::

A unified `AssetLoader` abstraction in Rust wraps `AAssetManager` (Android), `Bundle.main` (iOS), and the host filesystem (host tests and `safi preview`). All asset paths are relative to the bundled `assets/` root.

## The trait

```rust
use safi_ui::assets::{AssetLoader, AssetError};

pub trait AssetLoader: Send + Sync {
    fn load_bytes(&self, path: &str) -> Result<Vec<u8>, AssetError>;
    fn exists(&self, path: &str) -> bool;
}
```

`Send + Sync` because background image decode (PRD §13) hands the loader to a worker thread pool. This is the carve-out from `Component`'s main-thread-only model (PRD §6.8).

`AssetError` is `#[non_exhaustive]`. Today it surfaces `NotFound(String)` and `Io(std::io::Error)`. Platform-specific variants (JNI / ObjC) can be added without a breaking release.

## Path conventions

| Constant                              | Path                    | Contents                                       |
| ------------------------------------- | ----------------------- | ---------------------------------------------- |
| `safi_ui::assets::SCREENS_DIR`        | `assets/ui/screens/`    | Screen XML files                               |
| `safi_ui::assets::COMPONENTS_DIR`     | `assets/ui/components/` | User-defined XML components                    |
| `safi_ui::assets::IMAGES_DIR`         | `assets/images/`        | Image assets referenced by `<Image src="...">` |

## Platform mapping

| Platform | Backing API     | Type                                        | How it's constructed                                                                 |
| -------- | --------------- | ------------------------------------------- | ------------------------------------------------------------------------------------ |
| Android  | `AAssetManager` | `safi_ui::assets::AndroidAssetLoader`       | `App::run` → `AndroidAssetLoader::from_sdl_activity()` (SDL3 Activity → JNI → NDK)   |
| iOS      | `Bundle.main`   | `safi_ui::assets::IosAssetLoader`           | `App::run` → `IosAssetLoader::new()` (`NSBundle.mainBundle` via objc2)               |
| Host     | filesystem      | `safi_ui::assets::FilesystemAssetLoader`    | `App::run` → `FilesystemAssetLoader::new(".")` (and explicit construction for tests) |
| Tests    | `HashMap`       | `safi_ui::assets::MockAssetLoader`          | In-memory; cheap to clone via `Arc`                                                  |

## Bundling assets in a Safi-UI app

Authoring source-of-truth: keep your XML / images under `assets/` at the root of your app crate:

```
my-app/
  Cargo.toml
  src/
    app.rs
  assets/
    ui/
      screens/
        home.xml
      components/
        UserCard.xml
    images/
      logo.png
```

The runtime path conventions (`assets/ui/screens/`, etc.) match this layout exactly. The bundler-side work is per-platform.

### Android

Anything under `android/app/src/main/assets/` is packaged into the APK at `assets/` automatically by Gradle. The simplest pattern is a `prepareAssets` Gradle task that copies (or symlinks) `<project>/assets/` to `android/app/src/main/assets/` before each build. The `examples/window-smoke` Android project follows this convention.

### iOS

Add `assets/` as a **folder reference** (blue folder, not yellow group) in your Xcode project. Folder references preserve directory hierarchy as the bundle resource subdir — the same `assets/ui/screens/home.xml` path then works on both platforms unchanged.

In Xcode: File → Add Files → select the `assets/` directory → check "Create folder references". The directory then appears in the **Resources** build phase.

## Usage

```rust
use safi_ui::assets::{AssetLoader, FilesystemAssetLoader};

let loader = FilesystemAssetLoader::new("./");
let xml = loader.load_bytes("assets/ui/screens/home.xml")?;

if loader.exists("assets/images/logo.png") {
    // ...
}
```

The filesystem loader rejects absolute paths and paths containing `..` segments — host-side parity with the sandboxed access model on Android and iOS.

## DPI scaling

XML coordinates are authored in dp (density-independent pixels). The runtime resolves a `DpiScale` from `SDL_GetDisplayContentScale` at startup:

```rust
use safi_ui::assets::DpiScale;

let scale = DpiScale::from_sdl(2.625);   // Pixel 8
assert_eq!(scale.dp_to_physical(100.0), 262.5);
assert_eq!(scale.physical_to_dp(262.5),  100.0);
```

Reference values:

| Device           | `DpiScale` |
| ---------------- | ---------- |
| Pixel 8 (420dpi) | 2.625      |
| iPhone 15 Pro    | 3.0        |
| Desktop 1080p    | 1.0        |

`DpiScale::from_sdl` clamps non-finite, zero, and negative inputs to `1.0` — SDL3 returns `0.0` when the display has no content-scale information, and the runtime should keep painting rather than panic.

The orientation event (`SDL_EVENT_DISPLAY_ORIENTATION`) triggers a re-layout in `App::run`. The render-logical canvas stays pinned at `480×800` dp.

## Hot-reload assets

In dev mode, hot-reload loads from the **bundled** assets, not a dev-machine network path. To iterate quickly, rebuild the asset bundle (or use the planned [`safi preview`](/guide/tooling/cli) command for desktop iteration).
