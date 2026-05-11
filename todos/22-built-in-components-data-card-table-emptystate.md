# 22 — Built-in components: Data (excluding FlatList)

**Phase:** 4 — Component Library
**PRD refs:** §10.6

## Goal

The non-virtualised data components. `FlatList` is its own todo (`26`) because virtualization is non-trivial.

## Deliverables

- `Card` — `elevation`, `bg`, `radius`, `border`, `padding`; elevated container (shadow via `Command::Shadow`)
- `Table` — `columns`, `data`, `striped`, `border`; simple grid table
- `EmptyState` — `icon`, `title`, `message`, `action`; placeholder for empty lists / errors

## Dependencies

- `15`, `18`, `20`

## Acceptance

- `Card` shadow renders correctly on both platforms (shadow shader matches design ref)
- `Table` handles a 50-row × 6-column dataset without dropping frames
