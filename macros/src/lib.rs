mod primitives;
mod serializable;
mod typing;
mod utils;

use crate::{
    primitives::Primitive,
    utils::{extract_path_from_type, get_name_from_path},
};
use once_cell::sync::Lazy;
use proc_macro::{TokenStream, TokenTree};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use std::{collections::HashMap, sync::RwLock};
use syn::{FnArg, ForeignItemFn, ItemFn, ItemUse, Path, ReturnType};
use typing::FnSignature;
use utils::flatten_using_statement;

/// Used to annotate types (`enum`s and `struct`s) that can be passed across the Wasm bridge.
#[proc_macro_derive(Serializable, attributes(fp))]
pub fn derive_serializable(item: TokenStream) -> TokenStream {
    crate::serializable::impl_derive_serializable(item)
}

/// Declares functions the plugin can import from the host runtime.
#[proc_macro]
pub fn fp_import(token_stream: TokenStream) -> TokenStream {
    let (functions, serializable_types, deserializable_types) = parse_statements(token_stream);
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
    let (functions, serializable_types, deserializable_types) = parse_statements(token_stream);
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

/// Parses statements like function declearations and 'use Foobar;' and returns them in a list.
/// In addition, it returns a list of doc lines for every function as well.
/// Finally, it returns two sets: one with all the paths for types that may need serialization
/// to call the functions, and one with all the paths for types that may need deserialization to
/// call the functions.
fn parse_statements(token_stream: TokenStream) -> (Vec<String>, Vec<Path>, Vec<Path>) {
    let mut functions = Vec::new();
    let mut serializable_type_names = Vec::new();
    let mut deserializable_type_names = Vec::new();
    let mut current_item_tokens = Vec::<TokenTree>::new();
    for token in token_stream.into_iter() {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                current_item_tokens.push(TokenTree::Punct(punct));

                let stream = current_item_tokens.into_iter().collect::<TokenStream>();

                if let Ok(function) = syn::parse::<ForeignItemFn>(stream.clone()) {
                    for input in &function.sig.inputs {
                        match input {
                            FnArg::Receiver(_) => panic!(
                                "Methods are not supported. Found `self` in function declaration: {:?}",
                                function.sig
                            ),
                            FnArg::Typed(arg) => {
                                serializable_type_names.push(
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
                            deserializable_type_names.push(
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
                } else if let Ok(using) = syn::parse::<ItemUse>(stream) {
                    for path in flatten_using_statement(using) {
                        deserializable_type_names.push(path.clone());
                        serializable_type_names.push(path);
                    }
                }

                current_item_tokens = Vec::new();
            }
            other => current_item_tokens.push(other),
        }
    }

    serializable_type_names.dedup();
    deserializable_type_names.dedup();

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

static EXPORTED_SIGNATURES: Lazy<RwLock<HashMap<String, FnSignature>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[proc_macro_attribute]
#[proc_macro_error]
pub fn fp_export_signature(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());
    let func = match syn::parse_macro_input::parse::<ForeignItemFn>(input.clone()) {
        Ok(func) => func,
        Err(e) => abort!(e),
    };
    let sig: FnSignature = (&func.sig).into();
    if EXPORTED_SIGNATURES
        .write()
        .unwrap()
        .insert(sig.name.clone(), sig)
        .is_some()
    {
        emit_error!(func, "Can't export the same function name multiple times");
    }
    TokenStream::default() //eat the signature
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn fp_export_impl(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());

    let func = match syn::parse_macro_input::parse::<ItemFn>(input.clone()) {
        Ok(func) => func,
        Err(e) => abort!(e.to_compile_error(), "Not a valid function signature"),
    };

    typing::type_check_export(&func.sig);

    let args = func
        .sig
        .inputs
        .iter()
        .map(|arg| {
            let pt = typing::get_type(arg);
            (arg, pt, typing::is_type_complex(&pt.ty))
        })
        .collect::<Vec<_>>();

    // Check each argument and replace with FatPtr for complex types
    let formatted_args = args
        .iter()
        .map(|&(arg, pt, is_complex)| {
            if is_complex {
                let arg_name = pt.pat.as_ref();
                quote! {#arg_name : protocol::FatPtr}
            } else {
                arg.to_token_stream()
            }
        })
        .collect::<Vec<_>>();

    let (names, types): (Vec<_>, Vec<_>) = args
        .iter()
        .filter_map(|&(_, pt, is_complex)| {
            if is_complex {
                Some((pt.pat.as_ref(), pt.ty.as_ref()))
            } else {
                None
            }
        })
        .unzip();

    let call_args = args
        .iter()
        .map(|&(_, pt, _)| pt.pat.as_ref())
        .collect::<Vec<_>>();
    let fn_name = &func.sig.ident;

    // Check the output type and replace complex ones with FatPtr
    let (output, return_wrapper) = if typing::is_ret_type_complex(&func.sig.output) {
        (
            quote! {-> protocol::FatPtr},
            quote! {let ret = protocol::export_value_to_host(&ret);},
        )
    } else {
        (func.sig.output.to_token_stream(), Default::default())
    };

    let func_call = if func.sig.asyncness.is_some() {
        quote! {
                let len = std::mem::size_of::<protocol::AsyncValue>() as u32;
                let ptr = protocol::malloc(len);
                let fat_ptr = protocol::to_fat_ptr(ptr, len);

                Task::spawn(Box::pin(async move {
                    let ret = #fn_name(#(#call_args),*).await;
                    unsafe {
                        let result_ptr = protocol::export_value_to_host(&ret);
                        protocol::host_resolve_async_value(fat_ptr, result_ptr);
                    }
                }));

                let ret = fat_ptr;
        }
    } else {
        quote! {
            let ret = #fn_name(#(#call_args),*);
            #return_wrapper
        }
    };

    let ts: proc_macro2::TokenStream = input.clone().into();
    let exported_fn_name = format_ident!("__fp_gen_{}", fn_name);

    //build the actual exported wrapper function
    (quote! {
        #[no_mangle]
        fn #exported_fn_name( #(#formatted_args),* ) #output {
            #(let #names = unsafe { protocol::import_value_from_host::<#types>(#names) };)*
            #func_call
            ret
        }
        #ts
    })
    .into()
}
