use crate::utils::{extract_path_from_type, get_name_from_path, parse_type_item};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::Path;

pub(crate) fn impl_derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let (item_name, item, generics) = parse_type_item(item);

    let field_types: HashSet<Path> = match item {
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
            .collect(),
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
            .collect(),
        _ => HashSet::default(),
    };

    let collect_dependencies = match field_types.len() {
        0 => quote! { std::collections::BTreeSet::new() },
        1 => {
            let field_type = field_types.iter().next().unwrap();
            let name = get_name_from_path(field_type);
            quote! {
                #field_type::named_type_with_dependencies_and_generics(
                    #name,
                    generics,
                )
            }
        }
        _ => {
            let field_types = field_types.iter();
            let names = field_types.clone().map(get_name_from_path);
            quote! {
                let mut dependencies = std::collections::BTreeSet::new();
                #( dependencies.append(&mut #field_types::named_type_with_dependencies_and_generics(
                    #names,
                    generics,
                )); )*
                dependencies
            }
        }
    };

    let generics_str = generics.to_token_stream().to_string();

    let name = if generics.params.is_empty() {
        let item_name = item_name.to_string();
        quote! { #item_name.to_owned() }
    } else {
        quote! { Self::ty().name() }
    };

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
                #name
            }

            fn build_ty() -> fp_bindgen::prelude::Type {
                fp_bindgen::prelude::Type::from_item(#item_str, &Self::dependencies())
            }

            fn build_dependencies() -> std::collections::BTreeSet<fp_bindgen::prelude::Type> {
                let generics = #generics_str;
                #collect_dependencies
            }
        }
    };
    implementation.into()
}
