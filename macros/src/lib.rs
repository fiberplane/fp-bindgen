use fp_bindgen_common::{FunctionMap, Primitive, Type};
use proc_macro::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{Ident, Item};

/// Used to annotate types (`enum`s and `struct`s) that can be passed across the Wasm bridge.
#[proc_macro_derive(Serializable)]
pub fn derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let item = Type::try_from(syn::parse::<Item>(item).unwrap()).unwrap();
    let item_name = syn::parse_str::<Ident>(&item.name()).unwrap();
    let item_name_str = item_name.to_string();

    let implementation = quote! {
        impl Serializable for #item_name {
            fn name() -> String {
                #item_name_str.to_owned()
            }

            fn item() -> Type {
                use std::str::FromStr;
                fp_bindgen::prelude::Type::from_str(#item_str).unwrap()
            }

            fn is_primitive() -> bool {
                false
            }

            fn dependencies() -> Vec<fp_bindgen::prelude::Type> {
                // TODO
                vec![]
            }
        }
    };
    implementation.into()
}

/// Declares functions the plugin can import from the host runtime.
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

/// Declares functions the plugin may export to the host runtime.
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

/// Generates bindings for the functions declared in the `fp_import!{}` and `fp_export!{}` blocks.
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

#[doc(hidden)]
#[proc_macro]
pub fn primitive_impls(_: TokenStream) -> TokenStream {
    let mut token_stream = proc_macro2::TokenStream::new();
    token_stream.extend(Primitive::Bool.gen_impl().into_iter());
    token_stream.into()
}
