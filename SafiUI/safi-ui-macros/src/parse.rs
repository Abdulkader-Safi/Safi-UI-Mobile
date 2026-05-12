use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

pub(crate) struct Element {
    pub tag: String,
    pub tag_span: Span,
    pub attrs: Vec<Attr>,
    pub body: Body,
}

pub(crate) struct Attr {
    pub name: String,
    pub name_span: Span,
    pub value: String,
}

pub(crate) enum Body {
    Empty,
    Text(String),
    Children(Vec<Element>),
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;

        if input.peek(Token![/]) {
            return Err(syn::Error::new(
                input.span(),
                "unexpected closing tag (expected an element)",
            ));
        }

        let tag_ident: Ident = input.parse()?;
        let tag = tag_ident.to_string();
        let tag_span = tag_ident.span();

        let mut attrs = Vec::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let name_ident: Ident = input.parse()?;
            let name_span = name_ident.span();
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            attrs.push(Attr {
                name: name_ident.to_string(),
                name_span,
                value: value.value(),
            });
        }

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(Element {
                tag,
                tag_span,
                attrs,
                body: Body::Empty,
            });
        }

        input.parse::<Token![>]>()?;

        let body = parse_body(input)?;

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close_ident: Ident = input.parse()?;
        if close_ident != tag_ident {
            return Err(syn::Error::new(
                close_ident.span(),
                format!("closing tag `</{close_ident}>` does not match opening tag `<{tag}>`"),
            ));
        }
        input.parse::<Token![>]>()?;

        Ok(Element {
            tag,
            tag_span,
            attrs,
            body,
        })
    }
}

fn parse_body(input: ParseStream) -> syn::Result<Body> {
    if input.peek(Token![<]) && input.peek2(Token![/]) {
        return Ok(Body::Empty);
    }

    if input.peek(LitStr) {
        let text: LitStr = input.parse()?;
        if input.peek(Token![<]) && !input.peek2(Token![/]) {
            return Err(syn::Error::new(
                input.span(),
                "vnode! body must be either a single string literal or one or more child elements, not both",
            ));
        }
        return Ok(Body::Text(text.value()));
    }

    let mut children = Vec::new();
    while input.peek(Token![<]) && !input.peek2(Token![/]) {
        children.push(input.parse::<Element>()?);
        if input.peek(LitStr) {
            return Err(syn::Error::new(
                input.span(),
                "vnode! body must be either a single string literal or one or more child elements, not both",
            ));
        }
    }
    Ok(Body::Children(children))
}
