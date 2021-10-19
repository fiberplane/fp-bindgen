use std::collections::VecDeque;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{
    punctuated::Punctuated, GenericArgument, Generics, Item, ItemUse, Path, PathArguments,
    PathSegment, Type,
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

/// Returns the name of the type based on its path. The result of this should be the same as
/// provided by `Type::name()`, but without the need for constructing an intermediate `Type`.
///
/// If the returned name does *not* match the one returned by `Type::name()` we use that as an
/// indication there's either an alias or a generic argument present in the type, because
/// those show up in the path, but neither aliases nor the specialized types can be known to
/// the `Type` at the implementation site.
pub(crate) fn get_name_from_path(path: &Path) -> String {
    path.segments
        .last()
        .map(|segment| match &segment.arguments {
            PathArguments::None => segment.ident.to_string(),
            PathArguments::AngleBracketed(bracketed) => format!(
                "{}<{}>",
                segment.ident,
                bracketed
                    .args
                    .iter()
                    .map(|arg| match arg {
                        GenericArgument::Type(Type::Path(path)) if path.qself.is_none() =>
                            get_name_from_path(&path.path),
                        _ => panic!("Unsupported generic argument in path: {:?}", path),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PathArguments::Parenthesized(_) => {
                panic!("Unsupported arguments in path: {:?}", path)
            }
        })
        .unwrap_or_else(String::new)
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
