# 27 — `PlatformBridge`: safe area, keyboard, lifecycle

**Phase:** 6 — Platform Polish
**PRD refs:** §9.1, §9.2, §9.4

## Goal

Platform-specific bridges for safe-area insets, keyboard height, and lifecycle events — surfaced to Rust via SDL3 and small JNI/ObjC shims.

## Deliverables

- `safi-ui::platform::PlatformBridge` trait
- `AndroidPlatformBridge` via JNI:
  - `WindowInsetsCompat` → `safe_area()` insets
  - Keyboard show/hide → `keyboard_height()`
- `IosPlatformBridge` via ObjC bridge:
  - `UIView.safeAreaInsets` → `safe_area()`
  - `UIKeyboardWillShow/HideNotification` → `keyboard_height()`
- `SDL_EVENT_DISPLAY_ORIENTATION` triggers Taffy re-layout
- Lifecycle wiring: `WILL_ENTER_BACKGROUND` releases GPU resources, `DID_ENTER_FOREGROUND` recreates them
- `SafeAreaView` consumes `PlatformBridge::safe_area()` (real values now, not stubs)

## Dependencies

- `02`, `12`, `19` (keyboard interaction)

## Acceptance

- Notch + home indicator handled correctly on iPhone 15 Pro
- Status bar + nav bar handled correctly on Pixel 8 (gesture nav and 3-button nav)
- Soft keyboard appearance pushes focused input into view
- Orientation change re-lays out the screen without restart
