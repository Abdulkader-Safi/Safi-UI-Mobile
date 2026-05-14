# 14 — `ComponentRegistry` + resolution order

**Phase:** 3 — Component Registry
**PRD refs:** §5.4, §6.7, §11.2

**Status:** ✅ Complete — `safi-ui::registry::{ComponentRegistry,
ComponentFactory}` shipped with the `register_component!` declarative
macro and `safi-ui::debug_box::DebugBox` fallback widget. Registry is
mutex-guarded singleton via `ComponentRegistry::global()` (lazy
`OnceLock`); tests use `ComponentRegistry::new()` for isolation.
Resolution falls through to `DebugBox` (red 1dp outline + `<TagName>`
text) for any unknown tag. The middle layer (XML template lookup) is
left to callers that own both the registry and `XmlTemplateLoader`
when todo 27 lands. Duplicate registrations log once to stderr and
last-write-wins per PRD §6.7. 7 host tests cover the contract.

## Goal

Map XML tag names to component factories with the three-step fallback chain.

## Deliverables

- `safi-ui::registry::{ComponentRegistry, ComponentFactory}` per §6.7
  - Factories typed `Box<dyn Fn(&Props) -> Box<dyn Component> + Send + Sync>` (registry only — instances stay main-thread)
- `register_component!(tag, Type)` declarative macro
- Resolution order per §5.4:
  1. Built-in / Rust-registered component
  2. XML template component (`XmlTemplateLoader`, lands later with `27`)
  3. `DebugBox` fallback rendering red outline + unknown tag name
- Duplicate registrations log a warning; last-write-wins
- Global singleton accessor + test-friendly `ComponentRegistry::new()` constructor

## Dependencies

- `13`

## Acceptance

- `DebugBox` renders for any unknown tag and includes the tag name in red text
- Duplicate-registration warning fires exactly once per tag per session
