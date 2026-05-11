# Testing the SDL3 window smoke test on devices

This is the device-verification guide for todo `02 — SDL3 window on Android + iOS`. The Rust code, Android Gradle project, and Xcode project all live in this directory; nothing here builds until you run the per-platform `./build.sh` first.

You should end up with:

- A solid **`#4f8ef7`** (blue) window filling the screen
- A log line confirming the correct GPU backend: **`vulkan`** on Android, **`metal`** on iOS
- The app surviving a background → foreground cycle without crashing

---

## iOS — Xcode (real iPhone)

> :warning: **The iOS Simulator cannot run SDL_GPU Metal.** The simulator's
> virtual Metal device reports a GPU family lower than `MTLGPUFamilyApple3`,
> and SDL_GPU's Metal backend refuses to initialize on it with the error
> `Device does not meet the hardware requirements for SDL_GPU Metal`. This
> is an SDL3 limitation, not our bug. **Use a real iPhone (A11 / iPhone 8
> or newer)** for the smoke test. The simulator is fine for app launch /
> Swift / linking validation but not for the SDL_GPU layer.

### One-time setup

```bash
rustup target add aarch64-apple-ios       # real device
rustup target add aarch64-apple-ios-sim   # simulator (build-only, won't reach SDL_GPU)
```

### Build (for real device)

```bash
cd SafiUI/examples/window-smoke/ios
./build.sh device
```

`./build.sh device` does three things:

1. `cargo build -p safi-ui-window-smoke --features device-build --target aarch64-apple-ios` → `target/aarch64-apple-ios/debug/libsafi_ui_window_smoke.a`
2. Stages it under `Sources/build/iphoneos/libsafi_ui_window_smoke.a`
3. Copies SDL3's vendored headers into `build/sdl3-include/SDL3/`

For simulator-only build validation (verifies linking, won't render):

```bash
./build.sh sim   # default if no arg given
```

> **Cold first run: 5–10 min** because SDL3 CMake-builds from source. Cached after.

### Run on a real iPhone

1. Plug in your iPhone, tap "Trust this computer" on the phone
2. `open WindowSmoke.xcodeproj`
3. Xcode → select **WindowSmoke** target → **Signing & Capabilities** → pick a development team (free Apple ID works fine)
4. Select your iPhone in the scheme bar (not a simulator)
5. **Cmd+R**
6. First time: iOS will block the app — go to **Settings → General → VPN & Device Management** on the phone and trust your developer profile, then tap the app icon on Springboard

### What to expect

- Xcode console (bottom panel) — look for:
  ```
  safi-ui-window-smoke: SDL_GPU driver = metal
  safi-ui-window-smoke: first frame submitted
  ```
- The phone screen should be solid **`#4f8ef7`** (a recognizable blue)
- Background test: tap the Home indicator, wait 2 seconds, tap the app icon to foreground. Log should show:
  ```
  safi-ui-window-smoke: AppWillEnterBackground
  safi-ui-window-smoke: AppDidEnterBackground
  safi-ui-window-smoke: AppWillEnterForeground
  safi-ui-window-smoke: AppDidEnterForeground
  ```

---

## Android — Android Studio (emulator or real device)

### One-time setup

```bash
# 1. Install NDK r26d (one-time)
#    Open Android Studio → More Actions → SDK Manager → SDK Tools tab
#    → check "NDK (Side by side)" → select 26.3.11579264 → Apply
#
#    Or via the command-line SDK manager:
$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager "ndk;26.3.11579264"

# 2. Install cargo-ndk
cargo install cargo-ndk

# 3. Add the Android Rust target
rustup target add aarch64-linux-android
```

### Build

```bash
cd SafiUI/examples/window-smoke/android
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.3.11579264
./build.sh
```

`./build.sh` does three things:

1. `cargo ndk -t arm64-v8a build -p safi-ui-window-smoke --features device-build` → `libsafi_ui_window_smoke.so` for arm64-v8a
2. Copies SDL3's vendored Android Java glue (`SDLActivity.java`, `HIDDeviceManager.java`, etc.) out of the cargo cache into `app/src/main/java/org/libsdl/app/` so Gradle can compile it
3. `./gradlew assembleDebug` → `app/build/outputs/apk/debug/app-debug.apk`

> **Cold first run: 5–10 min** for SDL3 + first Gradle sync. Cached after.

### Run — Path A: Android Studio (GUI)

```bash
open -a "Android Studio" .   # opens this folder as a project
```

1. Android Studio syncs Gradle (first time: ~2 min)
2. Top bar: select a device. If you don't have one, open **Device Manager** → **Create Virtual Device** → pick an arm64 system image with **API 34+** (must be Vulkan-capable; most modern images are)
3. Click the green **▶ Run** button
4. **Logcat** (bottom panel) — set filter to `package:com.safiui.windowsmoke`. Look for:
   ```
   safi-ui-window-smoke: SDL_GPU driver = Some("vulkan")
   ```
5. The screen should be solid **`#4f8ef7`**
6. Background test: tap the **Home** button, then the app switcher → return. Logcat should show the same four lifecycle lines as iOS above.

### Run — Path B: command line only

```bash
# install
$ANDROID_HOME/platform-tools/adb install -r app/build/outputs/apk/debug/app-debug.apk

# launch
$ANDROID_HOME/platform-tools/adb shell am start -n com.safiui.windowsmoke/.MainActivity

# watch logs
$ANDROID_HOME/platform-tools/adb logcat -s "safi-ui-window-smoke" "SDL"
```

### Real Android phone (instead of emulator)

1. On the phone: **Settings → About phone → tap Build number 7 times** to enable Developer options
2. **Developer options → enable USB debugging**
3. Plug the phone in, tap "Allow" on the USB-debugging prompt
4. `adb devices` should list your phone
5. Same `./build.sh` + `adb install` / Android Studio Run

---

## Acceptance checklist (todo 02)

For **each** platform you test on, report three booleans + the driver string back:

| Check                            | What to look for                                                             |
| -------------------------------- | ---------------------------------------------------------------------------- |
| `SDL_GPU driver`                 | Must be `vulkan` on Android, `metal` on iOS. Anything else = fallback issue. |
| Blue `#4f8ef7` window appeared   | y / n                                                                        |
| Background → foreground survived | y / n + paste the four `App…ground` log lines                                |

---

## Likely first-run gotchas

| Symptom                                                  | Fix                                                                                                                                     |
| -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| iOS: `'SDL3/SDL_main.h' file not found`                  | `./build.sh` hasn't run yet — it populates `build/sdl3-include/`.                                                                       |
| iOS: `Code signing required`                             | Xcode → target → Signing & Capabilities → pick a team. For simulator-only runs you can set `CODE_SIGNING_ALLOWED=NO` in build settings. |
| Android: `org.libsdl.app.SDLActivity cannot be resolved` | `./build.sh` failed to copy SDL3's Java glue. Check `~/.cargo/registry/src/index.crates.io-*/sdl3-src-*/SDL/android-project/` exists.   |
| Android: `ANDROID_NDK_HOME is not set`                   | Export it before `./build.sh`.                                                                                                          |
| Android emulator: `Vulkan not supported`                 | Pick an arm64 system image on Apple Silicon Macs, or a recent x86_64 image on Intel. Older images don't ship Vulkan.                    |
| Cold build is slow                                       | First SDL3 CMake compile is ~5–10 min. Cached after.                                                                                    |
| `cargo-ndk: command not found`                           | `cargo install cargo-ndk`.                                                                                                              |
| Android Studio: Gradle JDK error                         | Settings → Build, Execution, Deployment → Build Tools → Gradle → set Gradle JDK to one ≥ 17.                                            |

---

## When something fails

Paste the full Xcode console output (iOS) or `adb logcat` (Android) and we'll iterate. Don't proceed to todo 03 until both platforms tick all three checks — every later phase assumes the smoke test works.
