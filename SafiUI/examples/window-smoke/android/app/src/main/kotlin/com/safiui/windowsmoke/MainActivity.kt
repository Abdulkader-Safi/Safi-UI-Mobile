package com.safiui.windowsmoke

import org.libsdl.app.SDLActivity

/**
 * Smoke-test activity. Inherits all of SDL's Android plumbing (touch input,
 * lifecycle bridge, Vulkan surface creation) from [SDLActivity] and just
 * declares which native library to load.
 *
 * The native entry point lives in the Rust crate at
 * `examples/window-smoke/src/app.rs` — the `#[sdl3_main::main]` attribute
 * exposes `SDL_main`, which `SDLActivity` invokes after the JVM-side glue
 * has built the Vulkan-capable surface.
 */
class MainActivity : SDLActivity() {
    override fun getLibraries(): Array<String> = arrayOf(
        // Order matters:
        //   1. c++_shared — provides __gxx_personality_v0 / __cxa_* that SDL3's
        //      C++ helpers reference.
        //   2. SDL3 — its JNI_OnLoad RegisterNatives()-registers SDLActivity's
        //      native methods so SDLActivity.nativeGetVersion() resolves.
        //   3. safi_ui_window_smoke — our Rust crate (depends on libSDL3.so
        //      via DT_NEEDED).
        "c++_shared",
        "SDL3",
        "safi_ui_window_smoke",
    )
}
