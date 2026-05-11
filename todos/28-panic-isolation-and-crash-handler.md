# 28 — Panic isolation + global crash handler

**Phase:** 6 — Platform Polish
**PRD refs:** §14.1, §14.2 (runtime panic case)

## Goal

Dev-only `catch_unwind` around `Component::build` plus a release-mode crash hook that flushes analytics before abort.

## Deliverables

- Dev (`#[cfg(debug_assertions)]`) only: snapshot `UIContext` state before each `build` call (`CommandBuffer.len`, `ClipStack` depth, `FocusSystem` state); restore on panic
- Render `DebugBox` at the panicking widget's **intended layout bounds**, not zero bounds
- Rest of the UI continues rendering normally
- Release builds: do **not** wrap in `catch_unwind`; process aborts on panic per `panic = "abort"` profile
- Global crash handler hook callable from both dev and release — invoked **before abort** in release for analytics flush
- API: `safi_ui::set_crash_handler(Box<dyn Fn(&PanicInfo) + Send + Sync>)`

## Dependencies

- `07`, `15`

## Acceptance

- A panicking custom component in dev shows a red `DebugBox` and the surrounding UI keeps working
- The crash handler fires on a release panic (verified with a test binary that calls `panic!` and exits)
- `ClipStack` depth never leaks across a panicked build call (regression test)
