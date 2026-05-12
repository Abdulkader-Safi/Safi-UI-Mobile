use safi_ui::vnode;
use safi_ui::vnode::{LayoutRect, VNode};

#[test]
fn text_only_element() {
    let tree: VNode = vnode! { <Text>"hi"</Text> };
    assert_eq!(tree.tag, "Text");
    assert_eq!(tree.text_content.as_deref(), Some("hi"));
    assert!(tree.children.is_empty());
    assert!(tree.props.is_empty());
    assert_eq!(tree.id, None);
    assert_eq!(tree.key, None);
    assert_eq!(tree.layout, LayoutRect::default());
}

#[test]
fn self_closing_with_prop() {
    let tree: VNode = vnode! { <Spacer size="16" /> };
    assert_eq!(tree.tag, "Spacer");
    assert_eq!(tree.text_content, None);
    assert!(tree.children.is_empty());
    assert_eq!(tree.props.get("size").map(String::as_str), Some("16"));
}

#[test]
fn nested_children() {
    let tree: VNode = vnode! {
        <Column gap="12">
            <Text>"a"</Text>
            <Text>"b"</Text>
        </Column>
    };
    assert_eq!(tree.tag, "Column");
    assert_eq!(tree.text_content, None);
    assert_eq!(tree.props.get("gap").map(String::as_str), Some("12"));
    assert_eq!(tree.children.len(), 2);
    assert_eq!(tree.children[0].text_content.as_deref(), Some("a"));
    assert_eq!(tree.children[1].text_content.as_deref(), Some("b"));
}

#[test]
fn id_and_key_are_lifted_out_of_props() {
    let tree: VNode = vnode! {
        <Input id="email" key="row-3" placeholder="x" />
    };
    assert_eq!(tree.id.as_deref(), Some("email"));
    assert_eq!(tree.key.as_deref(), Some("row-3"));
    assert_eq!(tree.props.len(), 1);
    assert_eq!(tree.props.get("placeholder").map(String::as_str), Some("x"));
    assert!(!tree.props.contains_key("id"));
    assert!(!tree.props.contains_key("key"));
}

#[test]
fn composite_binding_in_text_preserved_verbatim() {
    let tree: VNode = vnode! { <Text>"Hello {{name}}!"</Text> };
    assert_eq!(tree.text_content.as_deref(), Some("Hello {{name}}!"));
}

#[test]
fn dynamic_event_binding_in_prop_preserved_verbatim() {
    let tree: VNode = vnode! { <Button onPress="{{action}}" /> };
    assert_eq!(
        tree.props.get("onPress").map(String::as_str),
        Some("{{action}}")
    );
}

#[test]
fn prd_example_tree() {
    let tree: VNode = vnode! {
        <Screen bg="#0f0f1a" safeArea="true">
            <Column gap="12" padding="16">
                <Heading level="2" color="#fff">"Hello"</Heading>
                <Button id="cta" label="Tap me" onPress="demo.tap" />
            </Column>
        </Screen>
    };
    assert_eq!(tree.tag, "Screen");
    assert_eq!(tree.props.get("bg").map(String::as_str), Some("#0f0f1a"));
    assert_eq!(tree.props.get("safeArea").map(String::as_str), Some("true"));

    let column = &tree.children[0];
    assert_eq!(column.tag, "Column");
    assert_eq!(column.children.len(), 2);

    let heading = &column.children[0];
    assert_eq!(heading.tag, "Heading");
    assert_eq!(heading.props.get("level").map(String::as_str), Some("2"));
    assert_eq!(heading.text_content.as_deref(), Some("Hello"));

    let button = &column.children[1];
    assert_eq!(button.tag, "Button");
    assert_eq!(button.id.as_deref(), Some("cta"));
    assert!(!button.props.contains_key("id"));
    assert_eq!(
        button.props.get("label").map(String::as_str),
        Some("Tap me")
    );
    assert_eq!(
        button.props.get("onPress").map(String::as_str),
        Some("demo.tap")
    );
}

#[test]
fn layout_rect_default_is_zero() {
    let r = LayoutRect::default();
    assert_eq!(
        r,
        LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0
        }
    );
}

#[test]
fn empty_body_open_close() {
    let tree: VNode = vnode! { <View></View> };
    assert_eq!(tree.tag, "View");
    assert!(tree.children.is_empty());
    assert_eq!(tree.text_content, None);
}

#[test]
fn unknown_tag_compiles_fine() {
    let tree: VNode = vnode! { <SomeMadeUpComponent foo="bar" /> };
    assert_eq!(tree.tag, "SomeMadeUpComponent");
    assert_eq!(tree.props.get("foo").map(String::as_str), Some("bar"));
}
