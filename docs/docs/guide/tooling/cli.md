# `safi` CLI

:::warning Status: Specification (post-v1)
The `safi` binary is a planned **v1.1** deliverable. None of the commands below exist yet. They are listed here to lock in the contract for when CLI work begins.
:::

A standalone `safi` binary distributed via `cargo install safi-cli`. It consumes the same `safi-ui` crate the apps consume, so every command stays in lockstep with the runtime.

## Command surface

Built in this order. The capstone (`safi preview`) ships last because it depends on every other piece of the system being in place.

| Order | Command                           | Purpose                                                                                                                                                        |
| ----- | --------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | `safi new <project>`              | Scaffold a new project (Cargo workspace, Android + iOS host, `assets/ui/` skeleton, sample screen, README)                                                     |
| 2     | `safi generate screen <name>`     | Create `assets/ui/screens/<name>.xml` from a template; lowercase-hyphen naming enforced                                                                        |
| 2     | `safi generate component <Name>`  | Create `assets/ui/components/<Name>.xml` (PascalCase) with a default `<Component>` shell                                                                       |
| 3     | `safi doctor`                     | Verify toolchain: rustc + targets, `cargo-ndk`, `cargo-mobile2`, NDK r25+, Xcode, `glslc`. Reports gaps with fix-it commands                                   |
| 4     | `safi dev --target android\|ios`  | Wraps `cargo ndk` (or `cargo-mobile2`), installs to a connected device/emulator, launches with `dev` feature, streams logs                                     |
| 5     | `safi build --release --target …` | Release builds, strips symbols, reports binary size against the < 800KB worst-case arm64 target                                                                |
| 6     | `safi lint`                       | Validates every `.xml` in `assets/ui/` against the live `ComponentRegistry`: unknown tags, missing required `id`/`key`, malformed prop values, broken bindings |
| 7     | **`safi preview <file.xml>`**     | Desktop hot-reload window rendering a single screen or component. See below.                                                                                   |

## `safi preview`, the capstone

Opens an SDL3 window on the developer's macOS / Linux / Windows machine, draws a device-frame chrome around the rendered UI, runs the full Safi-UI pipeline against the target file, and hot-reloads on save. State values can be mocked from the command line or a JSON file.

```bash
safi preview assets/ui/screens/dashboard.xml --device pixel-8
safi preview assets/ui/components/UserCard.xml \
    --device iphone-15-pro \
    --orientation landscape \
    --state '{"user.name":"Safi","user.role":"Lead"}'
safi preview assets/ui/screens/chat.xml --state-file mocks/chat.json
```

| Attribute            | Detail                                                                                               |
| -------------------- | ---------------------------------------------------------------------------------------------------- |
| Backend              | Same SDL3 + SDL_GPU pipeline as the mobile runtime; uses the desktop SDL3 backend (community-added)  |
| Device frames        | `pixel-8`, `pixel-9-pro`, `iphone-15`, `iphone-15-pro`, `ipad-mini`, `ipad-pro-13`                   |
| Orientation          | `portrait` (default) or `landscape`; triggers Taffy re-layout                                        |
| DPI                  | Matches the selected device's `dpi_scale` exactly                                                    |
| Safe area            | Synthetic safe-area insets per device (notch, home indicator, status bar)                            |
| Mock state           | Inline JSON via `--state` or path via `--state-file`; populated into `StateStore` before first build |
| Hot-reload           | Watches the file (and its component dependencies); reload latency target < 100ms                     |
| Recording (v1.2)     | `--record out.gif` captures the window for marketing / docs / bug reports                            |
| Headless mode (v1.2) | `--snapshot out.png` for CI visual-regression testing                                                |

## Distribution

```bash
cargo install safi-cli            # from crates.io
brew install safi-studio/tap/safi # macOS, post-v1.1
```

The CLI is versioned in lockstep with the `safi-ui` crate. `safi --version` reports both. `safi doctor` warns on version skew between the installed CLI and the `safi-ui` dependency in the current project's `Cargo.toml`.
