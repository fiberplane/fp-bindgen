use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::Path;

use crate::utils::{extract_path_from_type, get_alias_from_path, parse_type_item};

pub(crate) fn impl_derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let (item_name, item, generics) = parse_type_item(item);
    let item_name_str = item_name.to_string();

    let field_types : Vec<Path> = match item {
        syn::Item::Enum(ty) => ty
            .variants
            .into_iter()
            .flat_map(|variant| variant.fields)
            .map(|field| {
                extract_path_from_type(&field.ty).unwrap_or_else(|| {
                    panic!(
                        "Only value types are supported. Incompatible type in enum variant field: {:?}",
                        field
                    )
                })
            })
            .collect::<HashSet<_>>(),
        syn::Item::Struct(ty) => ty
            .fields
            .into_iter()
            .map(|field| {
                extract_path_from_type(&field.ty).unwrap_or_else(|| {
                    panic!(
                        "Only value types are supported. Incompatible type in struct field: {:?}",
                        field
                    )
                })
            })
            .collect::<HashSet<_>>(),
        _ => HashSet::default(),
    }.into_iter().collect();

    // Aliases cannot be derived, but we can detect their presence by comparing
    // the paths of dependencies with their expected names:
    let aliases = field_types.iter().map(get_alias_from_path);

    let where_clause = if generics.params.is_empty() {
        quote! {}
    } else {
        let params = generics.type_params();
        quote! {
            where
                #( #params: Serializable ),*
        }
    };

    let implementation = quote! {
        impl#generics fp_bindgen::prelude::Serializable for #item_name#generics#where_clause {
            fn name() -> String {
                #item_name_str.to_owned()
            }

            fn ty() -> fp_bindgen::prelude::Type {
                fp_bindgen::prelude::Type::from_item(#item_str, &Self::dependencies())
            }

            fn dependencies() -> std::collections::BTreeSet<fp_bindgen::prelude::Type> {
                let mut dependencies = std::collections::BTreeSet::new();
                #( #field_types::add_type_with_dependencies_and_alias(&mut dependencies, #aliases); )*
                dependencies
            }
        }
    };
    implementation.into()
}
