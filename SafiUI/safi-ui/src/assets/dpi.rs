//! Density-independent pixel scaling (PRD §7.3).
//!
//! All XML coordinates are authored in dp. The renderer converts to
//! physical pixels at the GPU boundary by multiplying by [`DpiScale`].
//! The scale is resolved once at startup from
//! `SDL_GetDisplayContentScale` and stored on `UIContext.dpi_scale`.
//!
//! Reference values per PRD §7.3:
//!
//! | Device           | `DpiScale` |
//! | ---------------- | ---------- |
//! | Pixel 8 (420dpi) | 2.625      |
//! | iPhone 15 Pro    | 3.0        |
//! | Desktop 1080p    | 1.0        |

/// A density-independent → physical-pixel scale factor.
///
/// Wraps the raw `f32` from SDL so callers can't accidentally swap
/// dp and physical values at function boundaries. The inner field
/// is `pub` for ergonomic destructuring in renderer hot paths.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DpiScale(pub f32);

impl DpiScale {
    /// Identity scale — useful for host tests and desktop preview.
    pub const ONE: Self = Self(1.0);

    /// Build a `DpiScale`, clamping non-finite or non-positive inputs
    /// to `1.0`. SDL3 returns `0.0` if the display has no content scale
    /// information; we treat that as "render 1:1" rather than panic.
    pub fn from_sdl(raw: f32) -> Self {
        if raw.is_finite() && raw > 0.0 {
            Self(raw)
        } else {
            Self::ONE
        }
    }

    /// Convert a dp value to physical pixels.
    #[inline]
    pub fn dp_to_physical(self, dp: f32) -> f32 {
        dp * self.0
    }

    /// Convert a physical-pixel value back to dp. Mainly useful for
    /// translating raw touch coordinates into the layout's dp space.
    #[inline]
    pub fn physical_to_dp(self, physical: f32) -> f32 {
        physical / self.0
    }

    /// Raw scalar — read-only access for renderer code that already
    /// works in `f32`.
    #[inline]
    pub fn raw(self) -> f32 {
        self.0
    }
}

impl Default for DpiScale {
    fn default() -> Self {
        Self::ONE
    }
}

impl From<DpiScale> for f32 {
    fn from(s: DpiScale) -> f32 {
        s.0
    }
}
