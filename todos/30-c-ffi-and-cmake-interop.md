# 30 — C FFI surface (`cbindgen`) + CMake interop

**Phase:** 6 — Platform Polish
**PRD refs:** §15.4, §15.5

## Goal

Small, stable C FFI for game engines and CMake users. Custom component registration stays Rust-only in v1.

## Deliverables

- `safi-ui-ffi/` crate exposing the six symbols from §15.4:
  - `safi_ui_init(config)`
  - `safi_ui_load_screen(name)`
  - `safi_ui_frame()`
  - `safi_ui_set_state(key, value)`
  - `safi_ui_on_event(event)`
  - `safi_ui_shutdown()`
- `build.rs` invoking `cbindgen` to emit `safi_ui.h`
- No C++ wrapper; raw C header only
- CMake example under `examples/cmake-interop/` showing the §15.5 snippet driving a real screen
- Header pinned by `cbindgen.toml` config; CI fails on undocumented header changes

## Dependencies

- `15`, `23`, `24`

## Acceptance

- C example loads a screen, sets state, dispatches an event, and shuts down cleanly
- `safi_ui.h` lives in version control and matches the generated output (CI check)
