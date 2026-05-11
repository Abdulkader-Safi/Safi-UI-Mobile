# 01 — GitHub Actions CI

**Phase:** 0 — Foundations
**PRD refs:** §16 (Phase 0), §15.2, §15.3
**Status:** ✅ Completed — May 2026 (CI runs need first-PR verification on GitHub)

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

- All four jobs pass on an empty workspace — ⚠️ host (fmt/clippy/test) mirrored locally on macOS and pass; **Android + iOS cross-builds need first-PR verification on GitHub Actions** (no NDK/iOS toolchain assumed locally)
- Average CI runtime under 8 minutes with warm cache — ⚠️ measurable only after the first warm CI run

## Notes (post-implementation)

- Toolchain in CI is pinned via `dtolnay/rust-toolchain@master` with explicit `toolchain: "1.95.0"` (not via the `rust-toolchain.toml` pickup path), to keep behavior identical across all five jobs and avoid surprises if a runner can't auto-fetch the pinned channel.
- All jobs run with `defaults.run.working-directory: SafiUI` so cargo finds the workspace manifest without per-step `--manifest-path` flags.
- Cargo caches use `Swatinem/rust-cache@v2` with `workspaces: SafiUI -> target` to scope the cache key correctly to the nested workspace.
- Android job uses `nttld/setup-ndk@v1` with NDK r26d (PRD §9.1 requires r25+); `cargo-ndk` is installed at the repo root (not inside `SafiUI/`) so the install step uses `working-directory: .` to override the default.
- Release workflow (`.github/workflows/release.yml`) is dry-run only and the `safi-ui` step is wrapped in `continue-on-error: true` because `cargo publish --dry-run` cannot resolve the path+version `safi-ui-macros` dep until macros is on crates.io. Both flags flip when going live.
- Status badges added to root `README.md`: CI, MIT license, Rust ≥1.95.
