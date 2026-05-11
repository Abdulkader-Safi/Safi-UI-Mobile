//! Mobile smoke-test entry point. Compiled for `target_os = "android"` and
//! `target_os = "ios"` only — other targets fall through to the empty stub
//! in `lib.rs`.
//!
//! Pipeline:
//!   1. `sdl3::init()` → video subsystem
//!   2. Window sized to the display
//!   3. `SDL_GPU` device with SPIR-V (Android/Vulkan) + MSL (iOS/Metal)
//!      shader formats — SDL picks the active backend at device-creation
//!      time, which is logged so device verification can confirm Vulkan
//!      vs Metal at a glance.
//!   4. Frame loop clears the swapchain to `#4f8ef7` every frame
//!   5. Lifecycle events (`WillEnterBackground`, `DidEnterBackground`,
//!      `WillEnterForeground`, `DidEnterForeground`, `LowMemory`) route
//!      to no-op log handlers

use std::ffi::{c_char, c_int};

use sdl3::event::Event;
use sdl3::gpu::{ColorTargetInfo, Device, LoadOp, ShaderFormat, StoreOp};
use sdl3::pixels::Color;

const CLEAR_COLOR: Color = Color::RGB(0x4f, 0x8e, 0xf7);

/// Log line that always reaches the Xcode / Logcat console — uses both
/// stderr (Xcode picks it up via NSLog on iOS) and SDL's own logger so we
/// get output even before SDL is initialised.
fn log(msg: &str) {
    eprintln!("{msg}");
    sdl3::log::log(msg);
}

/// The Rust-side entry point. Called by SDL via `SDL_RunApp` after the
/// platform scaffolding (Android `SDLActivity`, iOS `UIApplication`) has
/// brought up the window surface.
///
/// On Android: `SDLActivity.onCreate` calls into `SDL_main`.
/// On iOS: our Swift `@_cdecl("main")` calls `SDL_RunApp(argc, argv,
/// SDL_main, NULL)`, which routes here once UIKit is ready.
///
/// # Safety
///
/// Called by SDL on the main thread with valid argc/argv. Returns 0 on
/// graceful exit, non-zero on fatal error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SDL_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    // First line — confirms SDL_main reached us before SDL itself is up.
    eprintln!("safi-ui-window-smoke: SDL_main entered");
    match run() {
        Ok(()) => 0,
        Err(e) => {
            log(&format!("safi-ui-window-smoke: fatal: {e}"));
            1
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    log("safi-ui-window-smoke: booting");

    let sdl = sdl3::init()?;
    log("safi-ui-window-smoke: sdl3::init OK");
    let video = sdl.video()?;
    log("safi-ui-window-smoke: video subsystem OK");

    // Use a small fixed size — SDL3 fullscreens automatically on iOS /
    // Android, so the requested dimensions are advisory. Avoids the
    // earlier failure mode where `display.get_mode()` returned physical
    // pixels that confused the swapchain.
    let window = video
        .window("safi-ui window smoke", 480, 800)
        .build()
        .map_err(|e| e.to_string())?;
    let (ww, wh) = window.size();
    log(&format!("safi-ui-window-smoke: window {ww}x{wh}"));

    // SPIRV on Android (Vulkan); MSL on iOS (Metal). SDL_GPU picks one.
    let gpu = Device::new(ShaderFormat::SPIRV | ShaderFormat::MSL, true)?.with_window(&window)?;
    // The high-level wrapper doesn't expose the driver name; reach into
    // sdl3-sys for SDL_GetGPUDeviceDriver. Safe: gpu.raw() returns the
    // valid device handle and the returned const char* is owned by SDL.
    let driver: &str = unsafe {
        let raw = sdl3::sys::gpu::SDL_GetGPUDeviceDriver(gpu.raw());
        if raw.is_null() {
            "<unknown>"
        } else {
            std::ffi::CStr::from_ptr(raw)
                .to_str()
                .unwrap_or("<non-utf8>")
        }
    };
    log(&format!("safi-ui-window-smoke: SDL_GPU driver = {driver}"));

    let mut event_pump = sdl.event_pump()?;
    log("safi-ui-window-smoke: entering frame loop");

    let mut frames_drawn: u32 = 0;
    let mut frames_skipped: u32 = 0;
    let mut last_skip_err: Option<String> = None;

    'frame: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    log("safi-ui-window-smoke: SDL_EVENT_QUIT");
                    break 'frame;
                }
                Event::AppTerminating { .. } => {
                    log("safi-ui-window-smoke: AppTerminating");
                    break 'frame;
                }
                Event::AppLowMemory { .. } => log("safi-ui-window-smoke: AppLowMemory"),
                Event::AppWillEnterBackground { .. } => {
                    log("safi-ui-window-smoke: AppWillEnterBackground");
                }
                Event::AppDidEnterBackground { .. } => {
                    log("safi-ui-window-smoke: AppDidEnterBackground");
                }
                Event::AppWillEnterForeground { .. } => {
                    log("safi-ui-window-smoke: AppWillEnterForeground");
                }
                Event::AppDidEnterForeground { .. } => {
                    log("safi-ui-window-smoke: AppDidEnterForeground");
                }
                _ => {}
            }
        }

        let mut cmd = match gpu.acquire_command_buffer() {
            Ok(c) => c,
            Err(e) => {
                log(&format!(
                    "safi-ui-window-smoke: acquire_command_buffer ERR: {e}"
                ));
                continue;
            }
        };
        match cmd.wait_and_acquire_swapchain_texture(&window) {
            Ok(swapchain) => {
                let targets = [ColorTargetInfo::default()
                    .with_texture(&swapchain)
                    .with_load_op(LoadOp::CLEAR)
                    .with_store_op(StoreOp::STORE)
                    .with_clear_color(CLEAR_COLOR)];
                let pass = gpu.begin_render_pass(&cmd, &targets, None)?;
                gpu.end_render_pass(pass);
                cmd.submit()?;
                frames_drawn += 1;
                if frames_drawn == 1 {
                    log("safi-ui-window-smoke: first frame submitted");
                }
            }
            Err(e) => {
                cmd.cancel();
                frames_skipped += 1;
                let msg = e.to_string();
                if last_skip_err.as_ref() != Some(&msg) {
                    log(&format!(
                        "safi-ui-window-smoke: swapchain unavailable (#{frames_skipped}): {msg}"
                    ));
                    last_skip_err = Some(msg);
                }
            }
        }
    }
    log(&format!(
        "safi-ui-window-smoke: exiting — drew {frames_drawn}, skipped {frames_skipped}"
    ));

    log("safi-ui-window-smoke: clean exit");
    Ok(())
}
