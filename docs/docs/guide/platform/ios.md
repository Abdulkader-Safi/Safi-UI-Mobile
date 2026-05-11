# iOS

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned iOS target.
:::

## Spec

| Attribute               | Detail                                              |
| ----------------------- | --------------------------------------------------- |
| **Minimum iOS version** | 16.0                                                |
| **GPU**                 | Metal via SDL_GPU                                   |
| **Surface**             | SDL3 manages `CAMetalLayer` → Metal surface         |
| **Input**               | SDL3 `SDL_EVENT_FINGER_*` — UITouch bridged by SDL3 |
| **Keyboard**            | `UIKeyboardWillShowNotification` via ObjC bridge    |
| **Safe area**           | `UIView.safeAreaInsets` via ObjC bridge             |
| **DPI**                 | `SDL_GetDisplayContentScale()`                      |
| **Orientation**         | `SDL_EVENT_DISPLAY_ORIENTATION` → Taffy re-layout   |
| **Build tool**          | `cargo-xcode` or `cargo-mobile2`                    |
| **Assets**              | `.app` bundle, accessed via `Bundle.main`           |

## No MoltenVK

Safi-UI uses native Metal through SDL_GPU, **not** MoltenVK. There is no Vulkan shim, no translation layer, and no SPIR-V → MSL conversion at runtime. MSL shaders are pre-compiled at build time via `glslc`.

## Build

```bash
cargo install cargo-mobile2
cargo mobile init
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --release
```

## Orientation changes

`SDL_EVENT_DISPLAY_ORIENTATION` triggers a Taffy re-layout pass with the new viewport dimensions. The font atlas is rebuilt if the DPI scale also changes (rare, but happens on external display attach).
