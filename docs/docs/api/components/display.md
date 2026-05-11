# Display Components

:::warning Status: Specification (v1.0)
None of these are implemented yet.
:::

| Component   | Tag             | Key Props                                 | Notes                                                                 |
| ----------- | --------------- | ----------------------------------------- | --------------------------------------------------------------------- |
| Image       | `<Image>`       | `src`, `width`, `height`, `radius`, `fit` | `fit`: `cover` \| `contain` \| `fill`; resolved from `assets/images/` |
| Avatar      | `<Avatar>`      | `src`, `size`, `fallback`, `radius`       | `fallback`: initials string                                           |
| Icon        | `<Icon>`        | `name`, `size`, `color`                   | Icon atlas; name maps to UV coords                                    |
| Badge       | `<Badge>`       | `text`, `color`, `bg`, `size`             | Small pill label                                                      |
| Divider     | `<Divider>`     | `color`, `thickness`, `margin`            | Horizontal rule                                                       |
| ProgressBar | `<ProgressBar>` | `value`, `max`, `color`, `height`         |                                                                       |
| Spinner     | `<Spinner>`     | `size`, `color`                           | Loading indicator                                                     |
| Tooltip     | `<Tooltip>`     | `text`, `position`                        | Wraps child; shows on long-press; uses `on_layout` for positioning    |

## `<Image>`

| Prop     | Type                                | Default  |
| -------- | ----------------------------------- | -------- |
| `src`    | Asset path or `https://` URL (v1.1) | required |
| `width`  | Dimension                           | `auto`   |
| `height` | Dimension                           | `auto`   |
| `radius` | Number                              | `0`      |
| `fit`    | `cover` \| `contain` \| `fill`      | `cover`  |

Async decode + LRU texture cache. See [Image Loading](/guide/platform/images).

## `<Avatar>`

| Prop       | Type   | Default    |
| ---------- | ------ | ---------- |
| `src`      | String | none       |
| `size`     | Number | `48`       |
| `fallback` | String | none       |
| `radius`   | Number | `size / 2` |

If `src` fails to load and `fallback` is set, renders the fallback string (typically initials) centered in a coloured circle.

## `<Icon>`

| Prop    | Type   | Default  | Notes                           |
| ------- | ------ | -------- | ------------------------------- |
| `name`  | String | required | Maps to UV coords in icon atlas |
| `size`  | Number | `24`     |                                 |
| `color` | Color  | `#000`   |                                 |

Icon system source TBD (see PRD §19 open questions).

## `<Badge>`

| Prop    | Type         | Default   |
| ------- | ------------ | --------- |
| `text`  | String       | required  |
| `color` | Color        | `#fff`    |
| `bg`    | Color        | `#4F8EF7` |
| `size`  | `sm` \| `md` | `sm`      |

## `<Divider>`

| Prop        | Type   | Default |
| ----------- | ------ | ------- |
| `color`     | Color  | `#ccc`  |
| `thickness` | Number | `1`     |
| `margin`    | Number | `8`     |

## `<ProgressBar>`

| Prop     | Type   | Default   |
| -------- | ------ | --------- |
| `value`  | Number | `0`       |
| `max`    | Number | `100`     |
| `color`  | Color  | `#4F8EF7` |
| `height` | Number | `4`       |

## `<Spinner>`

| Prop    | Type   | Default   |
| ------- | ------ | --------- |
| `size`  | Number | `24`      |
| `color` | Color  | `#4F8EF7` |

## `<Tooltip>`

Wraps a child component. Shown on long-press.

| Prop       | Type                                   | Default  |
| ---------- | -------------------------------------- | -------- |
| `text`     | String                                 | required |
| `position` | `top` \| `bottom` \| `left` \| `right` | `top`    |

Uses `on_layout` to position relative to the wrapped child without thrashing.
