# Fonts and Text

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned text pipeline.
:::

## Font rasterization

Safi-UI uses [`fontdue`](https://github.com/mooman219/fontdue) for font rasterization — a pure Rust font engine with no C dependencies and no FreeType requirement.

- Fonts loaded from `.ttf` or `.otf` files bundled in app assets
- Glyphs rasterized at startup into a GPU texture atlas
- The atlas is rebuilt when the DPI scale changes (orientation change, external display attach)

## Bundled fonts

| Font             | Coverage             |
| ---------------- | -------------------- |
| Inter            | Latin scripts        |
| Noto Sans Arabic | RTL / Arabic support |

Apps can add their own fonts via `FontManager::register(...)` (planned).

## Text shaping

For complex scripts and RTL text, Safi-UI uses [`rustybuzz`](https://github.com/RazrFalcon/rustybuzz) (a pure Rust port of HarfBuzz) for shaping before rasterization. This enables correct Arabic, Hindi, Thai, and other complex script rendering.

## Typography components

See [`<Text>`, `<Heading>`, `<Label>`, `<Code>`](/api/components/typography).
