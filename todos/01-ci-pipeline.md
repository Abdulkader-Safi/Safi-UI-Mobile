# 01 — GitHub Actions CI

**Phase:** 0 — Foundations
**PRD refs:** §16 (Phase 0), §15.2, §15.3

## Goal

CI that builds the workspace on macOS + Linux and cross-compiles for Android + iOS targets on every PR.

## Deliverables

- `.github/workflows/ci.yml` jobs:
  - `fmt` — `cargo fmt --check`
  - `clippy` — `cargo clippy --workspace --all-targets -- -D warnings`
  - `test-host` — `cargo test --workspace` on `ubuntu-latest` and `macos-latest`
  - `build-android` — installs `cargo-ndk`, adds `aarch64-linux-android`, runs `cargo ndk -t arm64-v8a build`
  - `build-ios` — adds `aarch64-apple-ios`, runs `cargo build --target aarch64-apple-ios`
- `.github/workflows/release.yml` skeleton (cargo publish gated behind tag)
- Caching of `~/.cargo/registry`, `~/.cargo/git`, and `target/` keyed by `Cargo.lock`
- Status badges in root `README.md`

## Dependencies

- `00-repo-and-cargo-setup`

## Acceptance

- All four jobs pass on an empty workspace
- Average CI runtime under 8 minutes with warm cache
