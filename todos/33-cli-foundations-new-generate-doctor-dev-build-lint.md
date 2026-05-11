# 33 — `safi` CLI foundations: `new`, `generate`, `doctor`, `dev`, `build`, `lint`

**Phase:** Post-v1 — CLI (v1.1)
**PRD refs:** §20.1

## Goal

The six non-capstone commands of the `safi` binary. Build them in the §20.1 order, but they can ship together as v1.1.0 alpha. The capstone (`preview`) is the next todo.

## Deliverables

- New `safi-cli/` crate, distributed as `cargo install safi-cli`
- Commands in build order:
  1. `safi new <project>` — scaffold Cargo workspace + Android + iOS host + `assets/ui/` skeleton + sample screen + README
  2. `safi generate screen <name>` — emit `assets/ui/screens/<name>.xml` (lowercase-hyphen enforced)
  3. `safi generate component <Name>` — emit `assets/ui/components/<Name>.xml` (PascalCase enforced)
  4. `safi doctor` — verify rustc + targets, `cargo-ndk`, `cargo-mobile2`, NDK r25+, Xcode, `glslc`; print fix-it commands
  5. `safi dev --target android|ios` — wraps `cargo ndk` / `cargo-mobile2`, installs to connected device, launches with `dev` feature, streams logs
  6. `safi build --release --target ...` — release builds, strips symbols, reports binary size against §17.1 target
  7. `safi lint` — validates every `.xml` in `assets/ui/` against the live `ComponentRegistry`; checks unknown tags, missing required `id`/`key`, malformed props, broken `{{binding}}` references against optional `state.schema.json`
- CLI links against the same `safi-ui` crate so the lint registry stays in lockstep with the runtime
- `safi --version` reports both CLI and `safi-ui` versions; warns on version skew (§20.4)

## Dependencies

- `32` (`safi-ui` v1.0 must be published first)

## Acceptance

- `safi new myapp && cd myapp && safi dev --target android` runs end-to-end on a clean machine after `safi doctor` reports green
- `safi lint` catches each of: unknown tag, missing `id` on `Input`, missing `key` on `FlatList` item, malformed `{{binding}}`
