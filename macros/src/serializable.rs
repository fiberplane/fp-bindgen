use crate::utils::{extract_path_from_type, parse_type_item};
use proc_macro::TokenStream;
use quote::quote;
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

    let collect_types = if field_types.is_empty() {
        quote! { types.entry(Self::ident()).or_insert_with(Self::ty); }
    } else {
        let field_types = field_types.iter();
        quote! {
            if let std::collections::btree_map::Entry::Vacant(entry) = types.entry(Self::ident()) {
                entry.insert(Self::ty());
                #( #field_types::collect_types(types); )*
            }
        }
    };

    let ident = {
        let item_name = item_name.to_string();
        if generics.params.is_empty() {
            quote! { fp_bindgen::prelude::TypeIdent::from(#item_name) }
        } else {
            let params = generics.type_params().map(|param| param.ident.to_string());
            quote! {
                fp_bindgen::prelude::TypeIdent {
                    name: #item_name.to_owned(),
                    generic_args: vec![#( fp_bindgen::prelude::TypeIdent::from(#params) ),*],
                }
            }
        }
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
            fn ident() -> fp_bindgen::prelude::TypeIdent {
                #ident
            }

            fn ty() -> fp_bindgen::prelude::Type {
                fp_bindgen::prelude::Type::from_item(#item_str)
            }

            fn collect_types(types: &mut fp_bindgen::prelude::TypeMap) {
                #collect_types
            }
        }
    };
    implementation.into()
}
