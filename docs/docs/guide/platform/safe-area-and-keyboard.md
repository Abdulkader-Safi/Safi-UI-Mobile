# Safe Area and Keyboard

:::warning Status: Specification (v1.0)
Not yet implemented. This page describes the planned safe-area and keyboard handling.
:::

## Safe area

The `<SafeAreaView>` component queries `PlatformBridge::safe_area()` and adds padding insets automatically.

```xml
<Screen>
  <SafeAreaView edges="top,bottom">
    <!-- content here is inset to avoid notches and home indicators -->
  </SafeAreaView>
</Screen>
```

Insets come from:

- **Android:** `WindowInsetsCompat` via JNI bridge
- **iOS:** `UIView.safeAreaInsets` via Objective-C bridge

`<Screen safeArea="true">` is shorthand for wrapping the screen content in a `<SafeAreaView edges="all">`.

## Keyboard layout

When the soft keyboard appears, `PlatformBridge::keyboard_height()` returns the current height in dp. Taffy re-runs layout with a reduced available height, **pushing focused inputs into the visible area** automatically.

| Platform | Source                                    |
| -------- | ----------------------------------------- |
| Android  | `SDL_EVENT_TEXT_INPUT` + JNI bridge       |
| iOS      | `UIKeyboardWillShowNotification` via ObjC |

## Density-independent pixels

All XML coordinates are in **dp**. Conversion to physical pixels happens at the `GpuRenderer` boundary:

```
physical_pixels = dp_value × dpi_scale

Pixel 8 (420 dpi):   dpi_scale = 2.625
iPhone 15 Pro:        dpi_scale = 3.0
```

`SDL_GetDisplayContentScale()` provides the scale factor at runtime.
