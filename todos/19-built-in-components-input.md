# 19 — Built-in components: Input

**Phase:** 4 — Component Library
**PRD refs:** §10.3, §9.4 (keyboard layout)

## Goal

The text-input and form-control component set.

## Deliverables

- `Input` — `placeholder`, `value`, `onChange`, `type` (`text` / `number` / `email` / `password`), `maxLength`; `id` required; cursor position preserved across hot-reload
- `TextArea` — multiline; `id` required; `placeholder`, `rows`, `onChange`
- `Checkbox` — `label`, `checked`, `onChange`
- `Switch` — `value`, `onChange`, `label`
- `Select` — dropdown with `options`, `value`, `onChange`, `placeholder`
- `Slider` — `min`, `max`, `value`, `step`, `onChange`
- Soft keyboard integration: `PlatformBridge::keyboard_height` drives Taffy re-layout to push focused inputs into view (§9.4)
- IME text input via `SDL_EVENT_TEXT_INPUT`

## Dependencies

- `15`, `18`

## Acceptance

- Focused input scrolls into view when soft keyboard appears on both platforms
- Cursor position and selection survive a `vnode!` re-render with stable `id`
- Each component has touch-target ≥ 44dp (iOS HIG / Android Material)
