# 02 — SDL3 window on Android + iOS

**Phase:** 0 — Foundations
**PRD refs:** §8.1, §9.1, §9.2, §16 (Phase 0)

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

- Cleared window visible on Pixel 8 (or arm64 emulator) and iPhone 15 simulator
- No translation layers in use (verified by SDL log: Vulkan on Android, Metal on iOS — no MoltenVK, no GLES)
- App backgrounds and foregrounds without crashing
