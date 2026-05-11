# 15 — Base widgets: `View`, `Text`, `Button` rendering end-to-end

**Phase:** 3 — Component Registry
**PRD refs:** §10.1 (View), §10.2 (Text), §10.3 (Button), §16 (Phase 3)

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
