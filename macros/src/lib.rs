mod primitives;

use crate::primitives::Primitive;
use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::collections::BTreeMap;
use syn::{ForeignItemFn, Ident, Item};

/// Used to annotate types (`enum`s and `struct`s) that can be passed across the Wasm bridge.
#[proc_macro_derive(Serializable)]
pub fn derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let (item_name, _item) = parse_type_item(item);
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

fn parse_type_item(item: TokenStream) -> (Ident, Item) {
    let item = syn::parse::<Item>(item).unwrap();
    match item {
        Item::Enum(item) => (item.ident.clone(), Item::Enum(item)),
        Item::Struct(item) => (item.ident.clone(), Item::Struct(item)),
        item => panic!(
            "Only struct and enum types can be constructed from an item. Found: {:?}",
            item
        ),
    }
}

/// Declares functions the plugin can import from the host runtime.
#[proc_macro]
pub fn fp_import(token_stream: TokenStream) -> TokenStream {
    let functions = parse_functions(token_stream);
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
pub fn fp_export(token_stream: TokenStream) -> TokenStream {
    let functions = parse_functions(token_stream);
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

fn parse_functions(token_stream: TokenStream) -> BTreeMap<String, String> {
    let mut functions = BTreeMap::new();
    let mut current_item_tokens = Vec::<TokenTree>::new();
    for token in token_stream.into_iter() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                current_item_tokens.push(TokenTree::Punct(punct));

                let stream = current_item_tokens.into_iter().collect::<TokenStream>();
                let function = syn::parse::<ForeignItemFn>(stream).unwrap();
                functions.insert(
                    function.sig.ident.to_string(),
                    function.into_token_stream().to_string(),
                );
                current_item_tokens = Vec::new();
            }
            other => current_item_tokens.push(other),
        }
    }
    functions
}

#[doc(hidden)]
#[proc_macro]
pub fn primitive_impls(_: TokenStream) -> TokenStream {
    let primitives = [
        Primitive::Bool,
        Primitive::F32,
        Primitive::F64,
        Primitive::I8,
        Primitive::I16,
        Primitive::I32,
        Primitive::I64,
        Primitive::I128,
        Primitive::Str,
        Primitive::String,
        Primitive::U8,
        Primitive::U16,
        Primitive::U32,
        Primitive::U64,
        Primitive::U128,
        Primitive::Unit,
    ];

    let mut token_stream = TokenStream::new();
    for primitive in primitives {
        token_stream.extend(primitive.gen_impl().into_iter());
    }
    token_stream
}
