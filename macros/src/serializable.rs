use crate::utils::{extract_path_from_type, parse_type_item};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use syn::punctuated::Punctuated;
use syn::{Path, TypeParamBound};

pub(crate) fn impl_derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let (item_name, item, mut generics) = parse_type_item(item);

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

    // Remove any bounds from the generic types and store them separately.
    // Otherwise, collect_types will be called like `Foo::<T: MyTrait>::collect_types()` and where clauses
    // will be incorrect, too.
    let mut bounds = HashMap::new();
    for param in generics.type_params_mut() {
        // For every parameter we want to either extract the existing trait bounds, or, if there
        // were no existing bounds, we will mark the parameter as having no bounds.

        param.bounds = if param.bounds.is_empty() {
            // No existing bound found, so mark this parameter as having 'None' as a bound
            bounds.insert(param.ident.clone(), None);
            Punctuated::new()
        } else {
            param
                .clone()
                .bounds
                .into_iter()
                .filter(|bound| {
                    match &bound {
                        TypeParamBound::Trait(tr) => {
                            // Extract the bound and remove it from the param
                            bounds.insert(param.ident.clone(), Some(tr.clone()));
                            false
                        }
                        // Ignore other bound types (lifetimes) for now
                        _ => true,
                    }
                })
                .collect()
        };
    }

    let where_clause = if bounds.is_empty() {
        quote! {}
    } else {
        let params = bounds.keys();

        // Add the appropriate bounds to the where clause
        // If no existing bounds were present, we will add the 'Serializable' bound.
        let param_bounds = bounds.values().map(|bound| match bound {
            Some(bound) => quote! { : #bound },
            None => quote! { : Serializable },
        });
        quote! {
            where
                #( #params#param_bounds ),*
        }
    };

    let collect_types = if field_types.is_empty() {
        quote! { types.entry(Self::ident()).or_insert_with(Self::ty); }
    } else {
        let field_types = field_types.iter();
        let generic_params = generics.type_params();
        quote! {
            if let std::collections::btree_map::Entry::Vacant(entry) = types.entry(Self::ident()) {
                entry.insert(Self::ty());
                #( #field_types::collect_types(types); )*
            }

            #( #generic_params::collect_types(types); )*
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
