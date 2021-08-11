use fp_bindgen_common::{DataStructureItem, FunctionMap};
use proc_macro::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{Ident, Item};

mod generator;
mod generators;

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(item: TokenStream) -> TokenStream {
    let item = DataStructureItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();
    let item_name = syn::parse_str::<Ident>(&item.name()).unwrap();

    let implementation = quote! {
        impl Deserialize for #item_name {}
    };
    implementation.into()
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(item: TokenStream) -> TokenStream {
    let item = DataStructureItem::try_from(syn::parse::<Item>(item).unwrap()).unwrap();
    let item_name = syn::parse_str::<Ident>(&item.name()).unwrap();

    let implementation = quote! {
        impl Serialize for #item_name {}
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
pub fn fp_bindgen(_args: TokenStream) -> TokenStream {
    let replacement = quote! {
        let import_functions = __fp_declare_import_fns();
        println!("Import: {:?}", import_functions);

        let export_functions = __fp_declare_export_fns();
        println!("Export: {:?}", export_functions);
    };
    replacement.into()
}
