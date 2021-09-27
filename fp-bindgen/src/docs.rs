use syn::{AttrStyle, Attribute};

pub fn get_doc_lines(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| {
            attr.style == AttrStyle::Outer
                && attr.path.get_ident().map(|ident| ident.to_string()) == Some("doc".to_owned())
        })
        .flat_map(|attr| {
            attr.tokens
                .clone()
                .into_iter()
                .filter_map(|token| match token {
                    proc_macro2::TokenTree::Literal(literal) => parse_literal(literal),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn parse_literal(literal: proc_macro2::Literal) -> Option<String> {
    let string = literal.to_string();
    if string.starts_with('"') && string.ends_with('"') {
        Some(
            string[1..string.len() - 1]
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"),
        )
    } else {
        None
    }
}
