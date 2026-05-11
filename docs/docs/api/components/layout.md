# Layout Components

:::warning Status: Specification (v1.0)
None of these are implemented yet.
:::

| Component    | Tag              | Key Props                          | Notes                                                     |
| ------------ | ---------------- | ---------------------------------- | --------------------------------------------------------- |
| Screen       | `<Screen>`       | `bg`, `safeArea`                   | Root container, fills viewport, handles safe-area insets  |
| View         | `<View>`         | `bg`, `radius`, `border`, `shadow` | Generic box container                                     |
| Row          | `<Row>`          | `gap`, `align`, `justify`, `wrap`  | `flexDirection: row`                                      |
| Column       | `<Column>`       | `gap`, `align`, `justify`          | `flexDirection: column`                                   |
| Stack        | `<Stack>`        | `align`                            | Absolute-positioned children layered on top of each other |
| ScrollView   | `<ScrollView>`   | `horizontal`, `showsBar`           | `id` required; scroll offset preserved across hot-reload  |
| SafeAreaView | `<SafeAreaView>` | `edges`                            | Platform safe-area inset padding                          |
| Spacer       | `<Spacer>`       | `size`                             | `flex: 1` spacer or fixed gap                             |

## `<Screen>`

The root container of every screen file. Fills the viewport.

| Prop       | Type    | Default | Notes                                                    |
| ---------- | ------- | ------- | -------------------------------------------------------- |
| `bg`       | Color   | none    | Background fill                                          |
| `safeArea` | Boolean | `false` | If `true`, content is inset by platform safe-area insets |

## `<View>`

Generic box container. The most flexible layout primitive.

| Prop     | Type   | Default | Notes                                       |
| -------- | ------ | ------- | ------------------------------------------- |
| `bg`     | Color  | none    | Background fill                             |
| `radius` | Number | `0`     | Corner radius (dp)                          |
| `border` | Border | none    | `"1 #ccc"` (thickness color)                |
| `shadow` | Shadow | none    | `"4 #000 0 2"` (blur color offsetX offsetY) |

## `<Row>` and `<Column>`

Shorthand for `<View>` with `flexDirection: row` or `column` and ergonomic gap/align props.

| Prop              | Values                                                          |
| ----------------- | --------------------------------------------------------------- |
| `gap`             | dp                                                              |
| `align`           | `start` \| `center` \| `end` \| `stretch`                       |
| `justify`         | `start` \| `center` \| `end` \| `spaceBetween` \| `spaceAround` |
| `wrap` (Row only) | `true` \| `false` (default `false`)                             |

## `<Stack>`

Children are absolutely positioned and layered on top of each other in source order.

## `<ScrollView>`

Scrollable via pan gesture. **Requires `id`** so scroll offset can be preserved across hot-reload.

| Prop         | Type    | Default |
| ------------ | ------- | ------- |
| `horizontal` | Boolean | `false` |
| `showsBar`   | Boolean | `true`  |

## `<SafeAreaView>`

| Prop    | Values                           | Notes                          |
| ------- | -------------------------------- | ------------------------------ |
| `edges` | `top,bottom,left,right` or `all` | Comma-separated; default `all` |

## `<Spacer>`

| Prop   | Type | Default | Notes                            |
| ------ | ---- | ------- | -------------------------------- |
| `size` | dp   | none    | Fixed gap. If omitted, `flex: 1` |
