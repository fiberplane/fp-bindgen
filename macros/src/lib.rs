mod primitives;

use crate::primitives::Primitive;
use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    FnArg, ForeignItemFn, GenericArgument, Generics, Ident, Item, Path, PathArguments, ReturnType,
    Type,
};

/// Used to annotate types (`enum`s and `struct`s) that can be passed across the Wasm bridge.
#[proc_macro_derive(Serializable, attributes(fp))]
pub fn derive_serializable(item: TokenStream) -> TokenStream {
    let item_str = item.to_string();
    let (item_name, item, generics) = parse_type_item(item);

    let dependencies = match item {
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
            .collect::<Vec<_>>(),
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
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let names = dependencies.iter().map(get_name_from_path);

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

            fn ty() -> fp_bindgen::prelude::Type {
                fp_bindgen::prelude::Type::from_item(#item_str, &Self::dependencies())
            }

            fn dependencies() -> std::collections::BTreeSet<fp_bindgen::prelude::Type> {
                let generics = #generics_str;
                let mut dependencies = std::collections::BTreeSet::new();
                #( #dependencies::add_named_type_with_dependencies_and_generics(
                    &mut dependencies,
                    #names,
                    generics,
                ); )*
                dependencies
            }
        }
    };
    implementation.into()
}

fn extract_path_from_type(ty: &Type) -> Option<Path> {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            let mut path = path.path.clone();
            for segment in &mut path.segments {
                if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    // When calling a static function on `Vec<T>`, it gets invoked
                    // as `Vec::<T>::some_func()`, so we need to insert extra colons
                    // to make the famed Rust "turbofish":
                    args.colon2_token = Some(syn::parse_quote!(::));
                }
            }
            Some(path)
        }
        _ => None,
    }
}

fn parse_type_item(item: TokenStream) -> (Ident, Item, Generics) {
    let item = syn::parse::<Item>(item).unwrap();
    match item {
        Item::Enum(item) => {
            let generics = item.generics.clone();
            (item.ident.clone(), Item::Enum(item), generics)
        }
        Item::Struct(item) => {
            let generics = item.generics.clone();
            (item.ident.clone(), Item::Struct(item), generics)
        }
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
    let serializable_names = serializable_types.iter().map(get_name_from_path);
    let serializable_types = serializable_types.iter();
    let deserializable_named = deserializable_types.iter().map(get_name_from_path);
    let deserializable_types = deserializable_types.iter();
    let replacement = quote! {
        fn __fp_declare_import_fns() -> (fp_bindgen::prelude::FunctionList, std::collections::BTreeSet<Type>, std::collections::BTreeSet<Type>) {
            let mut serializable_import_types = std::collections::BTreeSet::new();
            #( #serializable_types::add_named_type_with_dependencies(&mut serializable_import_types, #serializable_names); )*

            let mut deserializable_import_types = std::collections::BTreeSet::new();
            #( #deserializable_types::add_named_type_with_dependencies(&mut deserializable_import_types, #deserializable_named); )*

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
    let serializable_names = serializable_types.iter().map(get_name_from_path);
    let serializable_types = serializable_types.iter();
    let deserializable_names = deserializable_types.iter().map(get_name_from_path);
    let deserializable_types = deserializable_types.iter();
    let replacement = quote! {
        fn __fp_declare_export_fns() -> (fp_bindgen::prelude::FunctionList, std::collections::BTreeSet<Type>, std::collections::BTreeSet<Type>) {
            let mut serializable_export_types = std::collections::BTreeSet::new();
            #( #serializable_types::add_named_type_with_dependencies(&mut serializable_export_types, #serializable_names); )*

            let mut deserializable_export_types = std::collections::BTreeSet::new();
            #( #deserializable_types::add_named_type_with_dependencies(&mut deserializable_export_types, #deserializable_names); )*

            let mut list = fp_bindgen::prelude::FunctionList::new();
            #( list.add_function(#functions, &serializable_export_types, &deserializable_export_types); )*

            (list, serializable_export_types, deserializable_export_types)
        }
    };
    replacement.into()
}

/// Returns the name of the type based on its path. The result of this should be the same as
/// provided by `Type::name()`, but without the need for constructing an intermediate `Type`.
///
/// If the returned name does *not* match the one returned by `Type::name()` we use that as an
/// indication there's either an alias or a generic argument present in the type, because
/// those show up in the path, but neither aliases nor the specialized types can be known to
/// the `Type` at the implementation site.
fn get_name_from_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| match &segment.arguments {
            PathArguments::None => segment.ident.to_string(),
            PathArguments::AngleBracketed(bracketed) => format!(
                "{}<{}>",
                segment.ident,
                bracketed
                    .args
                    .iter()
                    .map(|arg| match arg {
                        GenericArgument::Type(Type::Path(path)) if path.qself.is_none() =>
                            get_name_from_path(&path.path),
                        _ => panic!("Unsupported generic argument in path: {:?}", path),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PathArguments::Parenthesized(_) => {
                panic!("Unsupported arguments in path: {:?}", path)
            }
        })
        .collect::<Vec<_>>()
        .join("::")
}

/// Parses function declarations and returns them in a list.
/// In addition, it returns a list of doc lines for every function as well.
/// Finally, it returns two sets: one with all the paths for types that may need serialization
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
                        FnArg::Typed(arg) => {
                            serializable_type_names.insert(
                                extract_path_from_type(arg.ty.as_ref()).unwrap_or_else(|| {
                                    panic!(
                                        "Only value types are supported. \
                                            Incompatible argument type in function declaration: {:?}",
                                        function.sig
                                    )
                                }),
                            );
                        }
                    }
                }

                match &function.sig.output {
                    ReturnType::Default => { /* No return value. */ }
                    ReturnType::Type(_, ty) => {
                        deserializable_type_names.insert(
                            extract_path_from_type(ty.as_ref()).unwrap_or_else(|| {
                                panic!(
                                    "Only value types are supported. \
                                        Incompatible return type in function declaration: {:?}",
                                    function.sig
                                )
                            }),
                        );
                    }
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
        Primitive::U8,
        Primitive::U16,
        Primitive::U32,
        Primitive::U64,
    ];

    let mut token_stream = TokenStream::new();
    for primitive in primitives {
        token_stream.extend(primitive.gen_impl().into_iter());
    }
    token_stream
}
