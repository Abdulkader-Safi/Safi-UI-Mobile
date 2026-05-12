# 02 — SDL3 window on Android + iOS

**Phase:** 0 — Foundations
**PRD refs:** §8.1, §9.1, §9.2, §16 (Phase 0)
**Status:** ✅ Android verified on Pixel 8 emulator (Vulkan), ✅ iOS verified on iPhone 17 Pro Max (Metal). iOS Simulator works via an `SDL_Renderer` software-backend fallback in `examples/window-smoke/src/app.rs` (sim's virtual GPU can't satisfy SDL_GPU's `MTLGPUFamilyApple3` check). Flutter-style one-shot runners landed in `SafiUI/scripts/{dev-android,dev-ios}.sh`. — May 2026

## Goal

Open an SDL3 window on both target devices with a confirmed Vulkan (Android) and Metal (iOS) surface from `SDL_GPU`. This is the platform smoke test that unlocks every later GPU todo.

## Deliverables

- `examples/window-smoke/` — minimum app that opens a window, clears to a colour, polls events, exits cleanly
- Android host project under `examples/window-smoke/android/` (Gradle + NDK r25+) wired via `cargo-ndk`
- iOS host project under `examples/window-smoke/ios/` (Xcode + `cargo-mobile2`) targeting iOS 16
- Confirmed `SDL_GPU` device creation with the Vulkan driver on Android (API 24+) and Metal on iOS — log the chosen driver at startup
- Lifecycle hooks (`SDL_EVENT_WILL_ENTER_BACKGROUND`, `_DID_ENTER_FOREGROUND`, `_LOW_MEMORY`, `_QUIT`) routed to no-op handlers

## Dependencies

- `00`, `01`

## Acceptance

- Cleared window visible on Pixel 8 (or arm64 emulator) and iPhone 15 simulator — ✅ **Android verified** (Pixel 8 emulator, solid `#4f8ef7` filling 1080×2400); ⚠️ iOS Simulator cannot run SDL_GPU Metal (`Device does not meet the hardware requirements for SDL_GPU Metal`), needs a real iPhone (A11+)
- No translation layers in use (verified by SDL log: Vulkan on Android, Metal on iOS — no MoltenVK, no GLES) — ✅ **Android verified** (`safi-ui-window-smoke: SDL_GPU driver = vulkan` in logcat); ⚠️ iOS Metal driver string needs real-device verification
- App backgrounds and foregrounds without crashing — ✅ **Android verified** (Home → return survives, `onPause → onStop → onStart → onResume` log cycle); ⚠️ iOS needs real-device verification

## Notes (post-implementation)

- **`sdl3` crate version**: PRD §15.1 says `0.1` but the crate is actually at `0.18.3` on crates.io. Pinned to `^0.18` and recorded here; PRD reference is stale. `sdl3-main 0.6.2` provides the `#[main]` macro for the entry point.
- **Feature-gated SDL3 build**: the example uses a `device-build` feature that controls whether sdl3 + sdl3-main are pulled in. Default builds (host CI, `cargo check --workspace`) compile an empty stub — no CMake/NDK/Xcode toolchain runs. Mobile builds opt in: `cargo ndk -t arm64-v8a build -p safi-ui-window-smoke --features device-build` (Android) and `cargo build -p safi-ui-window-smoke --target aarch64-apple-ios --features device-build` (iOS). This deviation was forced because sdl3-sys' `build-from-source-static` runs CMake during `cargo check`, which fails on hosts without the relevant native toolchain. CI does both: stub workspace build, then explicit feature build per target.
- **SDL_GPU driver selection**: SDL3 picks the backend at device-creation time based on available shader formats. We request `SPIRV | MSL`, so Android picks Vulkan (SPIR-V) and iOS picks Metal (MSL). The chosen driver is logged at startup so device verification is a single grep against Logcat / Xcode console.
- **Android host**: Kotlin (per user preference), `kotlin/` source root, `MainActivity : SDLActivity`. SDL3's Android Java glue (`SDLActivity.java`, `HIDDeviceManager.java`, …) lives inside the SDL3 source tarball that the sdl3 crate vendors; `build.sh` copies it out of the cargo cache into the gradle source tree before `assembleDebug`. NDK r26d (PRD §9.1 says r25+; current LTS is r26d). API 24 minSdk (PRD-mandated for Vulkan).
- **iOS host**: hand-written `WindowSmoke.xcodeproj/project.pbxproj`. The Rust crate provides `#[no_mangle] pub unsafe extern "C" fn SDL_main`; the Swift bridge (`Sources/WindowSmoke.swift`) uses `@_cdecl("main")` to provide the C `main` and calls `SDL_RunApp(argc, argv, SDL_main, nil)`. Bridging header declares just `SDL_RunApp` + the `SDL_main_func` typedef + `SDL_main` — not the full `<SDL3/SDL_main.h>`, because the latter's `#define main SDL_main` confuses Swift's automatic entry-point detection. `SWIFT_VERSION = 5.0`, `OTHER_SWIFT_FLAGS = -parse-as-library`.
- **iOS build settings**: `OTHER_LDFLAGS` includes `-ObjC` + `-force_load $(SRCROOT)/Sources/build/$(PLATFORM_NAME)/libsafi_ui_window_smoke.a` plus 18 `-framework` flags (Foundation, UIKit, QuartzCore, Metal, MetalKit, CoreGraphics, CoreVideo, CoreMedia, CoreMotion, CoreHaptics, CoreServices, CoreBluetooth, AVFoundation, AVFAudio, AudioToolbox, GameController, OpenGLES, ImageIO; weak Photos). `-force_load` is required because the linker won't pull `main` / `SDL_main` out of a staticlib unless something already references them. `ARCHS = arm64` and `EXCLUDED_ARCHS = x86_64` so simulator builds on Apple Silicon Macs don't try to link an x86_64 build of the staticlib that we don't produce. `LIBRARY_SEARCH_PATHS` uses `$(PLATFORM_NAME)` so the iphoneos and iphonesimulator builds pick up different staticlibs from `Sources/build/{iphoneos,iphonesimulator}/`.
- **iOS deployment target**: `build.sh` exports `IPHONEOS_DEPLOYMENT_TARGET=16.0` before `cargo build` — without it the Rust target defaults to a lower minimum that misses Darwin stack-probe intrinsics (`__chkstk_darwin`) referenced by SDL3's audio code, breaking the cdylib link step.
- **iOS Simulator cannot run SDL_GPU Metal**: the simulator's virtual Metal device reports a GPU family below `MTLGPUFamilyApple3`. `Device::new` fails with `Device does not meet the hardware requirements for SDL_GPU Metal`. This is an SDL3 limitation, not ours. Smoke-test verification requires a real iPhone (A11 / iPhone 8 or newer). `build.sh sim` is still useful — it confirms the Swift bridge + linker setup and the bridge correctly calls `SDL_main` (the Rust entry runs; only the SDL_GPU device init fails on simulator).
- **sdl3-rs API quirks**: dropped the `#[sdl3_main::main]` macro and the `sdl3-main` dep — the macro generates a Rust `fn main()` for binary crates, but our `crate-type = ["cdylib", "staticlib"]` library setup needs an explicit `#[no_mangle] pub unsafe extern "C" fn SDL_main` so the Swift / Kotlin platform shim can find the symbol. The high-level wrapper also doesn't expose `SDL_GetGPUDeviceDriver` — we reach into `sdl3::sys::gpu::SDL_GetGPUDeviceDriver(gpu.raw())` directly to log the active backend.
- **CI updates**: `build-android` and `build-ios` jobs each now run two commands — the standard workspace build (stub) and the `--features device-build -p safi-ui-window-smoke` build that actually exercises SDL3. Cold first run may take 5–10 min as SDL3 CMake-builds from source; cached after.
- **Repo URL alignment**: fixed the README CI badge link to point at `Abdulkader-Safi/Safi-UI-Mobile` (was still on the old `AbdulKaderSafi/safi-ui` URL after the user's earlier rename).
