//! Virtual DOM node and its building blocks.
//!
//! See PRD §6.1.

use std::collections::HashMap;

pub type Props = HashMap<String, String>;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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
