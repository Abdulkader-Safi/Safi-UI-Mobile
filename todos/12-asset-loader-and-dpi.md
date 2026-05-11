# 12 — `AssetLoader` abstraction + DPI scaling

**Phase:** 2 — Layout + Parse
**PRD refs:** §7.3, §9.3

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
