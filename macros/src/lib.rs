mod primitives;

use crate::primitives::Primitive;
use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{FnArg, ForeignItemFn, Ident, Item, Path, PathArguments, ReturnType, Type};

/// Used to annotate types (`enum`s and `struct`s) that can be passed across the Wasm bridge.
#[proc_macro_derive(Serializable)]
pub fn derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let (item_name, item) = parse_type_item(item);
    let item_name_str = item_name.to_string();

    let dependencies = match item {
        syn::Item::Enum(ty) => {
            // TODO
            vec![]
        }
        syn::Item::Struct(ty) => ty
            .fields
            .into_iter()
            .map(|field| match field.ty {
                syn::Type::Path(path) if path.qself.is_none() => {
                    let mut path = path.path;
                    for segment in &mut path.segments {
                        if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                            // When calling it a static function on `Vec<T>`, it gets invoked
                            // as `Vec::<T>::some_func()`, so we need to insert extra colons:
                            args.colon2_token = Some(syn::parse_quote!(::));
                        }
                    }
                    path
                }
                _ => panic!(
                    "Only value types are supported. Incompatible type in struct field: {:?}",
                    field
                ),
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let implementation = quote! {
        impl fp_bindgen::prelude::Serializable for #item_name {
            fn name() -> String {
                #item_name_str.to_owned()
            }

            fn ty() -> Type {
                fp_bindgen::prelude::Type::from_item(#item_str, &Self::dependencies())
            }

            fn dependencies() -> std::collections::BTreeSet<fp_bindgen::prelude::Type> {
                use std::str::FromStr;
                let mut dependencies = std::collections::BTreeSet::new();
                #( #dependencies::add_type_with_dependencies(&mut dependencies); )*
                dependencies
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
    let serializable_types = serializable_types.iter();
    let deserializable_types = deserializable_types.iter();
    let replacement = quote! {
        fn __fp_declare_import_fns() -> (fp_bindgen::prelude::FunctionList, std::collections::BTreeSet<Type>, std::collections::BTreeSet<Type>) {
            let mut serializable_import_types = std::collections::BTreeSet::new();
            #( #serializable_types::add_type_with_dependencies(&mut serializable_import_types); )*

            let mut deserializable_import_types = std::collections::BTreeSet::new();
            #( #deserializable_types::add_type_with_dependencies(&mut deserializable_import_types); )*

            let mut list = fp_bindgen::prelude::FunctionList::new();
            #( list.add_function(#functions, &serializable_import_types, &deserializable_import_types); )*

            (list, serializable_import_types, deserializable_import_types)
        }
    };
    replacement.into()
}

/// Declares functions the plugin may export to the host runtime.
#[proc_macro]
pub fn fp_export(token_stream: TokenStream) -> TokenStream {
    let (functions, serializable_types, deserializable_types) = parse_functions(token_stream);
    let serializable_types = serializable_types.iter();
    let deserializable_types = deserializable_types.iter();
    let replacement = quote! {
        fn __fp_declare_export_fns() -> (fp_bindgen::prelude::FunctionList, std::collections::BTreeSet<Type>, std::collections::BTreeSet<Type>) {
            let mut serializable_export_types = std::collections::BTreeSet::new();
            #( #serializable_types::add_type_with_dependencies(&mut serializable_export_types); )*

            let mut deserializable_export_types = std::collections::BTreeSet::new();
            #( #deserializable_types::add_type_with_dependencies(&mut deserializable_export_types); )*

            let mut list = fp_bindgen::prelude::FunctionList::new();
            #( list.add_function(#functions, &serializable_export_types, &deserializable_export_types); )*

            (list, serializable_export_types, deserializable_export_types)
        }
    };
    replacement.into()
}

/// Parses function declarations and returns them in a list.
/// In addition, it returns two sets: one with all the paths for types that may need serialization
/// to call the functions, and one with all the paths for types that may need deserialization to
/// call the functions.
fn parse_functions(token_stream: TokenStream) -> (Vec<String>, HashSet<Path>, HashSet<Path>) {
    let mut functions = Vec::new();
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
                                "Only value types are supported. \
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
                            "Only value types are supported. \
                                Incompatible return type in function declaration: {:?}",
                            function.sig
                        ),
                    },
                }

                functions.push(function.into_token_stream().to_string());

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
