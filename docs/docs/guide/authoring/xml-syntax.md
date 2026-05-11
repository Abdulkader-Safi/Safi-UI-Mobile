# XML Syntax

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned XML authoring contract.
:::

Safi-UI screens and components are authored in **standard XML files**. No custom dialect, no preprocessor, no template language other than `{{key}}` bindings.

## File rules

| Rule                | Detail                                               |
| ------------------- | ---------------------------------------------------- |
| Encoding            | UTF-8 required                                       |
| Root element        | Each screen file must have exactly one root element  |
| File extension      | `.xml`                                               |
| Screens location    | `assets/ui/screens/`                                 |
| Components location | `assets/ui/components/`                              |
| Images location     | `assets/images/`                                     |
| Screen naming       | lowercase-hyphen: `home-screen.xml`                  |
| Component naming    | PascalCase: `UserCard.xml`                           |
| Comments            | Standard XML: `<!-- -->`                             |
| Max nesting depth   | No hard limit; 20+ levels logs a performance warning |

## Prop value types

| Type              | Format / examples                                                               |
| ----------------- | ------------------------------------------------------------------------------- |
| String            | `label="Sign In"`                                                               |
| Number            | `size="18"`, `padding="12"`, `opacity="0.8"`                                    |
| Boolean           | `disabled="true"`, `bold="false"`                                               |
| Color             | `"#RRGGBB"`, `"#AARRGGBB"`, `"rgba(255,100,0,0.5)"`, `"white"`, `"transparent"` |
| Dimension         | `"200"` (dp), `"50%"` (percent of parent), `"auto"`                             |
| Binding           | `"{{stateKey}}"` — resolves from StateStore; missing key → `""`                 |
| Composite binding | `"Hello {{name}}!"`, `"{{first}} {{last}}"`                                     |
| Event name        | `"auth.login"`, `"nav.back"`, `"{{dynamicAction}}"` — bindings allowed          |

## Special props (all components)

| Prop                 | Purpose                                                                                                              |
| -------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `id`                 | Globally unique. **Required for stateful components.** Used for StateStore bindings, EventBus targeting, hot-reload. |
| `key`                | Sibling-scoped. Required on FlatList items for state preservation across data reorders.                              |
| `visible`            | Boolean; hides component but preserves layout space. Does **not** unmount.                                           |
| `opacity`            | 0.0–1.0 float; applied to entire subtree                                                                             |
| `testID`             | String identifier for automated UI testing                                                                           |
| `onMount`            | Event name fired when component first appears in the tree                                                            |
| `onUnmount`          | Event name fired when component is removed (or recycled out of FlatList window)                                      |
| `accessibilityLabel` | Reserved for v2 accessibility support                                                                                |
| `accessibilityRole`  | Reserved for v2 accessibility support                                                                                |

## Layout props (most components)

`width`, `height`, `padding`, `margin`, `flex`, `gap`, `align`, `justify`, `wrap`, `flexDirection` — see the [Built-in Components reference](/api/components/) for which props each component accepts.

## Example

```xml
<Screen bg="#0f0f1a" safeArea="true">
  <NavBar title="Dashboard" bg="#1a1a2e" titleColor="#fff" />

  <ScrollView id="main-scroll" flex="1" padding="16">
    <UserCard name="{{user.name}}"
              avatar="{{user.avatar}}"
              role="{{user.role}}"
              onPress="nav.profile" />

    <Spacer size="16" />

    <Row gap="12" justify="spaceBetween">
      <Card flex="1" bg="#1e1e2e" padding="16" radius="12">
        <Label color="#4F8EF7">PROJECTS</Label>
        <Heading level="2" color="#fff">{{stats.projects}}</Heading>
      </Card>
    </Row>
  </ScrollView>
</Screen>
```
