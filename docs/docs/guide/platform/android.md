# Android

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned Android target.
:::

## Spec

| Attribute       | Detail                                                                    |
| --------------- | ------------------------------------------------------------------------- |
| **NDK version** | r25 or later                                                              |
| **API level**   | minSdk 24 (Android 7.0, Vulkan guaranteed), targetSdk 35                  |
| **GPU**         | Vulkan 1.1 via SDL_GPU                                                    |
| **Surface**     | SDL3 manages `ANativeWindow` → Vulkan surface                             |
| **Input**       | SDL3 `SDL_EVENT_FINGER_*` — multi-touch native                            |
| **Keyboard**    | `SDL_EVENT_TEXT_INPUT` + JNI bridge for keyboard height                   |
| **Safe area**   | `WindowInsetsCompat` via JNI → `PlatformBridge::safe_area()`              |
| **DPI**         | `SDL_GetDisplayContentScale()`                                            |
| **Lifecycle**   | `SDL_EVENT_WILL_ENTER_BACKGROUND / FOREGROUND / LOW_MEMORY / TERMINATING` |
| **Build tool**  | `cargo-ndk`                                                               |
| **Assets**      | APK `assets/` dir, accessed via `AAssetManager`                           |

## Why minSdk 24?

Vulkan 1.1 is guaranteed on API 24+. Lower API levels would require runtime Vulkan capability checks and an OpenGL ES fallback path, which is explicitly out of scope.

## Build

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android
cargo ndk -t arm64-v8a -o ./android/app/src/main/jniLibs build --release
```

## Lifecycle handling

The frame loop responds to SDL3 lifecycle events:

| Event                 | Action                     |
| --------------------- | -------------------------- |
| `WillEnterBackground` | `gpu.release_resources()`  |
| `DidEnterForeground`  | `gpu.recreate_resources()` |
| `LowMemory`           | `image_cache.evict_all()`  |
| `Terminating`         | Graceful shutdown          |
