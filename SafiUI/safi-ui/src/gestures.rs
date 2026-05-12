//! Gesture recognition (PRD §5.3, §6.9).
//!
//! Converts SDL3-style finger events into the four canonical [`Gesture`]
//! variants and dispatches them via reverse-Z hit testing through the
//! [`WidgetArena`]. Decoupled from SDL: callers translate `SDL_EVENT_FINGER_*`
//! to the four `finger_*` entry points, passing a monotonic `timestamp_ms`
//! (e.g. `SDL_GetTicks()`).

use std::collections::{HashMap, VecDeque};

use glam::Vec2;

use crate::arena::WidgetArena;

pub type FingerId = i64;

pub const TAP_DURATION_MS: u64 = 200;
pub const LONG_PRESS_DURATION_MS: u64 = 500;
pub const MOVEMENT_THRESHOLD_DP: f32 = 10.0;
pub const SWIPE_VELOCITY_THRESHOLD_DP_PER_SEC: f32 = 800.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    Tap {
        pos: Vec2,
    },
    LongPress {
        pos: Vec2,
    },
    Pan {
        delta: Vec2,
        velocity: Vec2,
    },
    Swipe {
        direction: SwipeDirection,
        velocity: f32,
    },
}

struct FingerState {
    down_pos: Vec2,
    last_pos: Vec2,
    down_time_ms: u64,
    last_motion_ms: u64,
    last_velocity: Vec2,
    max_movement_dp: f32,
    is_panning: bool,
    long_press_fired: bool,
}

impl FingerState {
    fn new(pos: Vec2, timestamp_ms: u64) -> Self {
        Self {
            down_pos: pos,
            last_pos: pos,
            down_time_ms: timestamp_ms,
            last_motion_ms: timestamp_ms,
            last_velocity: Vec2::ZERO,
            max_movement_dp: 0.0,
            is_panning: false,
            long_press_fired: false,
        }
    }
}

#[derive(Default)]
pub struct GestureRecognizer {
    fingers: HashMap<FingerId, FingerState>,
    pending: VecDeque<(Gesture, Vec2)>,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finger_down(&mut self, id: FingerId, pos: Vec2, timestamp_ms: u64) {
        self.fingers.insert(id, FingerState::new(pos, timestamp_ms));
    }

    pub fn finger_motion(&mut self, id: FingerId, pos: Vec2, timestamp_ms: u64) {
        let Some(s) = self.fingers.get_mut(&id) else {
            return;
        };
        let delta = pos - s.last_pos;
        let dt_ms = timestamp_ms.saturating_sub(s.last_motion_ms).max(1);
        #[allow(clippy::cast_precision_loss)]
        // session timestamps fit in f32 mantissa for any realistic SDL run
        let dt_sec = dt_ms as f32 / 1000.0;
        let velocity = delta / dt_sec;
        s.last_velocity = velocity;
        let total_movement = (pos - s.down_pos).length();
        if total_movement > s.max_movement_dp {
            s.max_movement_dp = total_movement;
        }
        if s.max_movement_dp > MOVEMENT_THRESHOLD_DP && !s.is_panning {
            s.is_panning = true;
        }
        if s.is_panning {
            self.pending
                .push_back((Gesture::Pan { delta, velocity }, pos));
        }
        s.last_pos = pos;
        s.last_motion_ms = timestamp_ms;
    }

    pub fn finger_up(&mut self, id: FingerId, pos: Vec2, timestamp_ms: u64) {
        let Some(s) = self.fingers.remove(&id) else {
            return;
        };
        let duration = timestamp_ms.saturating_sub(s.down_time_ms);

        if s.is_panning {
            let speed = s.last_velocity.length();
            if speed >= SWIPE_VELOCITY_THRESHOLD_DP_PER_SEC {
                let direction = swipe_direction_from(s.last_velocity);
                self.pending.push_back((
                    Gesture::Swipe {
                        direction,
                        velocity: speed,
                    },
                    pos,
                ));
            }
        } else if !s.long_press_fired
            && s.max_movement_dp < MOVEMENT_THRESHOLD_DP
            && duration < TAP_DURATION_MS
        {
            self.pending
                .push_back((Gesture::Tap { pos: s.down_pos }, s.down_pos));
        }
        // 200-499ms hold without movement and no fired LongPress: dead zone, no gesture.
    }

    pub fn finger_cancel(&mut self, id: FingerId) {
        self.fingers.remove(&id);
    }

    /// Per-frame tick. Emits `LongPress` for any finger held past the threshold.
    pub fn tick(&mut self, now_ms: u64) {
        for s in self.fingers.values_mut() {
            if !s.long_press_fired
                && !s.is_panning
                && s.max_movement_dp < MOVEMENT_THRESHOLD_DP
                && now_ms.saturating_sub(s.down_time_ms) >= LONG_PRESS_DURATION_MS
            {
                self.pending
                    .push_back((Gesture::LongPress { pos: s.down_pos }, s.down_pos));
                s.long_press_fired = true;
            }
        }
    }

    /// Dispatch all pending gestures via reverse-Z hit testing.
    /// Returns the count of gestures that landed on a widget.
    pub fn flush(&mut self, arena: &mut WidgetArena) -> usize {
        let mut dispatched = 0usize;
        while let Some((gesture, pos)) = self.pending.pop_front() {
            let candidate = arena
                .iter_z_reverse()
                .find(|&id| arena.get(id).is_some_and(|c| c.hit_test(pos)));
            if let Some(id) = candidate {
                if let Some(c) = arena.get_mut(id) {
                    let _consumed = c.on_gesture(gesture);
                    dispatched += 1;
                }
            }
        }
        dispatched
    }

    pub fn pending(&self) -> impl Iterator<Item = &(Gesture, Vec2)> + '_ {
        self.pending.iter()
    }

    pub fn finger_count(&self) -> usize {
        self.fingers.len()
    }
}

fn swipe_direction_from(v: Vec2) -> SwipeDirection {
    if v.x.abs() >= v.y.abs() {
        if v.x >= 0.0 {
            SwipeDirection::Right
        } else {
            SwipeDirection::Left
        }
    } else if v.y >= 0.0 {
        SwipeDirection::Down
    } else {
        SwipeDirection::Up
    }
}
