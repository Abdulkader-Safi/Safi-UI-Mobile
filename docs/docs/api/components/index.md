# Built-in Components

:::warning Status: Specification (v1.0)
None of the components below are implemented yet. Target: 30+ shipping at v1.0.
:::

## Base props (every component)

Every built-in accepts these in addition to its specific props:

| Prop                 | Notes                                                                |
| -------------------- | -------------------------------------------------------------------- |
| `width`, `height`    | dp / `%` / `auto`                                                    |
| `padding`, `margin`  | dp on each edge or per-edge: `padding="12 8"` (vertical horizontal)  |
| `flex`               | `flex_grow`                                                          |
| `visible`            | Hides but preserves layout space; does **not** unmount               |
| `opacity`            | 0.0–1.0; applies to entire subtree                                   |
| `id`                 | Globally unique; **required for stateful components**                |
| `key`                | Sibling-scoped; required on FlatList items                           |
| `testID`             | Identifier for automated UI tests                                    |
| `onMount`            | Event name fired on tree-entry                                       |
| `onUnmount`          | Event name fired on tree-leave                                       |
| `accessibilityLabel` | Reserved for v2                                                      |
| `accessibilityRole`  | Reserved for v2                                                      |

`visible="false"` hides the component and preserves layout space but does **not** fire `on_unmount`. The component instance remains alive and its state is preserved.

## Categories

| Category                                        | Tags                                                                                              |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| [Layout](/api/components/layout)                | `<Screen>`, `<View>`, `<Row>`, `<Column>`, `<Stack>`, `<ScrollView>`, `<SafeAreaView>`, `<Spacer>` |
| [Typography](/api/components/typography)        | `<Text>`, `<Heading>`, `<Label>`, `<Code>`                                                        |
| [Input](/api/components/input)                  | `<Button>`, `<Input>`, `<TextArea>`, `<Checkbox>`, `<Switch>`, `<Select>`, `<Slider>`             |
| [Display](/api/components/display)              | `<Image>`, `<Avatar>`, `<Icon>`, `<Badge>`, `<Divider>`, `<ProgressBar>`, `<Spinner>`, `<Tooltip>` |
| [Navigation](/api/components/navigation)        | `<NavBar>`, `<TabBar>`, `<Drawer>`, `<Modal>`, `<BottomSheet>`                                    |
| [Data](/api/components/data)                    | `<FlatList>`, `<Card>`, `<Table>`, `<EmptyState>`                                                 |
