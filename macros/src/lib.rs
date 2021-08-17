mod primitives;

use crate::primitives::Primitive;
use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::collections::{BTreeMap, HashSet};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, FnArg, ForeignItemFn, Ident, Item, Path, Result, ReturnType, Token, Type,
};

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

            fn ty() -> Type {
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
    let (functions, serializable_types, deserializable_types) = parse_functions(token_stream);
    let function_name = functions.keys();
    let function_decl = functions.values();
    let serializable_types = serializable_types.iter();
    let deserializable_types = deserializable_types.iter();
    let replacement = quote! {
        fn __fp_declare_import_fns() -> (fp_bindgen::prelude::FunctionMap, std::collections::BTreeSet<Type>, std::collections::BTreeSet<Type>) {
            let mut map = fp_bindgen::prelude::FunctionMap::new();
            #( map.insert_str(#function_name, #function_decl); )*

            let mut serializable_import_types = std::collections::BTreeSet::new();
            #( add_type_recursively!(serializable_import_types, #serializable_types); )*

            let mut deserializable_import_types = std::collections::BTreeSet::new();
            #( add_type_recursively!(deserializable_import_types, #deserializable_types); )*

            (map, serializable_import_types, deserializable_import_types)
        }
    };
    replacement.into()
}

/// Declares functions the plugin may export to the host runtime.
#[proc_macro]
pub fn fp_export(token_stream: TokenStream) -> TokenStream {
    let (functions, serializable_types, deserializable_types) = parse_functions(token_stream);
    let function_name = functions.keys();
    let function_decl = functions.values();
    let serializable_types = serializable_types.iter();
    let deserializable_types = deserializable_types.iter();
    let replacement = quote! {
        fn __fp_declare_export_fns() -> (fp_bindgen::prelude::FunctionMap, std::collections::BTreeSet<Type>, std::collections::BTreeSet<Type>) {
            let mut map = fp_bindgen::prelude::FunctionMap::new();
            #( map.insert_str(#function_name, #function_decl); )*

            let mut serializable_export_types = std::collections::BTreeSet::new();
            #( add_type_recursively!(serializable_export_types, #serializable_types); )*

            let mut deserializable_export_types = std::collections::BTreeSet::new();
            #( add_type_recursively!(deserializable_export_types, #deserializable_types); )*

            (map, serializable_export_types, deserializable_export_types)
        }
    };
    replacement.into()
}

struct AddTypeArgs {
    type_set: Ident,
    _comma_token: Token![,],
    type_path: Path,
}

impl Parse for AddTypeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            type_set: input.parse()?,
            _comma_token: input.parse()?,
            type_path: input.parse()?,
        })
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn add_type_recursively(token_stream: TokenStream) -> TokenStream {
    let args: AddTypeArgs = parse_macro_input!(token_stream);
    let type_path = args.type_path;
    let type_set = args.type_set;

    let implementation = quote!({
        type Type = #type_path;
        if !Type::is_primitive() {
            #type_set.insert(Type::ty());
            for dependency in Type::dependencies() {
                // FIXME: This isn't really recursive and only adds dependencies one level deep.
                #type_set.insert(dependency);
            }
        }
    });
    implementation.into()
}

/// Parses function declarations and returns a map with all functions keyed by name.
/// In addition, it returns two sets: one with all the names for types that may need serialization
/// to call the functions, and one with all the names for types that may need deserialization to
/// call the functions.
fn parse_functions(
    token_stream: TokenStream,
) -> (BTreeMap<String, String>, HashSet<Path>, HashSet<Path>) {
    let mut functions = BTreeMap::new();
    let mut serializable_type_names = HashSet::new();
    let mut deserializable_type_names = HashSet::new();
    let mut current_item_tokens = Vec::<TokenTree>::new();
    for token in token_stream.into_iter() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                current_item_tokens.push(TokenTree::Punct(punct));

                let stream = current_item_tokens.into_iter().collect::<TokenStream>();
                let function = syn::parse::<ForeignItemFn>(stream).unwrap();

                for input in &function.sig.inputs {
                    match input {
                        FnArg::Receiver(_) => panic!(
                            "Methods are not supported. Found `self` in function declaration: {:?}",
                            function.sig
                        ),
                        FnArg::Typed(arg) => match arg.ty.as_ref() {
                            Type::Path(path) if path.qself.is_none() => {
                                serializable_type_names.insert(path.path.clone());
                            }
                            _ => panic!(
                                "Only plain value types are supported. \
                                        Incompatible argument type in function declaration: {:?}",
                                function.sig
                            ),
                        },
                    }
                }

                match &function.sig.output {
                    ReturnType::Default => { /* No return value. */ }
                    ReturnType::Type(_, ty) => match ty.as_ref() {
                        Type::Path(path) if path.qself.is_none() => {
                            deserializable_type_names.insert(path.path.clone());
                        }
                        _ => panic!(
                            "Only plain value types are supported. \
                                    Incompatible return type in function declaration: {:?}",
                            function.sig
                        ),
                    },
                }

                functions.insert(
                    function.sig.ident.to_string(),
                    function.into_token_stream().to_string(),
                );

                current_item_tokens = Vec::new();
            }
            other => current_item_tokens.push(other),
        }
    }

    (
        functions,
        serializable_type_names,
        deserializable_type_names,
    )
}

/// Generates bindings for the functions declared in the `fp_import!{}` and `fp_export!{}` blocks.
#[proc_macro]
pub fn fp_bindgen(args: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let replacement = quote! {
        let (import_functions, serializable_import_types, deserializable_import_types) =
            __fp_declare_import_fns();
        let (export_functions, mut serializable_export_types, mut deserializable_export_types) =
            __fp_declare_export_fns();

        let mut serializable_types = serializable_import_types;
        serializable_types.append(&mut deserializable_export_types);

        let mut deserializable_types = deserializable_import_types;
        deserializable_types.append(&mut serializable_export_types);

        fp_bindgen::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            #args
        );
    };
    replacement.into()
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
        Primitive::String,
        Primitive::U8,
        Primitive::U16,
        Primitive::U32,
        Primitive::U64,
        Primitive::U128,
    ];

    let mut token_stream = TokenStream::new();
    for primitive in primitives {
        token_stream.extend(primitive.gen_impl().into_iter());
    }
    token_stream
}
