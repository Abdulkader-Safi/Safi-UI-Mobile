# Android

:::info Status: Smoke test landed (todo 02)
The Android host project, JNI bridge, and SDL3 + `SDL_GPU` window creation are wired up in `SafiUI/examples/window-smoke/`. Device verification (Vulkan driver confirmed in Logcat, background/foreground survives) is the user's hand-off. The full PRD §9.1 platform bridge (safe area, keyboard height, asset loader) lands in later todos.
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

## Reproducing the smoke test

Found in `SafiUI/examples/window-smoke/android/`. The Rust crate compiles as `crate-type = ["cdylib"]`, statically links SDL3 (via `sdl3` crate's `build-from-source-static` feature), and exports `SDL_main` through `#[sdl3_main::main]`. The Kotlin host is a single `SDLActivity` subclass that loads `libsafi_ui_window_smoke.so`.

```bash
cd SafiUI/examples/window-smoke/android
export ANDROID_NDK_HOME=/path/to/ndk/r26d
./build.sh         # cargo ndk build + gradle assembleDebug
./gradlew installDebug
adb shell am start -n com.safiui.windowsmoke/.MainActivity
adb logcat -s SDL safi-ui-window-smoke
```

Expected logcat output:

```
safi-ui-window-smoke: SDL_GPU driver = Some("vulkan")
```

Any other driver (`gles2`, `gles3`, `software`) means SDL3 fell back. The Vulkan feature filter in `AndroidManifest.xml` prevents installation on devices that lack Vulkan 1.1, so the fallback should never happen in practice on API 24+.

## Lifecycle handling

The frame loop responds to SDL3 lifecycle events:

| Event                 | Action                     |
| --------------------- | -------------------------- |
| `WillEnterBackground` | `gpu.release_resources()`  |
| `DidEnterForeground`  | `gpu.recreate_resources()` |
| `LowMemory`           | `image_cache.evict_all()`  |
| `Terminating`         | Graceful shutdown          |
