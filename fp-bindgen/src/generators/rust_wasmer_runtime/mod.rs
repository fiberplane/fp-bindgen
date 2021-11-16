use crate::functions::{Function, FunctionArg, FunctionList};
use crate::generators::rust_plugin::{
    generate_type_bindings,
};
use crate::types::Type;
use crate::WasmerRuntimeConfig;
use proc_macro2::{Punct, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::token::Async;
use std::collections::BTreeSet;
use std::fs;


pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    _runtime_config: WasmerRuntimeConfig,
    path: &str,
) {
    let spec_path = format!("{}/spec", path);
    fs::create_dir_all(&spec_path).expect("Could not create spec/ directory");

    // We use the same type generation as for the Rust plugin, only with the
    // serializable and deserializable types inverted:
    generate_type_bindings(
        deserializable_types,
        serializable_types,
        &spec_path,
        "rust_wasmer_runtime",
    );

    generate_function_bindings(
        import_functions,
        export_functions,
        &spec_path,
    );

    write_bindings_file(
        format!("{}/errors.rs", path),
        include_bytes!("assets/errors.rs"),
    );
    write_bindings_file(format!("{}/lib.rs", path), include_bytes!("assets/lib.rs"));
    write_bindings_file(
        format!("{}/support.rs", path),
        include_bytes!("assets/support.rs"),
    );
}

fn generate_create_import_object_func(import_functions: &FunctionList) -> TokenStream {
    //yes this is pretty ugly but fortunately *only* required here to get proper formatting with quote
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


pub struct ExportSafeFunctionArg<'a>(pub &'a FunctionArg);

impl ToTokens for ExportSafeFunctionArg<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.0.name);
        let ty = ExportSafeType(&self.0.ty);
        quote!(#name: #ty).to_tokens(tokens)
    }
}

pub struct ExportSafeType<'a>(pub &'a Type);

impl ToTokens for ExportSafeType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.0 {
            Type::Primitive(p) => quote! {#p},
            _ => quote! {FatPtr},
        }
        .to_tokens(tokens)
    }
}

struct ComplexArgsToVec<'a>(&'a FunctionArg);

impl ToTokens for ComplexArgsToVec<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.0.name);
        let ty = match self.0.ty {
            Type::Primitive(p) => quote! {#p},
            _ => (quote! {Vec<u8>}),
        };
        (quote! {#name: #ty}).to_tokens(tokens)
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

        let arg_names: Vec<_> = args.iter().map(|a| format_ident!("{}", a.name)).collect();
        let serialize_names: Vec<_> = args
            .iter()
            .filter_map(|a| {
                (!matches!(a.ty, Type::Primitive(..))).then(|| format_ident!("{}", a.name))
            })
            .collect();
        let raw_format_args = args.iter().map(ComplexArgsToVec);
        let safe_arg_types = args.iter().map(|a| ExportSafeType(&a.ty));
        let safe_return_type = ExportSafeType(return_type);

        let asyncness = is_async.then(|| Async::default());
        let awaiter = is_async.then(|| quote! {let res = res.await?;});

        let return_wrapper = if *is_async { 
            quote!{
                let result = ModuleRawFuture::new(env.clone(), result).await;
            }
        } else {
            quote!{
                let result = import_from_guest_raw(&env, result);
            }
        };

        (quote! {
            #(#[doc = #doc_lines])*
            pub #asyncness fn #name(&self #(,#args)*) -> Result<#return_type, InvocationError> {
                #(let #serialize_names = serialize_to_vec(#serialize_names);)*

                let res = self.#raw_name(#(#arg_names),*);

                #awaiter

                rmp_serde::from_slice(&res).unwrap()
            }
            
            pub #asyncness fn #raw_name(&self #(,#raw_format_args)*) -> Result<Vec<u8>, InvocationError> {
                let mut env = RuntimeInstanceData::default();
                let import_object = create_import_object(self.module.store(), &env);
                let instance = Instance::new(&self.module, &import_object).unwrap();
                env.init_with_instance(&instance).unwrap();
                
                #(let #serialize_names = export_to_guest_raw(#serialize_names);)*
                
                let function = instance
                    .exports
                    .get_native_function::<(#(#safe_arg_types),*), #safe_return_type>(#fp_gen_name)
                    .map_err(|_| InvocationError::FunctionNotExported)?;

                let result = function.call((#(#arg_names),*))?;
                
                #return_wrapper
                
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
        let input_args = args.iter().map(ExportSafeFunctionArg);
        let wrapper_return_type = if *is_async {
            quote! {-> FatPtr}
        }
        else if matches!(return_type, Type::Unit) {
            TokenStream::default()
        }
        else {
            let est = ExportSafeType(return_type);
            quote!{-> #est}
        };

        let complex_args = args
            .iter()
            .filter(|a| !matches!(a.ty, Type::Primitive(_)))
            .collect::<Vec<_>>();
        let complex_types = complex_args
            .iter()
            .map(|a| &a.ty);
        let complex_idents = complex_args.iter().map(|a| format_ident!("{}", a.name)).collect::<Vec<_>>();

        let impl_func_name = format_ident!("{}", name);
        let arg_idents = args.iter().map(|a| format_ident!("{}",a.name));
        let func_call = quote!(super::#impl_func_name(#(#arg_idents),*));

        let wrapper = if *is_async {
            quote!{
                let env = env.clone();
                let async_ptr = create_future_value(&env);
                let handle = tokio::runtime::Handle::current();
                handle.spawn(async move {
                    let result = result.await;
                    let result_ptr = export_to_guest(&env, &result);
                    unsafe {
                        env.__fp_guest_resolve_async_value
                            .get_unchecked()
                            .call(async_ptr, result_ptr)
                            .expect("Runtime error: Cannot resolve async value");
                    }
                });
                async_ptr
            }
        }
        else
        {
            match return_type {
                Type::Primitive(_) => quote!{result},
                Type::Unit => quote!{()},
                _ => quote!{export_to_guest(env, &result)}
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
        use crate::errors::InvocationError;
        use crate::{
            support::{
                create_future_value, export_to_guest, export_to_guest_raw, import_from_guest,
                resolve_async_value, FatPtr, ModuleRawFuture,
            },
            Runtime, RuntimeInstanceData,
        };
        use wasmer::{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv};
        #newline
        #newline
        impl Runtime {
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
    use super::ExportSafeFunctionArg;
    use crate::{functions::FunctionArg, types::Type};
    use quote::ToTokens;

    #[test]
    fn test_function_arg_to_tokens() {
        let arg = FunctionArg {
            name: "foobar".into(),
            ty: Type::String,
        };
        let arg = ExportSafeFunctionArg(&arg);

        let stringified = arg.into_token_stream().to_string();

        pretty_assertions::assert_eq!(&stringified, "foobar : FatPtr");
    }
}
