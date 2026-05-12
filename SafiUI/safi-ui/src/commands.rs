//! Typed draw-command buffer (PRD §5.1 Pillar 1, §6.3).
//!
//! All rendering flows through a flat list of [`Command`]s. Components emit
//! into a [`CommandBuffer`]; the GPU renderer consumes [`CommandBuffer::as_slice`].
//! No component ever calls GPU APIs directly.

use glam::Vec2;

use crate::arena::WidgetId;
use crate::vnode::LayoutRect;

pub const COMMAND_BUFFER_CAPACITY_DEFAULT: usize = 8192;
const OVERFLOW_WARN_THRESHOLD: f32 = 0.75;

pub type Rect = LayoutRect;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::rgba(0, 0, 0, 255);
    pub const WHITE: Self = Self::rgba(255, 255, 255, 255);

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }
}

/// Opaque font handle. Promoted to a real handle by todo `16` (font atlas).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct FontHandle(pub u32);

/// Opaque GPU texture handle. Promoted to a real handle by todo `17` (image
/// pipeline).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
    Fill,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Rect {
        rect: Rect,
        color: Color,
        radius: f32,
    },
    Border {
        rect: Rect,
        color: Color,
        radius: f32,
        thickness: f32,
    },
    Text {
        pos: Vec2,
        text: String,
        font: FontHandle,
        size: f32,
        color: Color,
    },
    Image {
        rect: Rect,
        texture: TextureHandle,
        radius: f32,
        fit: ImageFit,
    },
    Shadow {
        rect: Rect,
        color: Color,
        blur: f32,
        offset: Vec2,
    },
    Clip {
        rect: Rect,
    },
    ClipPop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetRange {
    pub widget: WidgetId,
    pub start: usize,
    pub end: usize,
}

pub struct CommandBuffer {
    commands: Vec<Command>,
    ranges: Vec<WidgetRange>,
    open: Vec<(WidgetId, usize)>,
    initial_capacity: usize,
    overflow_warned: bool,
    threshold_warned: bool,
    #[cfg(any(test, debug_assertions))]
    overflow_warn_count: u32,
    #[cfg(any(test, debug_assertions))]
    threshold_warn_count: u32,
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::with_capacity(COMMAND_BUFFER_CAPACITY_DEFAULT)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            commands: Vec::with_capacity(cap),
            ranges: Vec::new(),
            open: Vec::new(),
            initial_capacity: cap,
            overflow_warned: false,
            threshold_warned: false,
            #[cfg(any(test, debug_assertions))]
            overflow_warn_count: 0,
            #[cfg(any(test, debug_assertions))]
            threshold_warn_count: 0,
        }
    }

    pub fn push(&mut self, cmd: Command) {
        if self.commands.len() == self.commands.capacity() && !self.overflow_warned {
            eprintln!(
                "safi-ui: CommandBuffer overflow (cap {} -> growing)",
                self.commands.capacity()
            );
            #[cfg(any(test, debug_assertions))]
            {
                self.overflow_warn_count += 1;
            }
            self.overflow_warned = true;
        }

        self.commands.push(cmd);

        if !self.threshold_warned {
            let cap = self.initial_capacity.max(1);
            #[allow(clippy::cast_precision_loss)]
            let utilisation = self.commands.len() as f32 / cap as f32;
            if utilisation >= OVERFLOW_WARN_THRESHOLD {
                #[cfg(debug_assertions)]
                eprintln!(
                    "safi-ui: CommandBuffer at {:.0}% of initial capacity {}",
                    utilisation * 100.0,
                    self.initial_capacity
                );
                #[cfg(any(test, debug_assertions))]
                {
                    self.threshold_warn_count += 1;
                }
                self.threshold_warned = true;
            }
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.ranges.clear();
        self.open.clear();
        self.overflow_warned = false;
        self.threshold_warned = false;
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.commands.capacity()
    }

    pub fn initial_capacity(&self) -> usize {
        self.initial_capacity
    }

    pub fn as_slice(&self) -> &[Command] {
        &self.commands
    }

    pub fn begin_widget(&mut self, id: WidgetId) {
        self.open.push((id, self.commands.len()));
    }

    pub fn end_widget(&mut self) {
        let Some((widget, start)) = self.open.pop() else {
            debug_assert!(false, "CommandBuffer::end_widget without matching begin");
            eprintln!("safi-ui: CommandBuffer::end_widget without matching begin");
            return;
        };
        let end = self.commands.len();
        if end > start {
            self.ranges.push(WidgetRange { widget, start, end });
        }
    }

    pub fn ranges(&self) -> &[WidgetRange] {
        &self.ranges
    }

    pub fn on_frame_complete(&mut self) {
        self.overflow_warned = false;
        self.threshold_warned = false;
    }

    #[cfg(any(test, debug_assertions))]
    pub fn overflow_warn_count(&self) -> u32 {
        self.overflow_warn_count
    }

    #[cfg(any(test, debug_assertions))]
    pub fn threshold_warn_count(&self) -> u32 {
        self.threshold_warn_count
    }
}
