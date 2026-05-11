# 00 — Repo and Cargo workspace setup

**Phase:** 0 — Foundations
**PRD refs:** §15.1, §19

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

- `cargo check --workspace` succeeds with no source files (empty `lib.rs` stubs)
- `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` are clean
- Workspace builds on macOS, Linux, and Windows hosts (no platform-specific dependencies yet)
