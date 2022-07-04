use std::collections::VecDeque;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{
    punctuated::Punctuated, Generics, Item, ItemUse, Path, PathArguments, PathSegment, ReturnType,
    Type,
};

pub(crate) fn extract_path_from_type(ty: &Type) -> Option<Path> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let mut path = path.path.clone();
            for segment in &mut path.segments {
                if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    // When calling a static function on `Vec<T>`, it gets invoked
                    // as `Vec::<T>::some_func()`, so we need to insert extra colons
                    // to make the famed Rust "turbofish":
                    args.colon2_token = Some(syn::parse_quote!(::));
                }
            }
            Some(path)
        }
        _ => None,
    }
}

pub(crate) fn parse_type_item(item: TokenStream) -> (Ident, Item, Generics) {
    let item = syn::parse::<Item>(item).unwrap();
    match item {
        Item::Enum(item) => {
            let generics = item.generics.clone();
            (item.ident.clone(), Item::Enum(item), generics)
        }
        Item::Struct(item) => {
            let generics = item.generics.clone();
            (item.ident.clone(), Item::Struct(item), generics)
        }
        item => panic!(
            "Only struct and enum types can be constructed from an item. Found: {:?}",
            item
        ),
    }
}

/// Use statements are well complicated...
/// Essentially you can have some rather absurd nested ones like: `use foobar::{bar::{A,B}, baz::{C,D}};`
/// This function takes that mess and returns an iterator of [foobar::bar::A, foobar::bar::B, foobar::baz::C, foobar::baz::D]
pub(crate) fn flatten_using_statement(using: ItemUse) -> impl Iterator<Item = Path> {
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    let mut path_segments = VecDeque::<PathSegment>::new();
    queue.push_back(using.tree);

    while let Some(tree) = queue.pop_back() {
        match tree {
            syn::UseTree::Name(name) => {
                let mut segments = Punctuated::new();
                for ps in &path_segments {
                    segments.push(ps.clone());
                }
                segments.push(name.ident.into());

                result.push(Path {
                    leading_colon: None,
                    segments,
                });
            }
            syn::UseTree::Path(path) => {
                path_segments.push_back(PathSegment::from(path.ident));
                queue.push_back(*path.tree);
            }
            syn::UseTree::Group(group) => {
                for item in group.items {
                    queue.push_back(item);
                }
            }
            _ => panic!("Glob and renames use statements are not supported at this time, sorry..."),
        }
    }

    result.into_iter()
}

pub(crate) fn normalize_return_type(ty: &ReturnType) -> Option<&Type> {
    match ty {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => {
            match ty.as_ref() {
                Type::Tuple(tuple) if tuple.elems.is_empty() => {
                    /* An empty '-> ()' return value */
                    None
                }
                r => Some(r),
            }
        }
    }
}
