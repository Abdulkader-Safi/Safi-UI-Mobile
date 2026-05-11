# iOS

:::info Status: Smoke test landed (todo 02)
The iOS host project (`WindowSmoke.xcodeproj`), main.m bridge, and SDL3 + `SDL_GPU` window creation are wired up in `SafiUI/examples/window-smoke/ios/`. Device verification (Metal driver confirmed in Xcode console, background/foreground survives) is the user's hand-off. The full PRD §9.2 platform bridge (safe area, keyboard, orientation) lands in later todos.
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
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --release
```

## Reproducing the smoke test

:::warning iOS Simulator cannot run SDL_GPU Metal
The simulator's virtual Metal device reports a GPU family below `MTLGPUFamilyApple3`, and SDL_GPU refuses to initialize on it (`Device does not meet the hardware requirements for SDL_GPU Metal`). The smoke test must be run on a real iPhone (A11 / iPhone 8 or newer). Simulator builds still validate the Swift bridge, linker setup, and SDL3 init, but cannot reach the GPU.
:::

Found in `SafiUI/examples/window-smoke/ios/`. The Rust crate compiles as `crate-type = ["staticlib"]` with SDL3 statically linked in (`sdl3` crate `build-from-source-static`). The Xcode project links `libsafi_ui_window_smoke.a` and uses a single-file **Swift** bridge (`Sources/main.swift` with `@_cdecl("main")`) calling `SDL_RunApp` to hand control over to the Rust-exported `SDL_main`.

```bash
cd SafiUI/examples/window-smoke/ios
./build.sh           # cargo build + xcodebuild
# Then in Xcode: open WindowSmoke.xcodeproj, select an iPhone simulator,
# Cmd+R.
```

Expected Xcode console output:

```
safi-ui-window-smoke: SDL_GPU driver = Some("metal")
```

The `UIRequiredDeviceCapabilities` array in `Info.plist` includes `metal`, so the app refuses to install on devices that don't support it.

## Orientation changes

`SDL_EVENT_DISPLAY_ORIENTATION` triggers a Taffy re-layout pass with the new viewport dimensions. The font atlas is rebuilt if the DPI scale also changes (rare, but happens on external display attach).
