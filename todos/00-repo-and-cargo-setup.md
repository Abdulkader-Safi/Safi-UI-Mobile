# 00 — Repo and Cargo workspace setup

**Phase:** 0 — Foundations
**PRD refs:** §15.1, §19
**Status:** ✅ Completed — May 2026

## Goal

Stand up the Cargo workspace, lockfile, license, and base directory layout so every subsequent module has a home.

## Deliverables

- `Cargo.toml` workspace root with members: `safi-ui`, `safi-ui-macros`, `examples/*`
- `safi-ui/Cargo.toml` with the dependency set from §15.1 (`sdl3`, `taffy`, `fontdue`, `rustybuzz`, `roxmltree`, `image`, `glam`, `serde`, `hashbrown`)
- `safi-ui/Cargo.toml` features: `default = []`, `dev = ["hot-reload"]`
- `safi-ui-macros/` proc-macro crate skeleton (used in todo `03`)
- `MIT` LICENSE file
- `rust-toolchain.toml` pinning the 2021 edition toolchain
- `.gitignore` excluding `target/`, `*.xcodeproj/`, `local.properties`, NDK build outputs
- Top-level `README.md` linking to `PRD.md` and `docs/`

## Acceptance

- `cargo check --workspace` succeeds with no source files (empty `lib.rs` stubs) — ✅
- `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` are clean — ✅
- Workspace builds on macOS, Linux, and Windows hosts (no platform-specific dependencies yet) — ⚠️ macOS verified; Linux/Windows pending user device verification (pure-Rust deps so expected portable)

## Notes (post-implementation)

- Code lives under `SafiUI/` at the repo root (not the repo root itself), keeping PRD/docs/todos at root.
- `examples/*` workspace glob omitted until todo 31 — Cargo errors on an empty glob.
- `sdl3` dep deferred to todo 02 (SDL3 window) — pulling its native build script would break `cargo check` on hosts without SDL3 headers and contradict the "builds on all three hosts" acceptance bullet.
- Rust toolchain pinned to 1.95.0 (matches the user's installed stable as of May 2026).
