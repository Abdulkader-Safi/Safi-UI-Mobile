//! Window smoke test — opens an SDL3 window with an `SDL_GPU` device, clears
//! the framebuffer to a recognisable colour every frame, and routes the four
//! lifecycle events to no-op handlers.
//!
//! Targets: Android (Vulkan via `SDL_GPU`) and iOS (Metal via `SDL_GPU`).
//!
//! The default build (`cargo check --workspace`, host CI) compiles as an
//! empty stub. The real entry point only enters when both
//! `--features device-build` is set **and** the target is `android` /
//! `ios`. Any other target/feature combination falls through to the stub
//! so `CMake` never runs during host checks.

#![cfg_attr(
    not(all(
        feature = "device-build",
        any(target_os = "android", target_os = "ios")
    )),
    allow(dead_code)
)]

#[cfg(all(
    feature = "device-build",
    any(target_os = "android", target_os = "ios")
))]
mod app;

// Stub symbol so the cdylib / staticlib always has something to export, even
// when sdl3 isn't compiled in. Lets every host CI job link cleanly.
#[cfg(not(all(
    feature = "device-build",
    any(target_os = "android", target_os = "ios")
)))]
#[unsafe(no_mangle)]
pub extern "C" fn safi_ui_window_smoke_stub() {}
