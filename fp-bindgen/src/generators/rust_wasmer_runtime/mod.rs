use crate::{
    functions::{Function, FunctionArg, FunctionList},
    generators::rust_plugin::{
        format_doc_lines, format_ident, format_modifiers, generate_type_bindings,
    },
    types::{TypeIdent, TypeMap},
};
use std::fs;

pub(crate) fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: TypeMap,
    path: &str,
) {
    fs::create_dir_all(&path).expect("Could not create output directory");

    // We use the same type generation as for the Rust plugin, only with the
    // serializable and deserializable types inverted:
    generate_type_bindings(&types, path, "rust_wasmer_runtime");

    generate_function_bindings(import_functions, export_functions, &types, path);
}

fn generate_create_import_object_func(import_functions: &FunctionList) -> String {
    let imports = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            format!(
                "\"__fp_gen_{name}\" => Function::new_typed_with_env(store, env, _{name}),"
            )
        })
        .collect::<Vec<_>>()
        .join("\n            ");

    format!(
        r#"fn create_import_object(store: &mut Store, env: &FunctionEnv<RuntimeInstanceData>) -> Imports {{
    imports! {{
        "fp" => {{
            "__fp_host_resolve_async_value" => Function::new_typed_with_env(store, env, resolve_async_value),
            {imports}
        }}
    }}
}}"#
    )
}

pub(crate) fn format_raw_ident(ty: &TypeIdent, types: &TypeMap) -> String {
    if ty.is_primitive() {
        format_ident(ty, types)
    } else {
        "Vec<u8>".to_owned()
    }
}

pub(crate) fn format_wasm_ident(ty: &TypeIdent) -> String {
    if ty.is_primitive() {
        format!("<{} as WasmAbi>::AbiType", ty.name)
    } else {
        "FatPtr".to_owned()
    }
}

pub(crate) fn generate_import_function_variables<'a>(
    function: &'a Function,
    types: &TypeMap,
) -> (
    String,
    String,
    &'a String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    let doc = format_doc_lines(&function.doc_lines);
    let modifiers = format_modifiers(function);

    let name = &function.name;

    let args = function
        .args
        .iter()
        .map(|FunctionArg { name, ty }| format!(", {name}: {}", format_ident(ty, types)))
        .collect::<Vec<_>>()
        .join("");
    let raw_args = function
        .args
        .iter()
        .map(|FunctionArg { name, ty }| format!(", {name}: {}", format_raw_ident(ty, types)))
        .collect::<Vec<_>>()
        .join("");
    let wasm_args = function
        .args
        .iter()
        .map(|arg| format_wasm_ident(&arg.ty))
        .collect::<Vec<_>>();
    let wasm_args = if wasm_args.len() == 1 {
        let mut wasm_args = wasm_args;
        wasm_args.remove(0)
    } else {
        format!("({})", wasm_args.join(", "))
    };

    let return_type = match &function.return_type {
        Some(ty) => format_ident(ty, types),
        None => "()".to_owned(),
    };
    let raw_return_type = match &function.return_type {
        Some(ty) => format_raw_ident(ty, types),
        None => "()".to_owned(),
    };
    let wasm_return_type = match &function.return_type {
        Some(ty) => format_wasm_ident(ty),
        None => "()".to_owned(),
    };

    let serialize_args = function
        .args
        .iter()
        .filter(|arg| !arg.ty.is_primitive())
        .map(|FunctionArg { name, .. }| format!("let {name} = serialize_to_vec(&{name});"))
        .collect::<Vec<_>>()
        .join("\n");
    let serialize_raw_args = function
        .args
        .iter()
        .filter(|arg| !arg.ty.is_primitive())
        .map(|FunctionArg { name, .. }| format!("let {name} = export_to_guest_raw(&mut env.clone().into_mut(&mut *self.store.borrow_mut()), {name});"))
        .collect::<Vec<_>>()
        .join("\n");

    let arg_names = function
        .args
        .iter()
        .map(|arg| arg.name.as_ref())
        .collect::<Vec<_>>()
        .join(", ");
    let wasm_arg_names = function
        .args
        .iter()
        .map(|arg| format!("{}.to_abi()", arg.name))
        .collect::<Vec<_>>()
        .join(", ");

    let (raw_return_wrapper, return_wrapper) = if function.is_async {
        (
            "let result = ModuleRawFuture::new(env.clone().into_mut(&mut *self.store.borrow_mut()), result).await;".to_string(),
            "let result = result.await;\nlet result = result.map(|ref data| deserialize_from_slice(data));".to_string(),
        )
    } else if !function
        .return_type
        .as_ref()
        .map(TypeIdent::is_primitive)
        .unwrap_or(true)
    {
        (
            "let result = import_from_guest_raw(&mut env.clone().into_mut(&mut *self.store.borrow_mut()), result);".to_string(),
            "let result = result.map(|ref data| deserialize_from_slice(data));".to_string(),
        )
    } else {
        (
            "let result = WasmAbi::from_abi(result);".to_string(),
            "".to_string(),
        )
    };

    (
        doc,
        modifiers,
        name,
        args,
        raw_args,
        wasm_args,
        return_type,
        raw_return_type,
        wasm_return_type,
        serialize_args,
        serialize_raw_args,
        arg_names,
        wasm_arg_names,
        raw_return_wrapper,
        return_wrapper,
    )
}

fn format_import_function(function: &Function, types: &TypeMap) -> String {
    let (
        doc,
        modifiers,
        name,
        args,
        raw_args,
        wasm_args,
        return_type,
        raw_return_type,
        wasm_return_type,
        serialize_args,
        serialize_raw_args,
        arg_names,
        wasm_arg_names,
        raw_return_wrapper,
        return_wrapper,
    ) = generate_import_function_variables(function, types);

    format!(
        r#"{doc}pub {modifiers}fn {name}(&self{args}) -> Result<{return_type}, InvocationError> {{
    {serialize_args}
    let result = self.{name}_raw({arg_names});
    {return_wrapper}result
}}
pub {modifiers}fn {name}_raw(&self{raw_args}) -> Result<{raw_return_type}, InvocationError> {{
    let env = FunctionEnv::new(&mut *self.store.borrow_mut(), RuntimeInstanceData::default());
    let import_object = create_import_object(&mut *self.store.borrow_mut(), &env);
    let instance = Instance::new(&mut *self.store.borrow_mut(), &self.module, &import_object).unwrap();
    env.as_ref(&*self.store.borrow()).init_with_instance(&*self.store.borrow(), &instance);
    {serialize_raw_args}let function = instance
        .exports
        .get_typed_function::<{wasm_args}, {wasm_return_type}>(&*self.store.borrow(), "__fp_gen_{name}")
        .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_{name}".to_owned()))?;
    let result = function.call(&mut *self.store.borrow_mut(), {wasm_arg_names})?;
    {raw_return_wrapper}Ok(result)
}}"#
    )
}

pub(crate) fn format_import_arg(name: &str, ty: &TypeIdent, types: &TypeMap) -> String {
    if ty.is_primitive() {
        format!("let {name} = WasmAbi::from_abi({name});")
    } else {
        let ty = format_ident(ty, types);
        format!("let {name} = import_from_guest::<{ty}>(&mut env, {name});")
    }
}

pub(crate) fn format_export_function(function: &Function, types: &TypeMap) -> String {
    let name = &function.name;
    let wasm_args = function
        .args
        .iter()
        .map(|FunctionArg { name, ty }| format!(", {name}: {}", format_wasm_ident(ty)))
        .collect::<Vec<_>>()
        .join("");

    let wrapper_return_type = if function.is_async {
        " -> FatPtr".to_owned()
    } else {
        match &function.return_type {
            Some(ty) => format!(" -> {}", format_wasm_ident(ty)),
            None => "".to_owned(),
        }
    };

    let import_args = function
        .args
        .iter()
        .map(|arg| format_import_arg(&arg.name, &arg.ty, types))
        .collect::<Vec<_>>()
        .join("\n");

    let arg_names = function
        .args
        .iter()
        .map(|arg| arg.name.as_ref())
        .collect::<Vec<_>>()
        .join(", ");

    let return_wrapper = if function.is_async {
        r#"let async_ptr = create_future_value(&mut env);
    let handle = tokio::runtime::Handle::current();
    let result = handle.block_on(async move { result.await });
    let result_ptr = export_to_guest(&mut env, &result);
    env.data().guest_resolve_async_value().call(&mut env.as_store_mut(), async_ptr, result_ptr).unwrap();
    async_ptr"#
    } else {
        match &function.return_type {
            None => "",
            Some(ty) if ty.is_primitive() => "result.to_abi()",
            _ => "export_to_guest(&mut env, &result)",
        }
    };

    format!(
        r#"pub fn _{name}(mut env: FunctionEnvMut<RuntimeInstanceData>{wasm_args}){wrapper_return_type} {{
    {import_args}
    let result = super::{name}({arg_names});
    {return_wrapper}
}}"#
    )
}

fn generate_function_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: &TypeMap,
    path: &str,
) {
    let imports = import_functions
        .iter()
        .map(|function| format_export_function(function, types))
        .collect::<Vec<_>>()
        .join("\n\n");
    let exports = export_functions
        .iter()
        .map(|function| format_import_function(function, types))
        .collect::<Vec<_>>()
        .join("\n\n");
    let create_import_object_func = generate_create_import_object_func(&import_functions);
    format_function_bindings(imports, exports, create_import_object_func, path);
}

pub(crate) fn format_function_bindings(
    imports: String,
    exports: String,
    create_import_object_func: String,
    path: &str,
) {
    let full = rustfmt_wrapper::rustfmt(format!(r#"use super::types::*;
use fp_bindgen_support::{{
    common::{{mem::FatPtr, abi::WasmAbi}},
    host::{{
        errors::{{InvocationError, RuntimeError}},
        mem::{{export_to_guest, export_to_guest_raw, import_from_guest, import_from_guest_raw, deserialize_from_slice, serialize_to_vec}},
        r#async::{{create_future_value, future::ModuleRawFuture, resolve_async_value}},
        runtime::RuntimeInstanceData,
    }},
}};
use std::cell::RefCell;
use wasmer::{{imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports, Instance, Module, Store}};

pub struct Runtime {{
    store: RefCell<Store>,
    module: Module,
}}

impl Runtime {{
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {{
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        Ok(Self {{ store: RefCell::new(store), module }})
    }}

    fn default_store() -> wasmer::Store {{
        let compiler = wasmer::Cranelift::default();
        Store::new(compiler)
    }}

    {exports}
}}

{create_import_object_func}

{imports}
"#))
    .unwrap();
    write_bindings_file(format!("{}/bindings.rs", path), full);
}

pub(crate) fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
