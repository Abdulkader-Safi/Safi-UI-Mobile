use glam::Vec2;
use safi_ui::commands::{
    Color, Command, CommandBuffer, FontHandle, ImageFit, Rect, TextureHandle, WidgetRange,
    COMMAND_BUFFER_CAPACITY_DEFAULT,
};

fn rect() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    }
}

fn solid(color: Color) -> Command {
    Command::Rect {
        rect: rect(),
        color,
        radius: 0.0,
    }
}

fn red() -> Command {
    solid(Color::rgb(255, 0, 0))
}

#[test]
fn default_capacity_is_8192() {
    let buf = CommandBuffer::new();
    assert_eq!(buf.initial_capacity(), COMMAND_BUFFER_CAPACITY_DEFAULT);
    assert_eq!(buf.initial_capacity(), 8192);
}

#[test]
fn push_increments_len_and_appears_in_slice() {
    let mut buf = CommandBuffer::with_capacity(16);
    buf.push(red());
    buf.push(solid(Color::WHITE));
    assert_eq!(buf.len(), 2);
    assert!(!buf.is_empty());
    assert_eq!(buf.as_slice().len(), 2);
    assert_eq!(buf.as_slice()[1], solid(Color::WHITE));
}

#[test]
fn clear_resets_state_and_warn_latches() {
    let mut buf = CommandBuffer::with_capacity(4);
    for _ in 0..5 {
        buf.push(red());
    }
    buf.begin_widget(7);
    buf.push(red());
    buf.end_widget();
    assert!(!buf.ranges().is_empty());
    assert!(buf.overflow_warn_count() >= 1);

    buf.clear();
    assert!(buf.is_empty());
    assert_eq!(buf.len(), 0);
    assert_eq!(buf.as_slice().len(), 0);
    assert!(buf.ranges().is_empty());

    // Latches reset: crossing 75% of initial_capacity again should warn.
    // (Threshold is measured vs initial_capacity, so clear()'s retained Vec
    // capacity doesn't suppress it the way it does for the overflow latch.)
    let prev_threshold = buf.threshold_warn_count();
    for _ in 0..3 {
        buf.push(red()); // 3/4 = 75%
    }
    assert_eq!(buf.threshold_warn_count(), prev_threshold + 1);
}

#[test]
fn begin_end_records_widget_range() {
    let mut buf = CommandBuffer::with_capacity(16);
    buf.begin_widget(42);
    buf.push(red());
    buf.push(red());
    buf.end_widget();

    assert_eq!(
        buf.ranges(),
        &[WidgetRange {
            widget: 42,
            start: 0,
            end: 2
        }]
    );
}

#[test]
fn nested_begin_end_records_ranges_in_close_order() {
    let mut buf = CommandBuffer::with_capacity(32);

    buf.begin_widget(1); // outer
    buf.push(red());

    buf.begin_widget(2); // first inner
    buf.push(red());
    buf.push(red());
    buf.end_widget();

    buf.begin_widget(3); // second inner
    buf.push(red());
    buf.end_widget();

    buf.push(red());
    buf.end_widget(); // outer

    assert_eq!(
        buf.ranges(),
        &[
            WidgetRange {
                widget: 2,
                start: 1,
                end: 3,
            },
            WidgetRange {
                widget: 3,
                start: 3,
                end: 4,
            },
            WidgetRange {
                widget: 1,
                start: 0,
                end: 5,
            },
        ]
    );
}

#[test]
fn empty_widget_scope_records_no_range() {
    let mut buf = CommandBuffer::with_capacity(16);
    buf.begin_widget(9);
    buf.end_widget();
    assert!(buf.ranges().is_empty());
}

#[test]
fn growing_past_initial_capacity_warns_exactly_once() {
    let mut buf = CommandBuffer::with_capacity(4);
    for _ in 0..5 {
        buf.push(red());
    }
    assert_eq!(buf.len(), 5);
    assert_eq!(buf.as_slice().len(), 5);
    assert_eq!(buf.overflow_warn_count(), 1);

    for _ in 0..10 {
        buf.push(red());
    }
    assert_eq!(buf.overflow_warn_count(), 1, "latched within frame");

    buf.on_frame_complete();
    // Vec has already grown; push won't trip "len == capacity" naturally.
    // Force re-overflow by pushing until we hit current capacity again.
    while buf.len() < buf.capacity() {
        buf.push(red());
    }
    buf.push(red());
    assert_eq!(buf.overflow_warn_count(), 2, "re-armed across frames");
}

#[test]
fn threshold_warn_fires_once_per_frame_at_75_percent() {
    let mut buf = CommandBuffer::with_capacity(8);
    for _ in 0..5 {
        buf.push(red());
    }
    assert_eq!(buf.threshold_warn_count(), 0, "62.5% < 75%");

    buf.push(red()); // 6/8 = 75%
    assert_eq!(buf.threshold_warn_count(), 1);

    for _ in 0..10 {
        buf.push(red());
    }
    assert_eq!(buf.threshold_warn_count(), 1, "latched within frame");

    buf.on_frame_complete();
    buf.push(red());
    assert_eq!(buf.threshold_warn_count(), 2, "re-armed across frames");
}

#[test]
fn all_command_variants_construct() {
    let cmds = vec![
        Command::Rect {
            rect: rect(),
            color: Color::WHITE,
            radius: 4.0,
        },
        Command::Border {
            rect: rect(),
            color: Color::BLACK,
            radius: 4.0,
            thickness: 1.0,
        },
        Command::Text {
            pos: Vec2::ZERO,
            text: "hi".into(),
            font: FontHandle(0),
            size: 14.0,
            color: Color::WHITE,
        },
        Command::Image {
            rect: rect(),
            texture: TextureHandle(0),
            radius: 0.0,
            fit: ImageFit::Cover,
        },
        Command::Shadow {
            rect: rect(),
            color: Color::rgba(0, 0, 0, 128),
            blur: 8.0,
            offset: Vec2::new(0.0, 2.0),
        },
        Command::Clip { rect: rect() },
        Command::ClipPop,
    ];
    let mut buf = CommandBuffer::with_capacity(cmds.len());
    for c in &cmds {
        buf.push(c.clone());
    }
    assert_eq!(buf.len(), cmds.len());
}

#[test]
fn color_helpers() {
    assert_eq!(Color::TRANSPARENT, Color::rgba(0, 0, 0, 0));
    assert_eq!(Color::WHITE, Color::rgba(255, 255, 255, 255));
    assert_eq!(Color::rgb(1, 2, 3), Color::rgba(1, 2, 3, 255));
}
