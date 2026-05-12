# scripts/

Flutter-style one-shot device runners. Each script boots a device/simulator
(if none is up), builds the example, installs, launches, and tails logs.

These wrap the per-example `build.sh` scripts under `examples/<name>/{android,ios}/`.
They are dev-machine-only — CI doesn't run them. Long-term these are replaced
by `safi dev --target android|ios` (PRD §20, todo 33).

## Quickstart

```bash
./scripts/dev-android.sh           # default example: window-smoke
./scripts/dev-ios.sh               # default simulator: iPhone 15 Pro
```

## `dev-android.sh`

```bash
./scripts/dev-android.sh [example] [--release] [--clean]
                        [--avd <name>] [--device <serial>]
                        [--no-launch] [--no-logs]
```

Picks a device in this order:

1. `--device <serial>` if provided
2. First device shown by `adb devices`
3. AVD named via `--avd`, else the first AVD from `emulator -list-avds`

Boots the emulator with `-no-snapshot -no-audio -no-boot-anim`, waits for
`sys.boot_completed`, hands off to `examples/<example>/android/build.sh`,
installs the APK, launches via `am start … LAUNCHER`, then tails
`adb logcat -s "safi-ui-window-smoke" "SDL"`.

### Prerequisites

- `$ANDROID_HOME` (auto-detected on macOS / Linux), with `platform-tools/adb`,
  `emulator/emulator`, `build-tools/*/aapt`
- NDK r26+ (`$ANDROID_NDK_HOME` auto-detected by the underlying `build.sh`)
- `cargo install cargo-ndk`
- `rustup target add aarch64-linux-android`
- At least one arm64 AVD (Pixel 8 API 35 recommended), or a USB-connected phone

## `dev-ios.sh`

```bash
./scripts/dev-ios.sh [example] [--release] [--clean]
                    [--simulator "iPhone 15 Pro"]
                    [--no-launch] [--no-logs]
```

Boots the named simulator (idempotent), opens Simulator.app, builds via
`examples/<example>/ios/build.sh sim`, finds the produced `.app` under
`~/Library/Developer/Xcode/DerivedData/`, reads `CFBundleIdentifier`, then
`xcrun simctl install` + `xcrun simctl launch --console-pty` streams the
app's stdout/stderr into the terminal.

### ⚠️ SDL_GPU on the Simulator

The iOS Simulator's virtual Metal device does not satisfy SDL_GPU's
`MTLGPUFamilyApple3` requirement. The app launches and lifecycle logs
flow, but render passes error out. Use a **real iPhone via Xcode** for
visual verification (see `../examples/window-smoke/TESTING.md`).

### Prerequisites

- Xcode 15+ with command-line tools (`xcode-select --install`)
- An iOS Simulator runtime installed (Xcode → Settings → Platforms)
- `rustup target add aarch64-apple-ios-sim`

## Real device deployment

These scripts deliberately do **not** automate real-iPhone deployment —
Apple Developer signing makes that Xcode territory until `safi dev --target ios`
lands as part of the v1.1 CLI (todo 33). For now:

- iOS device: open `examples/window-smoke/ios/WindowSmoke.xcodeproj`, pick
  your team, Cmd+R. See `examples/window-smoke/TESTING.md`.
- Android phone: enable USB debugging, plug in, the script will pick it up
  automatically (no AVD boot).
