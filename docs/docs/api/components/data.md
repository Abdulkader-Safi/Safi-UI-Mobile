# Data Components

:::warning Status: Specification (v1.0)
None of these are implemented yet.
:::

| Component  | Tag            | Key Props                                         | Notes                                                                                            |
| ---------- | -------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| FlatList   | `<FlatList>`   | `data`, `renderItem`, `keyExtractor`, `separator` | Windowed recycling. 1k+ items. Reverse-infinite-scroll supported. `key` prop required on items.  |
| Card       | `<Card>`       | `elevation`, `bg`, `radius`, `border`, `padding`  | Elevated container                                                                               |
| Table      | `<Table>`      | `columns`, `data`, `striped`, `border`            | Grid table                                                                                       |
| EmptyState | `<EmptyState>` | `icon`, `title`, `message`, `action`              | Placeholder for empty lists                                                                      |

## `<FlatList>`

**Requires `id`**. Uses windowed recycling from v1. Recycled item components fire `on_unmount` when scrolled out and `on_mount` when recycled back into view.

| Prop           | Type        | Default  |
| -------------- | ----------- | -------- |
| `data`         | StateStore key (binding)| required |
| `renderItem`   | Component name | required |
| `keyExtractor` | Property name on each data item | required |
| `separator`    | dp          | `0`      |
| `reverse`      | Boolean     | `false`  |

The `key` prop (not `WidgetId`) is used for state preservation across data reorders. **Without a `key`, item state cannot be preserved when the data array reorders.**

```xml
<FlatList id="projects-list"
          data="projects.recent"
          renderItem="ProjectCard"
          keyExtractor="id"
          separator="8" />
```

`reverse="true"` enables chat-style reverse-infinite-scroll.

## `<Card>`

| Prop        | Type   | Default     |
| ----------- | ------ | ----------- |
| `elevation` | Number | `2`         |
| `bg`        | Color  | `#fff`      |
| `radius`    | Number | `12`        |
| `border`    | Border | none        |
| `padding`   | dp     | `16`        |

## `<Table>`

| Prop      | Type                         | Default  |
| --------- | ---------------------------- | -------- |
| `columns` | Comma list of column headers | required |
| `data`    | StateStore key (binding)     | required |
| `striped` | Boolean                      | `false`  |
| `border`  | Boolean                      | `true`   |

## `<EmptyState>`

| Prop      | Type        | Default  |
| --------- | ----------- | -------- |
| `icon`    | Icon name   | none     |
| `title`   | String      | none     |
| `message` | String      | none     |
| `action`  | Component XML | none   |
