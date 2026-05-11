# Getting Started

:::warning Status: Specification (v1.0)
The toolchain and project layout described here are the planned v1.0 developer experience. The library is not yet published to crates.io and the `safi` CLI does not yet exist (it is a planned v1.1 deliverable). Use this page as the contract that the implementation must satisfy.
:::

## Working inside this repository (today)

The library source lives in the `SafiUI/` workspace at the repo root. Today the workspace is a stub — `safi-ui` and `safi-ui-macros` exist as empty crates that `cargo check` cleanly. Modules will fill in across todos `00`…`32`.

```
.
├── SafiUI/                  # Cargo workspace
│   ├── Cargo.toml           # workspace root (members: safi-ui, safi-ui-macros)
│   ├── rust-toolchain.toml  # pins stable Rust (1.95.0)
│   ├── safi-ui/             # main library crate
│   └── safi-ui-macros/      # proc-macro crate (vnode! lands in todo 03)
├── docs/                    # this site (Rspress)
├── todos/                   # 35-step build plan
├── PRD.md                   # source of truth
└── LICENSE                  # MIT
```

```bash
cd SafiUI
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

The `examples/*` workspace glob is intentionally absent until todo 31 lands the example apps — Cargo errors on an empty glob.

## Prerequisites

| Tool                  | Version | Purpose                                                          |
| --------------------- | ------- | ---------------------------------------------------------------- |
| Rust (stable)         | 1.75+   | Compiler                                                         |
| Android NDK           | r25+    | Android native build toolchain                                   |
| Xcode                 | 15+     | iOS native build toolchain                                       |
| `cargo-ndk`           | 3.0+    | Android cross-compilation                                        |
| `cargo-mobile2`       | latest  | iOS Xcode project generation                                     |
| `glslc`               | SDK     | GLSL → SPIR-V / MSL shader compilation (used by `build.rs` only) |
| Android device or AVD | API 24+ | Vulkan-capable target                                            |
| iPhone or simulator   | iOS 16+ | Metal-capable target                                             |

## Add the dependency (planned)

```toml
[dependencies]
safi-ui = "1.0"

# Hot-reload during development
[dependencies.safi-ui-dev]
version = "1.0"
features = ["dev"]
```

## Project layout (planned)

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs
└── assets/
    └── ui/
        ├── screens/
        │   └── home-screen.xml
        └── components/
            └── UserCard.xml
```

| Path                    | Contents                                      |
| ----------------------- | --------------------------------------------- |
| `assets/ui/screens/`    | Screen XML files, lowercase-hyphen filenames  |
| `assets/ui/components/` | User-defined XML components, PascalCase       |
| `assets/images/`        | PNG / JPEG / WebP referenced by `<Image src>` |

## Hello world (planned)

`assets/ui/screens/home-screen.xml`:

```xml
<Screen bg="#0f0f1a" safeArea="true">
  <Column gap="16" padding="24" justify="center" align="center" flex="1">
    <Heading level="1" color="#fff">Hello, Safi-UI!</Heading>
    <Button id="cta" label="Tap me" onPress="demo.tap" />
  </Column>
</Screen>
```

`src/main.rs`:

```rust
use safi_ui::prelude::*;

fn main() -> safi_ui::Result<()> {
    let mut app = App::init(AppConfig::default())?;

    EventBus::global().on("demo.tap", || {
        StateStore::global().set("status", "Tapped!");
    });

    app.load_screen("home-screen")?;
    app.run()
}
```

## Build for Android (planned)

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android
cargo ndk -t arm64-v8a -o ./android/app/src/main/jniLibs build --release
```

## Build for iOS (planned)

```bash
cargo install cargo-mobile2
cargo mobile init
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios --release
```

## Next steps

- [Architecture](/guide/start/architecture) — understand the data flow
- [XML Authoring](/guide/authoring/xml-syntax) — learn the XML model
- [Built-in Components](/api/components/) — what ships out of the box
