use glam::Vec2;
use safi_ui::commands::{Color, Command, FontHandle, ImageFit, Rect, TextureHandle};
use safi_ui::gpu::{BatchKind, Batcher, MockRenderer, Renderer};

fn r(w: f32) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        width: w,
        height: 20.0,
    }
}

fn rect_cmd() -> Command {
    Command::Rect {
        rect: r(50.0),
        color: Color::WHITE,
        radius: 0.0,
    }
}

fn text_cmd(font_id: u32) -> Command {
    Command::Text {
        pos: Vec2::ZERO,
        text: "hi".into(),
        font: FontHandle(font_id),
        size: 14.0,
        color: Color::BLACK,
    }
}

fn image_cmd(tex_id: u32) -> Command {
    Command::Image {
        rect: r(50.0),
        texture: TextureHandle(tex_id),
        radius: 0.0,
        fit: ImageFit::Cover,
    }
}

fn clip_cmd() -> Command {
    Command::Clip { rect: r(100.0) }
}

// ---- batching unit tests ----

#[test]
fn empty_command_buffer_yields_no_batches() {
    assert!(Batcher::batch(&[]).is_empty());
}

#[test]
fn single_rect_yields_one_rect_batch() {
    let batches = Batcher::batch(&[rect_cmd()]);
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].kind, BatchKind::Rect);
    assert_eq!(batches[0].range, 0..1);
}

#[test]
fn consecutive_rects_fold_into_one_batch() {
    let cmds = vec![rect_cmd(), rect_cmd(), rect_cmd()];
    let batches = Batcher::batch(&cmds);
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].range, 0..3);
}

#[test]
fn rect_text_rect_yields_three_batches() {
    let cmds = vec![rect_cmd(), text_cmd(0), rect_cmd()];
    let batches = Batcher::batch(&cmds);
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].kind, BatchKind::Rect);
    assert_eq!(
        batches[1].kind,
        BatchKind::Text {
            font: FontHandle(0)
        }
    );
    assert_eq!(batches[2].kind, BatchKind::Rect);
}

#[test]
fn clip_breaks_rect_run() {
    let cmds = vec![rect_cmd(), clip_cmd(), rect_cmd()];
    let batches = Batcher::batch(&cmds);
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].kind, BatchKind::Rect);
    assert_eq!(batches[1].kind, BatchKind::ClipPush);
    assert_eq!(batches[1].range, 1..2);
    assert_eq!(batches[2].kind, BatchKind::Rect);
}

#[test]
fn clip_pop_is_its_own_batch() {
    let cmds = vec![rect_cmd(), Command::ClipPop, rect_cmd()];
    let batches = Batcher::batch(&cmds);
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[1].kind, BatchKind::ClipPop);
    assert_eq!(batches[1].range, 1..2);
}

#[test]
fn image_batches_split_on_texture_change() {
    let cmds = vec![image_cmd(1), image_cmd(1), image_cmd(2)];
    let batches = Batcher::batch(&cmds);
    assert_eq!(batches.len(), 2);
    assert_eq!(
        batches[0].kind,
        BatchKind::Image {
            texture: TextureHandle(1)
        }
    );
    assert_eq!(batches[0].range, 0..2);
    assert_eq!(
        batches[1].kind,
        BatchKind::Image {
            texture: TextureHandle(2)
        }
    );
}

#[test]
fn text_batches_split_on_font_change() {
    let cmds = vec![text_cmd(1), text_cmd(2)];
    let batches = Batcher::batch(&cmds);
    assert_eq!(batches.len(), 2);
}

// ---- MockRenderer ----

#[test]
fn mock_renderer_records_batches() {
    let mut r = MockRenderer::new();
    r.begin_frame(2.0);
    r.submit(&[rect_cmd(), rect_cmd(), text_cmd(0)]);
    r.end_frame();
    assert_eq!(r.frames, 1);
    assert_eq!(r.dpi.to_bits(), 2.0_f32.to_bits());
    assert_eq!(r.batches.len(), 2);
    assert_eq!(r.batches[0].kind, BatchKind::Rect);
    assert_eq!(
        r.batches[1].kind,
        BatchKind::Text {
            font: FontHandle(0)
        }
    );
}

#[test]
fn mock_renderer_lifecycle_counters() {
    let mut r = MockRenderer::new();
    r.release_resources();
    r.release_resources();
    r.recreate_resources();
    assert_eq!(r.releases, 2);
    assert_eq!(r.recreates, 1);
}

// ---- Phase 1 acceptance: 50-80 widgets → ≤ 15 GPU draws ----

#[test]
fn phase1_acceptance_50_to_80_widgets_under_15_batches() {
    let mut cmds: Vec<Command> = Vec::new();

    // Frame structure: outer clip, 8 alternating runs of Rect/Text, inner
    // clip in the middle, then unclip both. 60 total commands.
    cmds.push(clip_cmd());
    for _ in 0..5 {
        cmds.push(rect_cmd());
    }
    for _ in 0..3 {
        cmds.push(text_cmd(0));
    }
    for _ in 0..5 {
        cmds.push(rect_cmd());
    }
    for _ in 0..2 {
        cmds.push(text_cmd(0));
    }
    cmds.push(clip_cmd());
    for _ in 0..8 {
        cmds.push(rect_cmd());
    }
    for _ in 0..5 {
        cmds.push(text_cmd(0));
    }
    for _ in 0..7 {
        cmds.push(rect_cmd());
    }
    for _ in 0..3 {
        cmds.push(text_cmd(0));
    }
    for _ in 0..7 {
        cmds.push(rect_cmd());
    }
    for _ in 0..3 {
        cmds.push(text_cmd(0));
    }
    cmds.push(Command::ClipPop);
    cmds.push(Command::ClipPop);

    let count = cmds.len();
    assert!(
        (50..=80).contains(&count),
        "test fixture should land 50..=80 commands, got {count}"
    );

    let batches = Batcher::batch(&cmds);
    assert!(
        batches.len() <= 15,
        "expected <= 15 batches, got {} for {} commands",
        batches.len(),
        count
    );
}
