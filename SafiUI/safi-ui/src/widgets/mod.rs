//! Built-in widget library (todo 15+).
//!
//! Three widgets land in this todo (15): `View`, `Text`, `Button` —
//! the minimum needed to render the example app and prove the
//! end-to-end pipeline (XML → parse → registry → build → render).
//! Layout/Typography/Input/Display/Navigation/Data widgets per PRD
//! §10 fill out in todos 18+.
//!
//! ## Tags registered
//!
//! `View`'s factory is registered under multiple tag names because
//! the rendering shape is identical — only the layout direction (set
//! by `LayoutEngine` from the tag) differs:
//!
//! | Tag        | Factory     | Notes                                              |
//! | ---------- | ----------- | -------------------------------------------------- |
//! | `Screen`   | [`View`]    | Root container, identical paint to `View`          |
//! | `View`     | [`View`]    | Generic box container                              |
//! | `Row`      | [`View`]    | Same paint; `LayoutEngine` sets row flex-direction |
//! | `Column`   | [`View`]    | Same paint; column flex-direction                  |
//! | `Stack`    | [`View`]    | Same paint; absolute-position children             |
//! | `Spacer`   | [`View`]    | Empty paint; `flex: 1` for stretch fill            |
//! | `Text`     | [`Text`]    |                                                    |
//! | `Heading`  | [`Text`]    | Pre-sized by `level` prop                          |
//! | `Label`    | [`Text`]    | Small uppercase form label                         |
//! | `Button`   | [`Button`]  |                                                    |

pub mod button;
pub mod text;
pub mod view;

pub use button::Button;
pub use text::Text;
pub use view::View;

use crate::registry::ComponentRegistry;

/// Register every built-in widget under its canonical XML tag(s).
///
/// Called once at app startup. Idempotent in the sense that re-running
/// re-registers (last-write-wins), but the duplicate-registration
/// warning will fire — apps should call this exactly once.
pub fn register_builtins(reg: &mut ComponentRegistry) {
    use crate::vnode::Props;

    // View — the generic box. Same factory under every layout
    // container tag because the paint is identical; `LayoutEngine`
    // pulls flex-direction from the tag name (todo 10 tag-based
    // defaults).
    for tag in ["Screen", "View", "Row", "Column", "Stack", "Spacer"] {
        reg.register(
            tag,
            Box::new(|props: &Props| -> Box<dyn crate::component::Component> {
                Box::new(View::from_props(props))
            }),
        );
    }

    // Text variants — `Heading` and `Label` reuse the Text widget
    // with different size/style defaults. Heading level math (h1=32,
    // h2=24, …) is handled inside Text::from_props by inspecting the
    // `level` prop.
    for tag in ["Text", "Heading", "Label"] {
        let owned_tag = tag.to_string();
        reg.register(
            tag,
            Box::new(
                move |props: &Props| -> Box<dyn crate::component::Component> {
                    Box::new(Text::from_props(&owned_tag, props))
                },
            ),
        );
    }

    reg.register(
        "Button",
        Box::new(|props: &Props| -> Box<dyn crate::component::Component> {
            Box::new(Button::from_props(props))
        }),
    );
}
