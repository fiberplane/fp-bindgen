use crate::{
    functions::{Function, FunctionArg, FunctionList},
    generators::rust_plugin::generate_type_bindings,
    primitives::Primitive,
    types::{TypeIdent, TypeMap},
};
use proc_macro2::{Punct, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::{fs, str::FromStr};
use syn::token::Async;

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: TypeMap,
    path: &str,
) {
    fs::create_dir_all(&path).expect("Could not create output directory");

    // We use the same type generation as for the Rust plugin, only with the
    // serializable and deserializable types inverted:
    generate_type_bindings(&types, path, "rust_wasmer_runtime");

    generate_function_bindings(import_functions, export_functions, path);
}

fn generate_create_import_object_func(import_functions: &FunctionList) -> TokenStream {
    // Yes, this is pretty ugly but fortunately *only* required here to get
    // proper formatting with quote!, since rustfmt doesn't format inside macro
    // invocations :(
    let newline = Punct::new('\n', proc_macro2::Spacing::Alone);
    let space = Punct::new(' ', proc_macro2::Spacing::Joint);
    let spaces4: Vec<_> = (0..3).map(|_| &space).collect();
    let spaces8: Vec<_> = (0..7).map(|_| &space).collect();
    let spaces8 = quote! {#(#spaces8)*};

    let fp_gen_names = import_functions
        .iter()
        .map(|function| format!("__fp_gen_{}", function.name));
    let names = import_functions
        .iter()
        .map(|function| format_ident!("_{}", function.name));

    quote! {
        fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
            imports! {
                #newline
                #(#spaces4)* "fp" => {
                    #newline
                    #spaces8 "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
                    #newline
                    #(
                        #spaces8 #fp_gen_names => Function::new_native_with_env(store, env.clone(), #names),
                        #newline
                    )*
                #(#spaces4)* }
                #newline
            }
        }
    }
}

pub struct WasmType<'a>(pub &'a TypeIdent);

impl ToTokens for WasmType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Ok(p) = Primitive::from_str(&self.0.name) {
            quote! { #p }
        } else {
            quote! { FatPtr }
        }
        .to_tokens(tokens)
    }
}

pub struct WasmArg<'a>(pub &'a FunctionArg);

impl ToTokens for WasmArg<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.0.name);
        let ty = WasmType(&self.0.ty);
        quote!(#name: #ty).to_tokens(tokens)
    }
}

struct RawType<'a>(&'a TypeIdent);

impl ToTokens for RawType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Ok(p) = Primitive::from_str(&self.0.name) {
            quote! { #p }
        } else {
            quote! { Vec<u8> }
        }
        .to_tokens(tokens)
    }
}

struct RawArg<'a>(&'a FunctionArg);

impl ToTokens for RawArg<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.0.name);
        let ty = RawType(&self.0.ty);
        (quote! { #name: #ty }).to_tokens(tokens)
    }
}

struct RuntimeImportedFunction<'a>(&'a Function);

impl ToTokens for RuntimeImportedFunction<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let newline = Punct::new('\n', proc_macro2::Spacing::Alone);

        let Function {
            name,
            doc_lines,
            args,
            return_type,
            is_async,
        } = self.0;

        let fp_gen_name = format!("__fp_gen_{}", name);
        let raw_name = format_ident!("{}_raw", name);
        let name = format_ident!("{}", name);

        let arg_names: Vec<_> = args
            .iter()
            .map(|arg| format_ident!("{}", arg.name))
            .collect();
        let serialize_names: Vec<_> = args
            .iter()
            .filter(|arg| !&arg.ty.is_primitive())
            .map(|arg| format_ident!("{}", arg.name))
            .collect();
        let wasm_arg_types = args.iter().map(|arg| WasmType(&arg.ty));
        let wasm_return_type = match return_type {
            Some(ty) => {
                let ty = WasmType(ty);
                quote! { #ty }
            }
            None => quote! { () },
        };
        let raw_format_args = args.iter().map(RawArg);
        let raw_format_return_type = match return_type {
            Some(ty) => {
                let raw = RawType(ty);
                quote! { #raw }
            }
            None => quote! { () },
        };

        let asyncness = is_async.then(Async::default);

        let (raw_return_wrapper, return_wrapper) = if *is_async {
            (
                quote! {
                    let result = ModuleRawFuture::new(env.clone(), result).await;
                },
                quote! {
                    let result = result.await;
                    let result = result.map(|ref data| deserialize_from_slice(data));
                },
            )
        } else if !return_type
            .as_ref()
            .map(TypeIdent::is_primitive)
            .unwrap_or(true)
        {
            (
                quote! {
                    let result = import_from_guest_raw(&env, result);
                },
                quote! {
                    let result = result.map(|ref data| deserialize_from_slice(data));
                },
            )
        } else {
            (TokenStream::default(), TokenStream::default())
        };

        let return_type = match return_type {
            Some(ident) => quote! { #ident },
            None => quote! { () },
        };

        (quote! {
            #(#[doc = #doc_lines])*
            pub #asyncness fn #name(&self #(,#args)*) -> Result<#return_type, InvocationError> {
                #(let #serialize_names = serialize_to_vec(&#serialize_names);)*

                let result = self.#raw_name(#(#arg_names),*);

                #return_wrapper

                result
            }

            pub #asyncness fn #raw_name(&self #(,#raw_format_args)*) -> Result<#raw_format_return_type, InvocationError> {
                let mut env = RuntimeInstanceData::default();
                let import_object = create_import_object(self.module.store(), &env);
                let instance = Instance::new(&self.module, &import_object).unwrap();
                env.init_with_instance(&instance).unwrap();

                #(let #serialize_names = export_to_guest_raw(&env, #serialize_names);)*

                let function = instance
                    .exports
                    .get_native_function::<(#(#wasm_arg_types),*), #wasm_return_type>(#fp_gen_name)
                    .map_err(|_| InvocationError::FunctionNotExported)?;

                let result = function.call(#(#arg_names),*)?;

                #raw_return_wrapper

                Ok(result)
            }
            #newline
            #newline
            #newline
        })
        .to_tokens(tokens)
    }
}

struct RuntimeExportedFunction<'a>(&'a Function);

impl ToTokens for RuntimeExportedFunction<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Function {
            name,
            args,
            is_async,
            return_type,
            ..
        } = self.0;

        let underscore_name = format_ident!("_{}", name);
        let input_args = args.iter().map(WasmArg);
        let wrapper_return_type = if *is_async {
            quote! { -> FatPtr }
        } else {
            match return_type {
                Some(ty) => {
                    let ty = WasmType(ty);
                    quote! { -> #ty }
                }
                None => TokenStream::default(),
            }
        };

        let complex_args = args
            .iter()
            .filter(|arg| !arg.ty.is_primitive())
            .collect::<Vec<_>>();
        let complex_types = complex_args.iter().map(|a| &a.ty);
        let complex_idents = complex_args
            .iter()
            .map(|arg| format_ident!("{}", arg.name))
            .collect::<Vec<_>>();

        let impl_func_name = format_ident!("{}", name);
        let arg_idents = args.iter().map(|a| format_ident!("{}", a.name));
        let func_call = quote!(super::#impl_func_name(#(#arg_idents),*));

        let wrapper = if *is_async {
            quote! {
                let env = env.clone();
                let async_ptr = create_future_value(&env);
                let handle = tokio::runtime::Handle::current();
                handle.spawn(async move {
                    let result = result.await;
                    let result_ptr = export_to_guest(&env, &result);
                    env.guest_resolve_async_value(async_ptr, result_ptr);
                });
                async_ptr
            }
        } else {
            match return_type {
                None => quote! { () },
                Some(ty) if ty.is_primitive() => quote! { result },
                _ => quote! { export_to_guest(env, &result) },
            }
        };

        let newline = Punct::new('\n', proc_macro2::Spacing::Alone);

        (quote! {
            pub fn #underscore_name(env: &RuntimeInstanceData #(,#input_args)*) #wrapper_return_type {
                #(let #complex_idents = import_from_guest::<#complex_types>(env, #complex_idents);)*

                let result = #func_call;
                #wrapper
            }
            #newline
            #newline
        }).to_tokens(tokens)
    }
}

pub fn generate_function_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    path: &str,
) {
    let newline = Punct::new('\n', proc_macro2::Spacing::Alone);
    let create_import_object_func = generate_create_import_object_func(&import_functions);

    let imports = import_functions.iter().map(RuntimeExportedFunction);
    let exports = export_functions.iter().map(RuntimeImportedFunction);

    let full = rustfmt_wrapper::rustfmt(quote! {
        use super::types::*;
        use fp_bindgen_support::{
            common::mem::FatPtr,
            host::{
                errors::{InvocationError, RuntimeError},
                mem::{export_to_guest, export_to_guest_raw, import_from_guest, import_from_guest_raw, deserialize_from_slice, serialize_to_vec},
                r#async::{create_future_value, future::ModuleRawFuture, resolve_async_value},
                runtime::RuntimeInstanceData,
            },
        };
        use wasmer::{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv};
        #newline
        #newline
        pub struct Runtime {
            module: Module,
        }
        #newline
        #newline
        impl Runtime {
            pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
                let store = Self::default_store();
                let module = Module::new(&store, wasm_module)?;

                Ok(Self { module })
            }
            #newline
            #newline
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            fn default_store() -> wasmer::Store {
                let compiler = wasmer_compiler_cranelift::Cranelift::default();
                let engine = wasmer_engine_universal::Universal::new(compiler).engine();
                Store::new(&engine)
            }
            #newline
            #newline
            #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
            fn default_store() -> wasmer::Store {
                let compiler = wasmer_compiler_singlepass::Singlepass::default();
                let engine = wasmer_engine_universal::Universal::new(compiler).engine();
                Store::new(&engine)
            }
            #newline
            #newline
            #(#exports)*
        }
        #newline
        #newline

        #create_import_object_func

        #newline
        #newline
        #(#imports)*

    })
    .unwrap();

    write_bindings_file(format!("{}/bindings.rs", path), full);
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}

#[cfg(test)]
mod test {
    use super::WasmArg;
    use crate::{functions::FunctionArg, types::TypeIdent};
    use quote::ToTokens;

    #[test]
    fn test_function_arg_to_tokens() {
        let arg = FunctionArg {
            name: "foobar".into(),
            ty: TypeIdent::from("String"),
        };
        let arg = WasmArg(&arg);

        let stringified = arg.into_token_stream().to_string();

        pretty_assertions::assert_eq!(&stringified, "foobar : FatPtr");
    }
}
