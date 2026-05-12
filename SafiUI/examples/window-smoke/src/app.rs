//! Mobile smoke-test entry point. Compiled for `target_os = "android"` and
//! `target_os = "ios"` only — other targets fall through to the empty stub
//! in `lib.rs`.
//!
//! Pipeline:
//!   1. `sdl3::init()` → video subsystem
//!   2. Window sized to the display
//!   3. Try `SDL_GPU` with SPIR-V (Android/Vulkan) + MSL (iOS/Metal). On
//!      hardware that satisfies the backend's requirements this is the
//!      path used in production. The driver is logged so device
//!      verification can confirm Vulkan vs Metal at a glance.
//!   4. **Fallback** — if `SDL_GPU` init fails (notably the iOS Simulator,
//!      whose virtual Metal device doesn't satisfy `MTLGPUFamilyApple3`),
//!      drop down to plain `SDL_Renderer` so the sim still shows the
//!      clear colour for dev-loop / lifecycle validation. Real-device
//!      builds never hit this path.
//!   5. Frame loop clears the swapchain to `#4f8ef7` every frame
//!   6. Lifecycle events route to no-op log handlers

use std::ffi::{c_char, c_int};

use sdl3::Sdl;
use sdl3::event::Event;
use sdl3::gpu::{ColorTargetInfo, Device, LoadOp, ShaderFormat, StoreOp};
use sdl3::pixels::Color;
use sdl3::video::Window;

const CLEAR_COLOR: Color = Color::RGB(0x4f, 0x8e, 0xf7);

fn log(msg: &str) {
    eprintln!("{msg}");
    sdl3::log::log(msg);
}

// `pub(crate)` instead of `pub` — the function is reached from C (SDL3
// calls it after `SDL_RunApp` sets up the platform shell), not from Rust,
// and `#[unsafe(no_mangle)]` keeps the symbol exported regardless of Rust
// visibility. This satisfies the workspace's `unreachable_pub = warn`
// lint (and CI's `-D warnings` promotion).
#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn SDL_main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
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

    let window = video
        .window("safi-ui window smoke", 480, 800)
        .build()
        .map_err(|e| e.to_string())?;
    let (ww, wh) = window.size();
    log(&format!("safi-ui-window-smoke: window {ww}x{wh}"));

    // Try SDL_GPU; fall back to SDL_Renderer on the iOS Simulator (and
    // anywhere else SDL_GPU can't initialise).
    let gpu_init: Result<Device, sdl3::Error> = (|| {
        let dev = Device::new(ShaderFormat::SPIRV | ShaderFormat::MSL, true)?;
        dev.with_window(&window)
    })();

    match gpu_init {
        Ok(gpu) => {
            log_gpu_driver(&gpu);
            run_gpu_loop(sdl, window, gpu)
        }
        Err(e) => {
            log(&format!(
                "safi-ui-window-smoke: SDL_GPU init failed ({e}); falling back to SDL_Renderer"
            ));
            run_canvas_loop(sdl, window)
        }
    }
}

fn log_gpu_driver(gpu: &Device) {
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
}

/// Returns `true` when the event should terminate the loop.
fn handle_event(event: Event) -> bool {
    match event {
        Event::Quit { .. } => {
            log("safi-ui-window-smoke: SDL_EVENT_QUIT");
            true
        }
        Event::AppTerminating { .. } => {
            log("safi-ui-window-smoke: AppTerminating");
            true
        }
        Event::AppLowMemory { .. } => {
            log("safi-ui-window-smoke: AppLowMemory");
            false
        }
        Event::AppWillEnterBackground { .. } => {
            log("safi-ui-window-smoke: AppWillEnterBackground");
            false
        }
        Event::AppDidEnterBackground { .. } => {
            log("safi-ui-window-smoke: AppDidEnterBackground");
            false
        }
        Event::AppWillEnterForeground { .. } => {
            log("safi-ui-window-smoke: AppWillEnterForeground");
            false
        }
        Event::AppDidEnterForeground { .. } => {
            log("safi-ui-window-smoke: AppDidEnterForeground");
            false
        }
        _ => false,
    }
}

fn run_gpu_loop(sdl: Sdl, window: Window, gpu: Device) -> Result<(), Box<dyn std::error::Error>> {
    log("safi-ui-window-smoke: entering frame loop (SDL_GPU)");
    let mut event_pump = sdl.event_pump()?;

    let mut frames_drawn: u32 = 0;
    let mut frames_skipped: u32 = 0;
    let mut last_skip_err: Option<String> = None;

    'frame: loop {
        for event in event_pump.poll_iter() {
            if handle_event(event) {
                break 'frame;
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
                    log("safi-ui-window-smoke: first frame submitted (SDL_GPU)");
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
        "safi-ui-window-smoke: exiting GPU loop — drew {frames_drawn}, skipped {frames_skipped}"
    ));
    Ok(())
}

fn run_canvas_loop(sdl: Sdl, window: Window) -> Result<(), Box<dyn std::error::Error>> {
    log("safi-ui-window-smoke: entering frame loop (SDL_Renderer fallback)");

    // Diagnostic: list every render driver SDL3 has compiled in.
    unsafe {
        let n = sdl3::sys::render::SDL_GetNumRenderDrivers();
        log(&format!("safi-ui-window-smoke: {n} render drivers available:"));
        for i in 0..n {
            let p = sdl3::sys::render::SDL_GetRenderDriver(i);
            let name = if p.is_null() {
                "<null>".to_string()
            } else {
                std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
            };
            log(&format!("safi-ui-window-smoke:   driver[{i}] = {name}"));
        }
        // Force software renderer for the fallback path. iOS Simulator's
        // Metal SDL_Renderer doesn't actually present. Software is a CPU
        // framebuffer → one Metal blit; sidesteps the GPU-feature gate.
        let key = c"SDL_RENDER_DRIVER";
        let val = c"software";
        sdl3::sys::hints::SDL_SetHint(key.as_ptr(), val.as_ptr());
    }

    let mut canvas = window.into_canvas();

    // Log which renderer SDL3 actually picked.
    unsafe {
        let name_ptr = sdl3::sys::render::SDL_GetRendererName(canvas.raw());
        let name = if name_ptr.is_null() {
            "<unknown>".to_string()
        } else {
            std::ffi::CStr::from_ptr(name_ptr)
                .to_string_lossy()
                .into_owned()
        };
        log(&format!("safi-ui-window-smoke: picked renderer = {name}"));
    }

    // Output size sanity check.
    let (out_w, out_h) = canvas.output_size().unwrap_or((0, 0));
    log(&format!(
        "safi-ui-window-smoke: canvas output_size = {out_w}x{out_h}"
    ));

    let mut event_pump = sdl.event_pump()?;
    let mut frames_drawn: u32 = 0;

    'frame: loop {
        for event in event_pump.poll_iter() {
            if handle_event(event) {
                break 'frame;
            }
        }

        canvas.set_draw_color(CLEAR_COLOR);
        canvas.clear();
        // Also explicitly fill the entire viewport. Some SDL3 backends only
        // commit pixels when there's an actual draw call after clear.
        let fill_res = canvas.fill_rect(sdl3::rect::Rect::new(
            0,
            0,
            out_w.max(1) as u32,
            out_h.max(1) as u32,
        ));
        canvas.present();
        if frames_drawn == 0 {
            log(&format!(
                "safi-ui-window-smoke: first frame — fill_rect={:?}",
                fill_res.is_ok()
            ));
            if let Err(e) = fill_res {
                log(&format!("safi-ui-window-smoke: fill_rect error: {e}"));
            }
            log("safi-ui-window-smoke: first frame presented (SDL_Renderer)");
        }
        frames_drawn += 1;
    }
    log(&format!(
        "safi-ui-window-smoke: exiting Canvas loop — drew {frames_drawn}"
    ));
    Ok(())
}
