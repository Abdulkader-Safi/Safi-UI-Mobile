//! Safi-UI runtime entry point — `App::new(build_ui).run()`.
//!
//! This module owns every SDL3-facing concern an app needs: window creation,
//! event loop, frame-tick layout, canvas painting, and lifecycle dispatch.
//! Consumer apps construct an [`App`] with a closure that builds the root
//! [`VNode`] and call [`App::run`]. The `SDL_main` C entry shim that the
//! platform shells expect is emitted by the [`crate::app_main`] macro from
//! `safi-ui-macros`.
//!
//! See PRD §6.13 — the `App` handle is the engine-owned runtime, not
//! something consumers reimplement per app.
//!
//! Available behind `feature = "runtime"`. The `gpu` feature is orthogonal
//! and pulls in the future `SDL_GPU` rect pipeline; today this module paints
//! through `SDL_Renderer` because the rect.glsl pipeline isn't online yet
//! (todo 09 device demo / todo 15 base widgets). When it does land, the
//! interior of [`App::run`] swaps backends while the public surface stays
//! the same.

use std::ffi::CStr;

use sdl3::event::{DisplayEvent, Event};
use sdl3::gpu::{Device, ShaderFormat};
use sdl3::pixels::Color;
use sdl3::video::Window;
use taffy::{AvailableSpace, Size};

use crate::assets::{AssetLoader, DpiScale};
use crate::layout::LayoutEngine;
use crate::vnode::{LayoutRect, VNode};

const CLEAR_COLOR: Color = Color::RGB(0x0f, 0x0f, 0x1a);
const DEFAULT_WINDOW_WIDTH: u32 = 480;
const DEFAULT_WINDOW_HEIGHT: u32 = 800;

/// Virtual logical canvas size in dp. The renderer's logical presentation
/// is pinned to this on every platform so layout coordinates are the same
/// regardless of what the OS reports for `window.size()` (iOS returns
/// logical points, Android returns physical pixels, and we'd rather not
/// branch on that). SDL3 stretches/letterboxes from this virtual surface
/// to the actual backbuffer.
///
/// The DPI scale that maps these dp units to physical pixels at the
/// renderer boundary is resolved once at startup via
/// [`DpiScale::from_sdl`] over `SDL_GetDisplayContentScale` (PRD §7.3).
/// This module logs the resolved scale; it will be threaded into
/// `UIContext.dpi_scale` once the build path lands (todo 13).
const LOGICAL_DP_WIDTH: i32 = 480;
const LOGICAL_DP_HEIGHT: i32 = 800;

/// The runtime handle (PRD §6.13). Construct with [`App::new`], drive with
/// [`App::run`]. Future revisions will accept a `StateStore` and
/// `EventBus` here without breaking this surface.
pub struct App {
    build_root: Box<dyn Fn() -> VNode>,
}

impl App {
    /// Build an `App` over a root-tree factory. The factory is invoked once
    /// per layout cycle, so later iterations of the runtime can re-call it
    /// on hot-reload (todo `29`) or state change (todo `23`) without an API
    /// break.
    pub fn new<F>(build_root: F) -> Self
    where
        F: Fn() -> VNode + 'static,
    {
        Self {
            build_root: Box::new(build_root),
        }
    }

    /// Drive the app: open a window, lay out the root tree, paint each
    /// frame, and dispatch SDL3 lifecycle events until `SDL_EVENT_QUIT` or
    /// `AppTerminating`.
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        log("safi-ui::app: booting");

        let sdl = sdl3::init()?;
        log("safi-ui::app: sdl3::init OK");
        let video = sdl.video()?;
        log("safi-ui::app: video subsystem OK");

        let window = video
            .window("safi-ui", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
            .build()
            .map_err(|e| e.to_string())?;
        let (ww, wh) = window.size();
        log(&format!("safi-ui::app: window {ww}x{wh}"));

        // PRD §7.3 — resolve the primary display's content scale once at
        // startup. Pixel 8 ≈ 2.625, iPhone 15 Pro = 3.0, desktop = 1.0.
        // Bad / missing scale collapses to `DpiScale::ONE` rather than
        // failing the boot — apps still want to render on hardware where
        // SDL3 can't determine the value.
        let dpi_scale = resolve_dpi_scale(&video);
        log(&format!("safi-ui::app: dpi_scale = {:.3}", dpi_scale.raw()));

        // PRD §9.3 — pick the platform asset loader. Each branch resolves to
        // a `Box<dyn AssetLoader>` so the rest of the runtime ignores target_os.
        let asset_loader = build_asset_loader();
        log(&format!(
            "safi-ui::app: asset_loader = {}",
            asset_loader_label()
        ));

        // Probe `SDL_GPU` so device verification still sees the driver name
        // (Vulkan / Metal) in logs, then drop it. Real rendering goes
        // through SDL_Renderer until the rect pipeline lands. See the
        // module doc comment.
        match probe_gpu(&window) {
            Ok(driver) => log(&format!(
                "safi-ui::app: SDL_GPU probe OK (driver = {driver}); painting via SDL_Renderer"
            )),
            Err(e) => log(&format!(
                "safi-ui::app: SDL_GPU probe failed ({e}); SDL_Renderer is the only option here"
            )),
        }

        run_canvas_loop(&sdl, window, dpi_scale, asset_loader, &*self.build_root)
    }
}

#[cfg(target_os = "android")]
fn build_asset_loader() -> Box<dyn AssetLoader> {
    match crate::assets::AndroidAssetLoader::from_sdl_activity() {
        Ok(loader) => Box::new(loader),
        Err(e) => {
            log(&format!(
                "safi-ui::app: AndroidAssetLoader init failed ({e}); falling back to filesystem at '.'"
            ));
            Box::new(crate::assets::FilesystemAssetLoader::new("."))
        }
    }
}

#[cfg(target_os = "ios")]
fn build_asset_loader() -> Box<dyn AssetLoader> {
    Box::new(crate::assets::IosAssetLoader::new())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn build_asset_loader() -> Box<dyn AssetLoader> {
    Box::new(crate::assets::FilesystemAssetLoader::new("."))
}

#[cfg(target_os = "android")]
fn asset_loader_label() -> &'static str {
    "AndroidAssetLoader (AAssetManager via SDL Activity)"
}

#[cfg(target_os = "ios")]
fn asset_loader_label() -> &'static str {
    "IosAssetLoader (NSBundle.mainBundle)"
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn asset_loader_label() -> &'static str {
    "FilesystemAssetLoader (host filesystem rooted at '.')"
}

fn resolve_dpi_scale(video: &sdl3::VideoSubsystem) -> DpiScale {
    match video.get_primary_display() {
        Ok(display) => match display.get_content_scale() {
            Ok(raw) => DpiScale::from_sdl(raw),
            Err(e) => {
                log(&format!(
                    "safi-ui::app: SDL_GetDisplayContentScale failed ({e}); defaulting to 1.0"
                ));
                DpiScale::ONE
            }
        },
        Err(e) => {
            log(&format!(
                "safi-ui::app: no primary display ({e}); defaulting dpi_scale to 1.0"
            ));
            DpiScale::ONE
        }
    }
}

/// Public so the `app_main!` macro can route fatal errors through the same
/// log sink as the rest of the runtime without forcing the example crate to
/// import `sdl3::log`.
pub fn log_fatal(msg: &str) {
    log(&format!("safi-ui::app: fatal: {msg}"));
}

fn log(msg: &str) {
    eprintln!("{msg}");
    sdl3::log::log(msg);
}

fn probe_gpu(window: &Window) -> Result<String, sdl3::Error> {
    let dev = Device::new(ShaderFormat::SPIRV | ShaderFormat::MSL, true)?;
    let dev = dev.with_window(window)?;
    let driver: String = unsafe {
        let raw = sdl3::sys::gpu::SDL_GetGPUDeviceDriver(dev.raw());
        if raw.is_null() {
            "<unknown>".to_string()
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    };
    drop(dev);
    Ok(driver)
}

enum EventOutcome {
    Continue,
    Reflow,
    Terminate,
}

/// Dispatch one SDL3 event. Returns whether the loop should exit or
/// re-run layout on the next tick.
fn handle_event(event: &Event) -> EventOutcome {
    match event {
        Event::Quit { .. } => {
            log("safi-ui::app: SDL_EVENT_QUIT");
            EventOutcome::Terminate
        }
        Event::AppTerminating { .. } => {
            log("safi-ui::app: AppTerminating");
            EventOutcome::Terminate
        }
        Event::AppLowMemory { .. } => {
            log("safi-ui::app: AppLowMemory");
            EventOutcome::Continue
        }
        Event::AppWillEnterBackground { .. } => {
            log("safi-ui::app: AppWillEnterBackground");
            EventOutcome::Continue
        }
        Event::AppDidEnterBackground { .. } => {
            log("safi-ui::app: AppDidEnterBackground");
            EventOutcome::Continue
        }
        Event::AppWillEnterForeground { .. } => {
            log("safi-ui::app: AppWillEnterForeground");
            EventOutcome::Continue
        }
        Event::AppDidEnterForeground { .. } => {
            log("safi-ui::app: AppDidEnterForeground");
            EventOutcome::Continue
        }
        // PRD §9.2 — orientation change re-runs layout against the new
        // available size. The render-logical canvas stays pinned at
        // 480×800 dp, so this is mostly a forward-compat hook today;
        // once `App` accepts an `orientation` aware build_root (todo 27,
        // platform bridge) the new size flows through here.
        Event::Display {
            display_event: DisplayEvent::Orientation(_),
            ..
        } => {
            log("safi-ui::app: SDL_EVENT_DISPLAY_ORIENTATION (re-layout)");
            EventOutcome::Reflow
        }
        _ => EventOutcome::Continue,
    }
}

fn prepare_canvas(window: Window) -> sdl3::render::WindowCanvas {
    // Diagnostic: list every render driver SDL3 has compiled in, then —
    // only on iOS — force the software renderer because the simulator's
    // Metal SDL_Renderer doesn't actually present. On Android we want the
    // hardware renderer SDL3 picks for us.
    unsafe {
        let n = sdl3::sys::render::SDL_GetNumRenderDrivers();
        log(&format!("safi-ui::app: {n} render drivers available:"));
        for i in 0..n {
            let p = sdl3::sys::render::SDL_GetRenderDriver(i);
            let name = if p.is_null() {
                "<null>".to_string()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            };
            log(&format!("safi-ui::app:   driver[{i}] = {name}"));
        }
        #[cfg(target_os = "ios")]
        {
            let key = c"SDL_RENDER_DRIVER";
            let val = c"software";
            sdl3::sys::hints::SDL_SetHint(key.as_ptr(), val.as_ptr());
        }
    }

    // Diagnostic only — what does the platform report?
    let (win_w, win_h) = window.size();
    log(&format!(
        "safi-ui::app: window.size() = {win_w}x{win_h} (platform-dependent units)"
    ));

    let canvas = window.into_canvas();

    unsafe {
        let name_ptr = sdl3::sys::render::SDL_GetRendererName(canvas.raw());
        let name = if name_ptr.is_null() {
            "<unknown>".to_string()
        } else {
            CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        };
        log(&format!("safi-ui::app: picked renderer = {name}"));

        let (out_w, out_h) = canvas.output_size().unwrap_or((0, 0));
        log(&format!(
            "safi-ui::app: canvas output_size = {out_w}x{out_h}"
        ));

        // Pin a fixed virtual logical canvas. After this call, every
        // `fill_rect(x, y, w, h)` is interpreted as coordinates inside an
        // `LOGICAL_DP_WIDTH × LOGICAL_DP_HEIGHT` dp surface and SDL3
        // stretches that surface to whatever the actual physical backbuffer
        // is. STRETCH (not LETTERBOX) means the surface fills the whole
        // window on devices of any aspect ratio — fine for a smoke; a real
        // app would compute the dp size from `SDL_GetDisplayContentScale`
        // (PRD §7.3) and use LETTERBOX or OVERSCAN to preserve aspect.
        sdl3::sys::render::SDL_SetRenderLogicalPresentation(
            canvas.raw(),
            LOGICAL_DP_WIDTH,
            LOGICAL_DP_HEIGHT,
            sdl3::sys::render::SDL_LOGICAL_PRESENTATION_STRETCH,
        );
        log(&format!(
            "safi-ui::app: logical presentation = {LOGICAL_DP_WIDTH}x{LOGICAL_DP_HEIGHT} dp (STRETCH)"
        ));
    }

    canvas
}

fn run_canvas_loop(
    sdl: &sdl3::Sdl,
    window: Window,
    dpi_scale: DpiScale,
    asset_loader: Box<dyn AssetLoader>,
    build_root: &dyn Fn() -> VNode,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = dpi_scale; // Consumed by UIContext once todo 13 wires the build path.

    // Smoke-probe the asset loader: log whether the conventional
    // `assets/ui/screens/` directory is reachable. Catches "the app
    // shipped without its asset bundle" regressions before a user
    // sees a blank screen.
    let probe = asset_loader.exists(crate::assets::SCREENS_DIR);
    log(&format!(
        "safi-ui::app: asset probe '{}' exists = {probe}",
        crate::assets::SCREENS_DIR,
    ));
    // The loader is held here for the lifetime of the loop; todo 13
    // (build path) and todo 17 (image pipeline) hand it to UIContext.
    let _asset_loader = asset_loader;
    log("safi-ui::app: entering frame loop (SDL_Renderer)");

    let mut canvas = prepare_canvas(window);

    // Build the tree, lay it out against the *virtual* dp canvas, log the
    // computed layout once for device verification. Future state-driven
    // rebuilds will re-invoke `build_root` and `compute_if_dirty`; the
    // static demo path only needs one pass.
    let mut tree = build_root();
    let mut layout = LayoutEngine::new();
    #[allow(clippy::cast_precision_loss)]
    let available = definite(LOGICAL_DP_WIDTH as f32, LOGICAL_DP_HEIGHT as f32);
    layout.compute(&mut tree, available);
    log_layout("safi-ui::app: layout", &tree);

    let mut event_pump = sdl.event_pump()?;
    let mut frames_drawn: u32 = 0;
    let mut layout_dirty = false;

    'frame: loop {
        for event in event_pump.poll_iter() {
            match handle_event(&event) {
                EventOutcome::Terminate => break 'frame,
                EventOutcome::Reflow => layout_dirty = true,
                EventOutcome::Continue => {}
            }
        }

        if layout_dirty {
            tree = build_root();
            layout.compute(&mut tree, available);
            log_layout("safi-ui::app: layout (post-reflow)", &tree);
            layout_dirty = false;
        }

        canvas.set_draw_color(CLEAR_COLOR);
        canvas.clear();

        for_each_node(&tree, &mut |n| {
            let Some(color) = parse_hex_color(n.props.get("bg").map(String::as_str)) else {
                return;
            };
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let rect = sdl3::rect::Rect::new(
                n.layout.x as i32,
                n.layout.y as i32,
                n.layout.width.max(0.0) as u32,
                n.layout.height.max(0.0) as u32,
            );
            canvas.set_draw_color(color);
            let _ = canvas.fill_rect(rect);
        });

        canvas.present();
        if frames_drawn == 0 {
            log("safi-ui::app: first frame presented");
        }
        frames_drawn += 1;
    }
    log(&format!(
        "safi-ui::app: exiting frame loop — drew {frames_drawn}"
    ));
    Ok(())
}

fn definite(w: f32, h: f32) -> Size<AvailableSpace> {
    Size {
        width: AvailableSpace::Definite(w),
        height: AvailableSpace::Definite(h),
    }
}

fn parse_hex_color(prop: Option<&str>) -> Option<Color> {
    let raw = prop?.trim_start_matches('#');
    if raw.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&raw[0..2], 16).ok()?;
    let g = u8::from_str_radix(&raw[2..4], 16).ok()?;
    let b = u8::from_str_radix(&raw[4..6], 16).ok()?;
    Some(Color::RGB(r, g, b))
}

fn for_each_node(node: &VNode, f: &mut dyn FnMut(&VNode)) {
    f(node);
    for child in &node.children {
        for_each_node(child, f);
    }
}

fn log_layout(prefix: &str, node: &VNode) {
    fn walk(prefix: &str, node: &VNode, depth: usize) {
        let LayoutRect {
            x,
            y,
            width,
            height,
        } = node.layout;
        log(&format!(
            "{prefix}: {indent}<{tag}> x={x:.0} y={y:.0} w={width:.0} h={height:.0}",
            indent = "  ".repeat(depth),
            tag = node.tag,
        ));
        for child in &node.children {
            walk(prefix, child, depth + 1);
        }
    }
    walk(prefix, node, 0);
}
