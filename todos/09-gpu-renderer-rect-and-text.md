# 09 — `GpuRenderer` (rect + text, Phase 1 acceptance demo)

**Phase:** 1 — Core Engine
**PRD refs:** §8.1, §8.2, §8.3, §16 (Phase 1 acceptance demo)

## Goal

Consume `CommandBuffer` and submit batched draw calls through `SDL_GPU`. Two shaders only (`rect`, `text`) — enough for the Phase 1 acceptance demo.

## Deliverables

- `safi-ui::gpu::GpuRenderer` driving SDL_GPU on both backends
- Shader build pipeline in `build.rs` using `glslc`:
  - `rect.glsl` → SPIR-V (Vulkan) + MSL (Metal)
  - `text.glsl` → SPIR-V + MSL
- Batching: consecutive compatible commands fold into single draws
- Per-subtree command-range tracking so partial dirty frames don't rebuild the whole buffer
- Resource lifecycle hooks: `release_resources` on background, `recreate_resources` on foreground
- **Acceptance demo (Phase 1):** programmatic (`vnode!`) button that flips colour on tap — runs on Pixel 8 and iPhone 15

## Dependencies

- `02`, `05`, `06`, `07`, `08`

## Acceptance

- Demo button flips colour on both physical devices
- 50–80 widgets resolve to ≤ 15 GPU draw calls
- Backgrounding the app releases GPU resources cleanly; foregrounding recreates them
