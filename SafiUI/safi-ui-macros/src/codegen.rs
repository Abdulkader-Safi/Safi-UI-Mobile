use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{Body, Element};

pub(crate) fn emit(element: &Element) -> TokenStream {
    let tag = &element.tag;

    let mut id_value: Option<&str> = None;
    let mut key_value: Option<&str> = None;
    let mut prop_inserts: Vec<TokenStream> = Vec::new();
    let mut seen_id = false;
    let mut seen_key = false;

    for attr in &element.attrs {
        match attr.name.as_str() {
            "id" => {
                if seen_id {
                    return syn::Error::new(attr.name_span, "duplicate `id` attribute")
                        .to_compile_error();
                }
                seen_id = true;
                id_value = Some(&attr.value);
            }
            "key" => {
                if seen_key {
                    return syn::Error::new(attr.name_span, "duplicate `key` attribute")
                        .to_compile_error();
                }
                seen_key = true;
                key_value = Some(&attr.value);
            }
            _ => {
                let name = &attr.name;
                let value = &attr.value;
                prop_inserts.push(quote! {
                    __props.insert(
                        ::std::string::String::from(#name),
                        ::std::string::String::from(#value),
                    );
                });
            }
        }
    }

    let prop_capacity = prop_inserts.len();

    let id_expr = if let Some(v) = id_value {
        quote! { ::std::option::Option::Some(::std::string::String::from(#v)) }
    } else {
        quote! { ::std::option::Option::None }
    };
    let key_expr = if let Some(v) = key_value {
        quote! { ::std::option::Option::Some(::std::string::String::from(#v)) }
    } else {
        quote! { ::std::option::Option::None }
    };

    let (text_expr, children_expr) = match &element.body {
        Body::Empty => (
            quote! { ::std::option::Option::None },
            quote! { ::std::vec::Vec::new() },
        ),
        Body::Text(s) => (
            quote! { ::std::option::Option::Some(::std::string::String::from(#s)) },
            quote! { ::std::vec::Vec::new() },
        ),
        Body::Children(children) => {
            let child_exprs: Vec<TokenStream> = children.iter().map(emit).collect();
            (
                quote! { ::std::option::Option::None },
                quote! { ::std::vec![ #( #child_exprs ),* ] },
            )
        }
    };

    quote! {
        {
            let mut __props: ::std::collections::HashMap<
                ::std::string::String,
                ::std::string::String,
            > = ::std::collections::HashMap::with_capacity(#prop_capacity);
            #( #prop_inserts )*
            ::safi_ui::vnode::VNode {
                tag: ::std::string::String::from(#tag),
                props: __props,
                children: #children_expr,
                text_content: #text_expr,
                layout: ::safi_ui::vnode::LayoutRect::default(),
                id: #id_expr,
                key: #key_expr,
            }
        }
    }
}
