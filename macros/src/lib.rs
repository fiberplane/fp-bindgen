use fp_bindgen_common::{DataStructureItem, FunctionMap};
use proc_macro::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{Ident, Item};

#[proc_macro_derive(Deserializable)]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
    let item = DataStructureItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();
    let item_name = syn::parse_str::<Ident>(&item.name()).unwrap();
    let item_name_str = item_name.to_string();

    let implementation = quote! {
        impl Deserializable for #item_name {
            fn name() -> String {
                #item_name_str.to_owned()
            }
        }
    };
    implementation.into()
}

#[proc_macro_derive(Serializable)]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let item = DataStructureItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();
    let item_name = syn::parse_str::<Ident>(&item.name()).unwrap();
    let item_name_str = item_name.to_string();

    let implementation = quote! {
        impl Serializable for #item_name {
            fn name() -> String {
                #item_name_str.to_owned()
            }
        }
    };
    implementation.into()
}

#[proc_macro]
pub fn fp_import(block: TokenStream) -> TokenStream {
    let functions = FunctionMap::from_stream(block.into());
    let function_name = functions.keys();
    let function_decl = functions.values();
    let replacement = quote! {
        fn __fp_declare_import_fns() -> FunctionMap {
            let mut map = FunctionMap::new();
            #( map.insert_str(#function_name, #function_decl); )*
            map
        }
    };
    replacement.into()
}

#[proc_macro]
pub fn fp_export(block: TokenStream) -> TokenStream {
    let functions = FunctionMap::from_stream(block.into());
    let function_name = functions.keys();
    let function_decl = functions.values();
    let replacement = quote! {
        fn __fp_declare_export_fns() -> FunctionMap {
            let mut map = FunctionMap::new();
            #( map.insert_str(#function_name, #function_decl); )*
            map
        }
    };
    replacement.into()
}

#[proc_macro]
pub fn fp_bindgen(args: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let replacement = quote! {
        let import_functions = __fp_declare_import_fns();
        let export_functions = __fp_declare_export_fns();

        fp_bindgen::generate_bindings(import_functions, export_functions, #args);
    };
    replacement.into()
}
