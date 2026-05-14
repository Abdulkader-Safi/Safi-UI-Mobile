# 15 — Base widgets: `View`, `Text`, `Button` rendering end-to-end

**Phase:** 3 — Component Registry
**PRD refs:** §10.1 (View), §10.2 (Text), §10.3 (Button), §16 (Phase 3)

**Status:** ✅ Complete — `safi-ui::widgets::{View, Text, Button}` ship.
`View` registers under `Screen`, `View`, `Row`, `Column`, `Stack`,
`Spacer` (same paint, different layout direction set by tag). `Text`
registers under `Text`, `Heading`, `Label` (tag-aware default sizes:
h1=32, h2=24, h3=20, h4=18, h5=16, h6=14; Label uppercases at build
time). `Button` registers under `Button` with four variants (primary,
secondary, ghost, danger) and tap dispatch through
`take_pending_event` for the runtime drain (real `EventBus` wiring
lands with todo 24). `register_builtins(&mut ComponentRegistry)`
wires every canonical XML tag in one call. 18 host tests covering
visual variants, opacity modulation, binding substitution, tap
capture, disabled state, registry round-trip, and `DebugBox`
fallback when registry-lookup misses.

## Goal

Three real components wired through XML → parse → layout → registry → build → render. Proves the full pipeline.

## Deliverables

- `View` component: `bg`, `radius`, `border`, `shadow`, `padding`, `margin`, `width`, `height`, `flex`, `visible`, `opacity`
- `Text` component: `size`, `color`, `weight`, `align`, `italic`, `lineHeight`, supports `{{binding}}`
- `Button` component: `label`, `onPress`, `variant` (`primary` / `secondary` / `ghost` / `danger`), `size`, `icon`, `disabled`
- Button registers a `Tap` handler that fires `onPress` through the `EventBus` (stubbed at this stage if `25` hasn't landed; otherwise wire it)
- Example screen `examples/hello-xml/assets/ui/screens/home.xml` rendering all three

## Dependencies

- `09`, `10`, `11`, `12`, `13`, `14`

## Acceptance

- `home.xml` renders pixel-correct on Pixel 8 and iPhone 15
- Button taps fire the registered event
- All three components survive snapshot rendering tests
