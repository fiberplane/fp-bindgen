mod primitives;
mod serializable;
mod typing;
mod utils;

use std::iter::once;

use crate::{
    primitives::Primitive,
    utils::{extract_path_from_type, get_name_from_path},
};
use proc_macro::{TokenStream, TokenTree};
use proc_macro_error::{abort, proc_macro_error, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    AttributeArgs, FnArg, ForeignItemFn, GenericParam, ItemFn, ItemUse, Pat, PatPath, Path,
    PathArguments, PathSegment, ReturnType,
};
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

#[proc_macro_attribute]
#[proc_macro_error]
pub fn fp_export_signature(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());

    let func = syn::parse_macro_input::parse::<ForeignItemFn>(input.clone()).unwrap_or_abort();
    let args = typing::extract_args(&func.sig).collect::<Vec<_>>();

    let mut sig = func.sig.clone();
    //Massage the signature into what we wish to export
    {
        typing::morph_signature(&mut sig);
        sig.inputs = sig
            .inputs
            .into_iter()
            //append a function ptr to the end which signature matches the original exported function
            .chain(once({
                let input_types = args.iter().map(|(_, pt, _)| pt.ty.as_ref());
                let output = if func.sig.asyncness.is_some() {
                    syn::parse::<ReturnType>((quote! {-> FUT}).into()).unwrap_or_abort()
                } else {
                    func.sig.output.clone()
                };

                syn::parse::<FnArg>((quote! {fptr: fn (#(#input_types),*) #output}).into())
                    .unwrap_or_abort()
            }))
            .collect();
        sig.generics.params.clear();
        if func.sig.asyncness.is_some() {
            let output = typing::get_output_type(&func.sig.output);
            sig.generics.params.push(
                syn::parse::<GenericParam>(
                    (quote! {FUT: std::future::Future<Output=#output>}).into(),
                )
                .unwrap_or_abort(),
            )
        }
    }

    let (complex_names, complex_types): (Vec<_>, Vec<_>) = args
        .iter()
        .filter_map(|&(_, pt, is_complex)| {
            if is_complex {
                Some((pt.pat.as_ref(), pt.ty.as_ref()))
            } else {
                None
            }
        })
        .unzip();

    let names = args.iter().map(|(_, pt, _)| pt.pat.as_ref());

    // Check the output type and replace complex ones with FatPtr
    let return_wrapper = if typing::is_ret_type_complex(&func.sig.output) {
        quote! {let ret = fp_bindgen_lib::export_value_to_host(&ret);}
    } else {
        Default::default()
    };

    let func_call = quote! {(fptr)(#(#names),*)};

    let func_wrapper = if func.sig.asyncness.is_some() {
        quote! {
                let len = std::mem::size_of::<fp_bindgen_lib::AsyncValue>() as u32;
                let ptr = fp_bindgen_lib::malloc(len);
                let fat_ptr = fp_bindgen_lib::to_fat_ptr(ptr, len);

                fp_bindgen_lib::Task::spawn(Box::pin(async move {
                    let ret = #func_call.await;
                    unsafe {
                        let result_ptr = fp_bindgen_lib::export_value_to_host(&ret);
                        fp_bindgen_lib::host_resolve_async_value(fat_ptr, result_ptr);
                    }
                }));

                let ret = fat_ptr;
        }
    } else {
        quote! {
            let ret = #func_call;
            #return_wrapper
        }
    };

    //build the actual exported wrapper function
    (quote! {
        /// This is a implementation detail an should not be called directly
        #[inline(always)]
        pub #sig {
            #(let #complex_names = unsafe { fp_bindgen_lib::import_value_from_host::<#complex_types>(#complex_names) };)*
            #func_wrapper
            ret
        }
    })
    .into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn fp_export_impl(attributes: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());

    let func = syn::parse_macro_input::parse::<ItemFn>(input.clone()).unwrap_or_abort();
    let attrs =
        syn::parse_macro_input::parse::<AttributeArgs>(attributes.clone()).unwrap_or_abort();

    let protocol_path = attrs
        .get(0)
        .map(|om| match om {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => path,
                _ => abort!(meta, "unsupported attribute, must name a path"),
            },
            _ => abort!(om, "unsupported attribute, must name a path"),
        })
        .unwrap_or_else(|| abort!(func, "missing attribute. Must name which provider is being implemented eg: #[fp_export_impl(foobar)]"));

    let args = typing::extract_args(&func.sig).collect::<Vec<_>>();

    let mut sig = func.sig.clone();
    //Massage the signature into what we wish to export
    {
        typing::morph_signature(&mut sig);
        sig.ident = format_ident!("__fp_gen_{}", sig.ident);
    }

    let fn_name = &func.sig.ident;

    let impl_fn_pat = Pat::Path(PatPath {
        attrs: vec![],
        qself: None,
        path: PathSegment {
            ident: func.sig.ident.clone(),
            arguments: PathArguments::None,
        }
        .into(),
    });
    let call_args = args
        .iter()
        .map(|&(_, pt, _)| pt.pat.as_ref())
        .chain(once(&impl_fn_pat))
        .collect::<Vec<_>>();

    let ts: proc_macro2::TokenStream = input.clone().into();
    //build the actual exported wrapper function
    (quote! {
        #[no_mangle]
        pub #sig {
            #protocol_path::#fn_name(#(#call_args),*)
        }
        #ts
    })
    .into()
}
