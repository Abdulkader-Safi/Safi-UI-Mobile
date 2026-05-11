# Navigation Components

:::warning Status: Specification (v1.0)
None of these are implemented yet. A built-in navigation stack is **deferred to v1.1**; these components are container/chrome primitives only.
:::

| Component   | Tag             | Key Props                                                | Notes                                      |
| ----------- | --------------- | -------------------------------------------------------- | ------------------------------------------ |
| NavBar      | `<NavBar>`      | `title`, `leftAction`, `rightAction`, `bg`, `titleColor` | Fixed top navigation bar                   |
| TabBar      | `<TabBar>`      | `tabs`, `activeTab`, `onTabChange`, `bg`                 | Bottom tab bar                             |
| Drawer      | `<Drawer>`      | `open`, `onClose`, `side`, `width`                       | `id` required; open/closed state preserved |
| Modal       | `<Modal>`       | `open`, `onClose`, `title`, `size`                       | `id` required; centered modal dialog       |
| BottomSheet | `<BottomSheet>` | `open`, `onClose`, `snapPoints`                          | `id` required; slide-up bottom sheet       |

## `<NavBar>`

| Prop          | Type       | Default |
| ------------- | ---------- | ------- |
| `title`       | String     | none    |
| `leftAction`  | Event name | none    |
| `rightAction` | Event name | none    |
| `bg`          | Color      | `#fff`  |
| `titleColor`  | Color      | `#000`  |

## `<TabBar>`

| Prop          | Type             | Default  |
| ------------- | ---------------- | -------- |
| `tabs`        | Comma list       | required |
| `activeTab`   | String / binding | first    |
| `onTabChange` | Event name       | none     |
| `bg`          | Color            | `#fff`   |

## `<Drawer>`

**Requires `id`**.

| Prop      | Type              | Default |
| --------- | ----------------- | ------- |
| `open`    | Boolean / binding | `false` |
| `onClose` | Event name        | none    |
| `side`    | `left` \| `right` | `left`  |
| `width`   | Dimension         | `280`   |

## `<Modal>`

**Requires `id`**.

| Prop      | Type                           | Default |
| --------- | ------------------------------ | ------- |
| `open`    | Boolean / binding              | `false` |
| `onClose` | Event name                     | none    |
| `title`   | String                         | none    |
| `size`    | `sm` \| `md` \| `lg` \| `full` | `md`    |

## `<BottomSheet>`

**Requires `id`**.

| Prop         | Type                                   | Default |
| ------------ | -------------------------------------- | ------- |
| `open`       | Boolean / binding                      | `false` |
| `onClose`    | Event name                             | none    |
| `snapPoints` | Comma list of percentages (`25,50,90`) | `90`    |
