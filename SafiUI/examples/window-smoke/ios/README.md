# iOS host — `WindowSmoke.xcodeproj`

A minimal iOS app that links the `safi-ui-window-smoke` Rust staticlib and
hands control over to `SDL_RunApp`. SDL3 is statically linked into the `.a`
by the `sdl3-rs` `build-from-source-static` feature, so there is no separate
framework to bundle.

The bridge is **Swift** — a single `Sources/main.swift` with
`@_cdecl("main")` calling `SDL_RunApp`. A bridging header
(`Sources/SafiUI-Bridging-Header.h`) defines `SDL_MAIN_HANDLED` before
including `<SDL3/SDL_main.h>` so SDL's macro doesn't fight Swift over the
`main` symbol.

## Build + run

Prereqs:

- Xcode 16+ with the iOS 16.0+ SDK
- `rustup target add aarch64-apple-ios`

One-time setup:

```bash
./build.sh         # debug build for device
./build.sh release # release archive
```

`build.sh` populates `Sources/build/libsafi_ui_window_smoke.a` and
`build/sdl3-include/SDL3/` (vendored SDL3 headers) before invoking
`xcodebuild`. After the first successful run you can open
`WindowSmoke.xcodeproj` directly in Xcode and press Cmd+R — the static
library and headers stay on disk for incremental rebuilds.

## What you should see

Solid `#4f8ef7` window in the simulator / on device. Xcode console:

```
safi-ui-window-smoke: SDL_GPU driver = Some("metal")
```

If you see anything other than `metal` (e.g. `direct3d12`, `vulkan`,
`software`), the device or simulator is misconfigured.

## Hand-off note

Acceptance for todo 02 (PRD §16, Phase 0) requires:

- Cleared `#4f8ef7` window on iPhone 15 simulator — needs your device run.
- `SDL_GPU` driver = `metal` (no MoltenVK, no GLES) — needs your device run.
- App backgrounds and foregrounds without crashing — Home button + return.

The Rust + Xcode scaffolding is in place; the verification is yours.
