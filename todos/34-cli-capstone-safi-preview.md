# 34 — `safi preview` capstone (desktop hot-reload window)

**Phase:** Post-v1 — CLI (v1.1, capstone)
**PRD refs:** §20.1 (order 7), §20.2, §20.3

## Goal

The capstone command. Opens an SDL3 window on the dev machine, renders a single screen / component through the full Safi-UI pipeline inside a device-frame chrome, and hot-reloads on save. This is the command that collapses the dev loop from "edit → cargo ndk build → install → tap to screen" to "edit → save → see it".

## Deliverables

- `safi preview <file.xml>` subcommand in `safi-cli`
- Desktop SDL3 backend (community-added per §20.2) for macOS / Linux / Windows hosts
- Device-frame chrome:
  - Devices: `pixel-8`, `pixel-9-pro`, `iphone-15`, `iphone-15-pro`, `ipad-mini`, `ipad-pro-13`
  - Each device specifies dimensions, `dpi_scale`, synthetic safe-area insets (notch, home indicator, status bar)
- Orientation: `--orientation portrait|landscape` triggers Taffy re-layout
- Mock state:
  - `--state '{"user.name":"Safi"}'` inline JSON
  - `--state-file mocks/chat.json` populates `StateStore` before the first build
- Hot-reload reuses the `safi-ui` `HotReloadWatcher` (todo `29`); reload latency target < 100ms
- Watches the target file **and its component dependencies** (any `<UserCard>` etc. referenced in the tree)
- v1.2 stretch (parked, but reserve the flags):
  - `--record out.gif` window capture
  - `--snapshot out.png` headless render for CI visual regression

## Dependencies

- `29` (hot-reload), `33` (CLI shell)

## Acceptance

- `safi preview assets/ui/screens/dashboard.xml --device pixel-8` renders the Appendix example pixel-correct, inside a Pixel 8 frame
- `--state-file` mocks resolve into `{{bindings}}`
- Saving the file (or any of its component dependencies) reflects in the preview window in < 100ms
- Switching `--orientation landscape` re-lays out correctly
- Documented as the foundation for the v2 visual editor (§20.3)
