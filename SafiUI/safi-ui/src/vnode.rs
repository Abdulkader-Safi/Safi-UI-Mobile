//! Virtual DOM node and its building blocks.
//!
//! See PRD §6.1.

use std::collections::HashMap;

use glam::Vec2;

pub type Props = HashMap<String, String>;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    /// Half-open inclusion: `x ≤ p.x < x + width` and `y ≤ p.y < y + height`.
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.x < self.x + self.width && p.y >= self.y && p.y < self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VNode {
    pub tag: String,
    pub props: Props,
    pub children: Vec<VNode>,
    pub text_content: Option<String>,
    pub layout: LayoutRect,
    pub id: Option<String>,
    pub key: Option<String>,
}
