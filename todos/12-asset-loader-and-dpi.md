# 12 — `AssetLoader` abstraction + DPI scaling

**Phase:** 2 — Layout + Parse
**PRD refs:** §7.3, §9.3

**Status:** ✅ Complete — `safi-ui::assets::{AssetLoader, AssetError,
FilesystemAssetLoader, MockAssetLoader, AndroidAssetLoader,
IosAssetLoader, DpiScale, SCREENS_DIR, COMPONENTS_DIR, IMAGES_DIR}`
shipped with 16 host tests. `AndroidAssetLoader` wraps the NDK
`AAssetManager` (constructed via `SDL_GetAndroidActivity` → JNI →
`AAssetManager_fromJava`); `IosAssetLoader` wraps `NSBundle.mainBundle`
via the `objc2` stack. `App::run` resolves `DpiScale` from
`SDL_GetDisplayContentScale` at startup, picks the platform loader via
`cfg(target_os)`, logs an asset-bundle probe, and re-runs
`LayoutEngine::compute` on `SDL_EVENT_DISPLAY_ORIENTATION`.
Pixel-identical render acceptance becomes meaningful with todo 17
(image pipeline).

## Goal

Unified asset access across Android `AAssetManager` and iOS `Bundle.main`, plus DP → physical pixel conversion at the renderer boundary.

## Deliverables

- `safi-ui::assets::AssetLoader` trait: `load_bytes(path) -> Result<Vec<u8>>`, `exists(path) -> bool`
- `AndroidAssetLoader` via JNI to `AAssetManager`
- `IosAssetLoader` via ObjC FFI to `Bundle.main`
- Path conventions per §9.3: `assets/ui/screens/`, `assets/ui/components/`, `assets/images/`
- `DpiScale` resolved from `SDL_GetDisplayContentScale()` at startup and stored on `UIContext`
- All XML coordinates are DP; renderer multiplies by `dpi_scale` when emitting GPU vertices
- Orientation change (`SDL_EVENT_DISPLAY_ORIENTATION`) triggers re-layout

## Dependencies

- `02`, `07`, `10`

## Acceptance

- Same XML file renders pixel-identical on Pixel 8 (2.625x) and iPhone 15 Pro (3.0x) relative to DP-declared dimensions
- Asset path resolution unit-tested on both platforms (mock loaders for host tests)
