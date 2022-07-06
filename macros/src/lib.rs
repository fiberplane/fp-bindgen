use crate::{primitives::Primitive, utils::extract_path_from_type};
use proc_macro::{TokenStream, TokenTree};
use proc_macro_error::{abort, proc_macro_error, ResultExt};
use quote::{format_ident, quote, ToTokens};
use std::{
    collections::{HashMap, HashSet},
    iter::once,
};
use syn::{
    AttributeArgs, FnArg, ForeignItemFn, GenericParam, ItemFn, ItemType, ItemUse, Pat, PatPath,
    Path, PathArguments, PathSegment, ReturnType,
};
use utils::{flatten_using_statement, normalize_return_type};

mod primitives;
mod serializable;
mod typing;
mod utils;

/// Used to annotate types (`enum`s and `struct`s) that can be passed across the Wasm bridge.
#[proc_macro_derive(Serializable, attributes(fp))]
pub fn derive_serializable(item: TokenStream) -> TokenStream {
    crate::serializable::impl_derive_serializable(item)
}

/// Declares functions the plugin can import from the host runtime.
#[proc_macro]
pub fn fp_import(token_stream: TokenStream) -> TokenStream {
    let ParsedStatements {
        functions,
        type_paths,
        aliases,
    } = parse_statements(token_stream);
    let type_paths = type_paths.iter();
    let alias_keys = aliases.keys();
    let alias_paths = aliases
        .values()
        .map(|path| path.to_token_stream().to_string());

    let replacement = quote! {
        fn __fp_declare_import_fns() -> (fp_bindgen::prelude::FunctionList, fp_bindgen::prelude::TypeMap) {
            let mut import_types = fp_bindgen::prelude::TypeMap::new();
            #( #type_paths::collect_types(&mut import_types); )*
            #( import_types.insert(TypeIdent::from(#alias_keys), Type::Alias(#alias_keys.to_owned(), std::str::FromStr::from_str(#alias_paths).unwrap())); )*

            let mut list = fp_bindgen::prelude::FunctionList::new();
            #( list.add_function(#functions); )*

            (list, import_types)
        }
    };
    replacement.into()
}

/// Declares functions the plugin may export to the host runtime.
#[proc_macro]
pub fn fp_export(token_stream: TokenStream) -> TokenStream {
    let ParsedStatements {
        functions,
        type_paths,
        aliases,
    } = parse_statements(token_stream);
    let type_paths = type_paths.iter();
    let alias_keys = aliases.keys();
    let alias_paths = aliases
        .values()
        .map(|path| path.to_token_stream().to_string());

    let replacement = quote! {
        fn __fp_declare_export_fns() -> (fp_bindgen::prelude::FunctionList, fp_bindgen::prelude::TypeMap) {
            let mut export_types = fp_bindgen::prelude::TypeMap::new();
            #( #type_paths::collect_types(&mut export_types); )*
            #( export_types.insert(TypeIdent::from(#alias_keys), Type::Alias(#alias_keys.to_owned(), std::str::FromStr::from_str(#alias_paths).unwrap())); )*

            let mut list = fp_bindgen::prelude::FunctionList::new();
            #( list.add_function(#functions); )*

            (list, export_types)
        }
    };
    replacement.into()
}

/// Contains all the relevant information extracted from inside the `fp_import!` and `fp_export!`
/// macros.
struct ParsedStatements {
    pub functions: Vec<String>,
    pub type_paths: HashSet<Path>,
    pub aliases: HashMap<String, Path>,
}

/// Parses statements like function declarations and 'use Foobar;' and returns them in a list.
/// In addition, it returns a list of doc lines for every function as well.
/// Finally, it returns two sets: one with all the paths for types that may need serialization
/// to call the functions, and one with all the paths for types that may need deserialization to
/// call the functions.
fn parse_statements(token_stream: TokenStream) -> ParsedStatements {
    let mut functions = Vec::new();
    let mut type_paths = HashSet::new();
    let mut aliases = HashMap::new();

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
                                type_paths.insert(
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

                    if let Some(ty) = normalize_return_type(&function.sig.output) {
                        type_paths.insert(extract_path_from_type(ty).unwrap_or_else(|| {
                            panic!(
                                "Only value types are supported. \
                                            Incompatible return type in function declaration: {:?}",
                                function.sig
                            )
                        }));
                    }

                    functions.push(function.into_token_stream().to_string());
                } else if let Ok(using) = syn::parse::<ItemUse>(stream.clone()) {
                    for path in flatten_using_statement(using) {
                        type_paths.insert(path);
                    }
                } else if let Ok(type_alias) = syn::parse::<ItemType>(stream) {
                    aliases.insert(
                        type_alias.ident.to_string(),
                        extract_path_from_type(type_alias.ty.as_ref()).unwrap_or_else(|| {
                            panic!(
                                "Only value types are supported. \
                                    Incompatible type in alias: {:?}",
                                type_alias
                            )
                        }),
                    );
                }

                current_item_tokens = Vec::new();
            }
            other => current_item_tokens.push(other),
        }
    }

    ParsedStatements {
        functions,
        type_paths,
        aliases,
    }
}

/// Generates bindings for the functions declared in the `fp_import!{}` and `fp_export!{}` blocks.
#[proc_macro]
pub fn fp_bindgen(args: TokenStream) -> TokenStream {
    let args: proc_macro2::TokenStream = args.into();
    let replacement = quote! {
        let (import_functions, import_types) = __fp_declare_import_fns();
        let (export_functions, mut export_types) = __fp_declare_export_fns();

        let mut types = import_types;
        types.append(&mut export_types);

        fp_bindgen::generate_bindings(
            import_functions,
            export_functions,
            types,
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

/// Exports a signature in a provider crate.
/// This is not meant to be used directly.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn fp_export_signature(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());

    let func = syn::parse_macro_input::parse::<ForeignItemFn>(input.clone()).unwrap_or_abort();
    let args = typing::extract_args(&func.sig).collect::<Vec<_>>();

    let mut sig = func.sig.clone();
    //Massage the signature into what we wish to export
    {
        typing::morph_signature(&mut sig, "fp_bindgen_support");
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
                    //the 'static life time is ok since we give it a box::pin
                    (quote! {FUT: std::future::Future<Output=#output> + 'static}).into(),
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
    let func_call = quote! {(fptr)(#(#names),*)};

    let func_wrapper = if func.sig.asyncness.is_some() {
        quote! {
            let ret = fp_bindgen_support::guest::r#async::task::Task::alloc_and_spawn(#func_call);
        }
    } else {
        // Check the output type and replace complex ones with FatPtr
        let return_wrapper = if typing::is_ret_type_complex(&func.sig.output) {
            quote! {let ret = fp_bindgen_support::guest::io::export_value_to_host(&ret);}
        } else {
            Default::default()
        };
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
            #(let #complex_names = unsafe { fp_bindgen_support::guest::io::import_value_from_host::<#complex_types>(#complex_names) };)*
            #func_wrapper
            ret
        }
    })
    .into()
}

/// Exports an implementation of a specific provider function
///
/// Example usage of implementing a `log` function of a `logger` provider:
/// ```no_compile
/// use fp_bindgen_macros::fp_export_impl; //this would be `logger::fp_export_impl` inside the plugin crate
/// #[fp_export_impl(logger)]
/// pub fn log(msg: String, foo: String) -> String {
///     format!("{} + {} => {0}{1}", msg, foo)
/// }
/// ```
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
        typing::morph_signature(
            &mut sig,
            protocol_path.to_token_stream().to_string().as_str(),
        );
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

/// Imports a signature in a provider crate.
/// This is not meant to be used directly.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn fp_import_signature(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    proc_macro_error::set_dummy(input.clone().into());

    let func = syn::parse_macro_input::parse::<ForeignItemFn>(input.clone()).unwrap_or_abort();
    let args = typing::extract_args(&func.sig).collect::<Vec<_>>();

    let wrapper_sig = func.sig.clone();
    let mut extern_sig = wrapper_sig.clone();
    //Massage the signature into what we wish to export
    {
        extern_sig.ident = format_ident!("__fp_gen_{}", extern_sig.ident);
        typing::morph_signature(&mut extern_sig, "fp_bindgen_support");
    }

    let complex_names: Vec<_> = args
        .iter()
        .filter_map(|&(_, pt, is_complex)| {
            if is_complex {
                Some(pt.pat.as_ref())
            } else {
                None
            }
        })
        .collect();

    let names = args.iter().map(|(_, pt, _)| pt.pat.as_ref());
    let extern_ident = &extern_sig.ident;
    let func_call = quote! {#extern_ident(#(#names),*)};

    let ret_wrapper = if func.sig.asyncness.is_some() {
        quote! {
            let ret = unsafe {
                fp_bindgen_support::guest::io::import_value_from_host(fp_bindgen_support::guest::r#async::HostFuture::new(ret).await)
            };
        }
    } else {
        // Check the output type and replace complex ones with FatPtr
        if typing::is_ret_type_complex(&func.sig.output) {
            quote! {
                let ret = unsafe { fp_bindgen_support::guest::io::import_value_from_host(ret) };
            }
        } else {
            Default::default()
        }
    };

    let attrs = &func.attrs;

    //build the actual imported wrapper function
    (quote! {
        #[link(wasm_import_module = "fp")]
        extern "C" { #extern_sig; }

        #[inline(always)]
        #(#attrs)*
        pub #wrapper_sig {
            #(let #complex_names = fp_bindgen_support::guest::io::export_value_to_host(&#complex_names);)*
            let ret = unsafe { #func_call };
            #ret_wrapper
            ret
        }
    })
    .into()
}
