# Assets and AssetLoader

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned asset model.
:::

A unified `AssetLoader` abstraction in Rust wraps `AAssetManager` (Android) and `Bundle.main` (iOS). All asset paths are relative to the bundled `assets/` root.

## Path conventions

| Path Convention         | Contents                                       |
| ----------------------- | ---------------------------------------------- |
| `assets/ui/screens/`    | Screen XML files                               |
| `assets/ui/components/` | User-defined XML components                    |
| `assets/images/`        | Image assets referenced by `<Image src="...">` |

## Platform mapping

| Platform | Backing API     | Notes                   |
| -------- | --------------- | ----------------------- |
| Android  | `AAssetManager` | APK `assets/` directory |
| iOS      | `Bundle.main`   | `.app` bundle resources |

## Hot-reload assets

In dev mode, hot-reload loads from the **bundled** assets, not a dev-machine network path. To iterate quickly, rebuild the asset bundle (or use the planned [`safi preview`](/guide/tooling/cli) command for desktop iteration).
