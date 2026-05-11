# 25 — XML template components (`<Component name="..." props="...">`)

**Phase:** 5 — State + Events
**PRD refs:** §11.1

## Goal

User-defined reusable components authored as `.xml` files, auto-discovered at startup.

## Deliverables

- `safi-ui::templates::XmlTemplateLoader` — scans `assets/ui/components/` recursively at startup
- Registers each `<Component>` into the `ComponentRegistry` so it participates in §5.4 resolution
- Supports `props="name,avatar,role,onPress"` declarations and defaults: `props="name:Anonymous,role:Member"`
- Prop substitution into the template body via `{{propName}}` (reuses `PropUtils::resolve_binding`)
- Naming convention enforced: PascalCase filenames per §12.1
- Hot-reload integration deferred to todo `29`

## Dependencies

- `11`, `13`, `14`, `23`

## Acceptance

- `UserCard.xml` example from §11.1 renders correctly when used as `<UserCard ... />`
- Missing template prop falls through to declared default
- Duplicate `Component name="..."` declarations across files warn and last-write-wins
