//! XML → `VNode` parser (PRD §6.5, §12).
//!
//! Wraps [`roxmltree`] and produces the same `VNode` shape the `vnode!` macro
//! emits, so a screen authored as XML and one authored programmatically are
//! interchangeable. All prop values stay as `String` (PRD §6.1); `id` and
//! `key` are lifted to dedicated `VNode` fields and are **not** retained in
//! `props` (matches `safi-ui-macros::codegen`).

use std::fs;
use std::path::Path;

use roxmltree::{Document, Node, NodeType, TextPos};

use crate::vnode::{LayoutRect, VNode};

/// Recommended max source size from PRD §6.5.
const SIZE_WARN_BYTES: usize = 512 * 1024;
/// Performance-warning threshold from PRD §12.1.
const DEPTH_WARN: usize = 20;

/// Zero-sized handle. Both entry points are associated functions.
pub struct XmlParser;

impl XmlParser {
    /// Read `path` as UTF-8 and parse it into a `VNode` tree.
    pub fn parse_path(path: &Path) -> Result<VNode, ParseError> {
        let source_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<input>")
            .to_string();

        let bytes = fs::read(path).map_err(|e| ParseError {
            source_name: source_name.clone(),
            line: 0,
            column: 0,
            message: format!("io error: {e}"),
        })?;

        if bytes.len() > SIZE_WARN_BYTES {
            eprintln!(
                "safi-ui::parse: {source_name} is {} KB, recommended < 512 KB per screen (PRD §6.5)",
                bytes.len() / 1024
            );
        }

        let input = std::str::from_utf8(&bytes).map_err(|_| ParseError {
            source_name: source_name.clone(),
            line: 0,
            column: 0,
            message: "input is not valid UTF-8".into(),
        })?;

        parse_inner(input, &source_name)
    }

    /// Parse `input` directly. `source_name` is used in error messages
    /// (e.g. a file name or `"<inline>"`).
    pub fn parse_str(input: &str, source_name: &str) -> Result<VNode, ParseError> {
        if input.len() > SIZE_WARN_BYTES {
            eprintln!(
                "safi-ui::parse: {source_name} is {} KB, recommended < 512 KB per screen (PRD §6.5)",
                input.len() / 1024
            );
        }
        parse_inner(input, source_name)
    }
}

fn parse_inner(input: &str, source_name: &str) -> Result<VNode, ParseError> {
    let doc = Document::parse(input).map_err(|e| {
        let pos = e.pos();
        ParseError {
            source_name: source_name.to_string(),
            line: pos.row,
            column: pos.col,
            message: e.to_string(),
        }
    })?;

    let mut depth_warned = false;
    build_vnode(doc.root_element(), &doc, source_name, 1, &mut depth_warned)
}

fn build_vnode(
    node: Node<'_, '_>,
    doc: &Document<'_>,
    source_name: &str,
    depth: usize,
    depth_warned: &mut bool,
) -> Result<VNode, ParseError> {
    if depth > DEPTH_WARN && !*depth_warned {
        eprintln!(
            "safi-ui::parse: {source_name} nests {depth} levels deep, performance warning (PRD §12.1)"
        );
        *depth_warned = true;
    }

    let tag = node.tag_name().name().to_string();

    let mut props = crate::vnode::Props::new();
    let mut id: Option<String> = None;
    let mut key: Option<String> = None;

    for attr in node.attributes() {
        let name = attr.name();
        let value = attr.value().to_string();
        match name {
            "id" => id = Some(value),
            "key" => key = Some(value),
            _ => {
                props.insert(name.to_string(), value);
            }
        }
    }

    let mut children = Vec::new();
    let mut text_content: Option<String> = None;
    let mut has_element_child = false;

    for child in node.children() {
        match child.node_type() {
            NodeType::Element => {
                has_element_child = true;
                children.push(build_vnode(
                    child,
                    doc,
                    source_name,
                    depth + 1,
                    depth_warned,
                )?);
            }
            NodeType::Text => {
                let raw = child.text().unwrap_or("");
                if raw.trim().is_empty() {
                    continue;
                }
                if has_element_child || !children.is_empty() {
                    let pos = doc.text_pos_at(child.range().start);
                    return Err(ParseError {
                        source_name: source_name.to_string(),
                        line: pos.row,
                        column: pos.col,
                        message: "mixed text + element content is not supported".into(),
                    });
                }
                text_content = Some(raw.to_string());
            }
            NodeType::Comment | NodeType::PI | NodeType::Root => {}
        }
    }

    if !children.is_empty() {
        if let Some(text) = &text_content {
            if !text.trim().is_empty() {
                let pos = node_text_pos(node, doc);
                return Err(ParseError {
                    source_name: source_name.to_string(),
                    line: pos.row,
                    column: pos.col,
                    message: "mixed text + element content is not supported".into(),
                });
            }
        }
    }

    Ok(VNode {
        tag,
        props,
        children,
        text_content,
        layout: LayoutRect::default(),
        id,
        key,
    })
}

fn node_text_pos(node: Node<'_, '_>, doc: &Document<'_>) -> TextPos {
    doc.text_pos_at(node.range().start)
}

/// Parser error carrying enough context for an actionable message.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub source_name: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.source_name, self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vnode::VNode;

    fn parse(input: &str) -> VNode {
        XmlParser::parse_str(input, "test.xml").expect("parse")
    }

    #[test]
    fn minimal_self_closing() {
        let v = parse("<Screen />");
        assert_eq!(v.tag, "Screen");
        assert!(v.props.is_empty());
        assert!(v.children.is_empty());
        assert!(v.text_content.is_none());
        assert!(v.id.is_none());
        assert!(v.key.is_none());
    }

    #[test]
    fn id_and_key_are_lifted_out_of_props() {
        let v = parse(r#"<Button id="cta" key="row-0" label="Tap" />"#);
        assert_eq!(v.id.as_deref(), Some("cta"));
        assert_eq!(v.key.as_deref(), Some("row-0"));
        assert!(!v.props.contains_key("id"));
        assert!(!v.props.contains_key("key"));
        assert_eq!(v.props.get("label").map(String::as_str), Some("Tap"));
    }

    #[test]
    fn comments_are_stripped() {
        let v = parse("<View><!-- hello --><Text /></View>");
        assert_eq!(v.children.len(), 1);
        assert_eq!(v.children[0].tag, "Text");
    }

    #[test]
    fn whitespace_only_text_is_skipped() {
        let v = parse("<View>\n  <Text />\n  <Text />\n</View>");
        assert_eq!(v.children.len(), 2);
        assert!(v.text_content.is_none());
    }

    #[test]
    fn leaf_text_is_preserved() {
        let v = parse("<Text>hello world</Text>");
        assert_eq!(v.text_content.as_deref(), Some("hello world"));
        assert!(v.children.is_empty());
    }

    #[test]
    fn leaf_text_preserves_inner_whitespace() {
        let v = parse("<Text>  spaced  </Text>");
        assert_eq!(v.text_content.as_deref(), Some("  spaced  "));
    }

    #[test]
    fn mixed_content_is_rejected() {
        let err = XmlParser::parse_str("<View>oops<Text/></View>", "mixed.xml").unwrap_err();
        assert!(err.message.contains("mixed"));
        assert!(err.line >= 1);
        assert_eq!(err.source_name, "mixed.xml");
    }

    #[test]
    fn nested_tree_round_trips_shape() {
        let v = parse(
            r##"<Screen bg="#000">
                 <Column gap="12">
                   <Heading level="2">Hello</Heading>
                   <Button id="cta" label="Tap" />
                 </Column>
               </Screen>"##,
        );
        assert_eq!(v.tag, "Screen");
        assert_eq!(v.props.get("bg").map(String::as_str), Some("#000"));
        assert_eq!(v.children.len(), 1);
        let col = &v.children[0];
        assert_eq!(col.tag, "Column");
        assert_eq!(col.children.len(), 2);
        assert_eq!(col.children[0].tag, "Heading");
        assert_eq!(col.children[0].text_content.as_deref(), Some("Hello"));
        assert_eq!(col.children[1].id.as_deref(), Some("cta"));
    }

    #[test]
    fn malformed_xml_carries_line_and_column() {
        let err = XmlParser::parse_str("<View>\n  <Text>\n</View>", "bad.xml").unwrap_err();
        assert!(err.line >= 1);
        assert_eq!(err.source_name, "bad.xml");
    }

    #[test]
    fn parse_path_rejects_non_utf8() {
        use std::io::Write;
        let mut path = std::env::temp_dir();
        path.push("safi-ui-parse-non-utf8.xml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&[0xff, 0xfe, 0x00, 0x3c]).unwrap();
        drop(f);
        let err = XmlParser::parse_path(&path).unwrap_err();
        assert!(err.message.contains("UTF-8"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn parse_path_reads_file() {
        use std::io::Write;
        let mut path = std::env::temp_dir();
        path.push("safi-ui-parse-ok.xml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"<Screen />").unwrap();
        drop(f);
        let v = XmlParser::parse_path(&path).unwrap();
        assert_eq!(v.tag, "Screen");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn display_format_is_source_line_column_message() {
        let err = ParseError {
            source_name: "foo.xml".into(),
            line: 3,
            column: 7,
            message: "boom".into(),
        };
        assert_eq!(err.to_string(), "foo.xml:3:7: boom");
    }
}
