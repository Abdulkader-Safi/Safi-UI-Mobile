# Input Components

:::warning Status: Specification (v1.0)
None of these are implemented yet.
:::

| Component | Tag          | Key Props                                                 | Notes                                                      |
| --------- | ------------ | --------------------------------------------------------- | ---------------------------------------------------------- |
| Button    | `<Button>`   | `label`, `onPress`, `variant`, `size`, `icon`, `disabled` | `variant`: `primary` \| `secondary` \| `ghost` \| `danger` |
| Input     | `<Input>`    | `placeholder`, `value`, `onChange`, `type`, `maxLength`   | `id` required; cursor position preserved                   |
| TextArea  | `<TextArea>` | `placeholder`, `rows`, `onChange`                         | `id` required; multiline                                   |
| Checkbox  | `<Checkbox>` | `label`, `checked`, `onChange`                            |                                                            |
| Switch    | `<Switch>`   | `value`, `onChange`, `label`                              | Toggle switch                                              |
| Select    | `<Select>`   | `options`, `value`, `onChange`, `placeholder`             | Dropdown picker                                            |
| Slider    | `<Slider>`   | `min`, `max`, `value`, `step`, `onChange`                 |                                                            |

## `<Button>`

| Prop       | Type                                            | Default    |
| ---------- | ----------------------------------------------- | ---------- |
| `label`    | String                                          | `"Button"` |
| `onPress`  | Event name                                      | none       |
| `variant`  | `primary` \| `secondary` \| `ghost` \| `danger` | `primary`  |
| `size`     | `sm` \| `md` \| `lg`                            | `md`       |
| `icon`     | Icon name                                       | none       |
| `disabled` | Boolean                                         | `false`    |

## `<Input>`

**Requires `id`** so cursor position can survive hot-reload.

| Prop          | Type                                        | Default   |
| ------------- | ------------------------------------------- | --------- |
| `placeholder` | String                                      | `""`      |
| `value`       | String / binding                            | `""`      |
| `onChange`    | Event name                                  | none      |
| `type`        | `text` \| `password` \| `number` \| `email` | `text`    |
| `maxLength`   | Number                                      | unlimited |

## `<TextArea>`

**Requires `id`**.

| Prop          | Type       | Default |
| ------------- | ---------- | ------- |
| `placeholder` | String     | `""`    |
| `rows`        | Number     | `4`     |
| `onChange`    | Event name | none    |

## `<Checkbox>`

| Prop       | Type       | Default |
| ---------- | ---------- | ------- |
| `label`    | String     | none    |
| `checked`  | Boolean    | `false` |
| `onChange` | Event name | none    |

## `<Switch>`

| Prop       | Type       | Default |
| ---------- | ---------- | ------- |
| `value`    | Boolean    | `false` |
| `onChange` | Event name | none    |
| `label`    | String     | none    |

## `<Select>`

| Prop          | Type                 | Default |
| ------------- | -------------------- | ------- |
| `options`     | Comma list / binding | `[]`    |
| `value`       | String               | none    |
| `onChange`    | Event name           | none    |
| `placeholder` | String               | `""`    |

## `<Slider>`

| Prop       | Type       | Default |
| ---------- | ---------- | ------- |
| `min`      | Number     | `0`     |
| `max`      | Number     | `100`   |
| `value`    | Number     | `0`     |
| `step`     | Number     | `1`     |
| `onChange` | Event name | none    |
