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
    fs::create_dir_all(path).expect("Could not create output directory");

    generate_type_bindings(&types, path);

    generate_function_bindings(import_functions, export_functions, &types, path);
}

fn generate_create_import_object_func(import_functions: &FunctionList) -> String {
    let imports = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            format!(
                "\"__fp_gen_{name}\" => Function::new_native_with_env(store, env.clone(), _{name}),"
            )
        })
        .collect::<Vec<_>>()
        .join("\n            ");

    format!(
        r#"fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {{
    imports! {{
        "fp" => {{
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
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

pub(crate) struct ExportFunctionVariables<'a> {
    pub doc: String,
    pub modifiers: String,
    pub name: &'a str,
    pub args: String,
    pub raw_args: String,
    pub wasm_args: String,
    pub return_type: String,
    pub raw_return_type: String,
    pub wasm_return_type: String,
    pub serialize_args: String,
    pub serialize_raw_args: String,
    pub arg_names: String,
    pub wasm_arg_names: String,
    pub raw_return_wrapper: String,
    pub return_wrapper: String,
}

pub(crate) fn generate_export_function_variables<'a>(
    function: &'a Function,
    types: &TypeMap,
) -> ExportFunctionVariables<'a> {
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
    let raw_return_type = match (function.is_async, &function.return_type) {
        (true, _) => "Vec<u8>".to_owned(),
        (false, Some(ty)) => format_raw_ident(ty, types),
        (false, None) => "()".to_owned(),
    };
    let wasm_return_type = match (function.is_async, &function.return_type) {
        (true, _) => "FatPtr".to_owned(),
        (false, Some(ty)) => format_wasm_ident(ty),
        (false, None) => "()".to_owned(),
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
        .map(|FunctionArg { name, .. }| {
            format!("let {name} = export_to_guest_raw(&self.env, {name});")
        })
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
            "let result = ModuleRawFuture::new(self.env.clone(), result).await;".to_string(),
            format!("let result = result.await;\nlet result = result.map(|ref data| deserialize_from_slice::<{return_type}>(data));").to_string(),
        )
    } else if !function
        .return_type
        .as_ref()
        .map(TypeIdent::is_primitive)
        .unwrap_or(true)
    {
        (
            "let result = import_from_guest_raw(&self.env, result);".to_string(),
            "let result = result.map(|ref data| deserialize_from_slice(data));".to_string(),
        )
    } else {
        (
            "let result = WasmAbi::from_abi(result);".to_string(),
            "".to_string(),
        )
    };

    ExportFunctionVariables {
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
    }
}

fn format_export_function(function: &Function, types: &TypeMap) -> String {
    let ExportFunctionVariables {
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
    } = generate_export_function_variables(function, types);

    if function.is_async {
        format!(
            r#"{doc}pub {modifiers}fn {name}(&self{args}) -> Result<{return_type}, InvocationError> {{
    let this = self.clone();
    let task = async move {{
    {serialize_args}
    let result = this.{name}_raw({arg_names});
    {return_wrapper}result.unwrap()
    }};
    let mut recv = CURRENT_SPAWNER.spawn_async(task).await;
    match recv.recv().await {{
        Some(result) => Ok(*result.downcast::<{return_type}>().unwrap()),
        None => Err(InvocationError::UnexpectedReturnType),
    }}
}}
pub {modifiers}fn {name}_raw(&self{raw_args}) -> Result<{raw_return_type}, InvocationError> {{
    {serialize_raw_args}let function = self.instance
        .exports
        .get_native_function::<{wasm_args}, {wasm_return_type}>("__fp_gen_{name}")
        .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_{name}".to_owned()))?;
    let result = function.call({wasm_arg_names})?;
    {raw_return_wrapper}Ok(result)
}}"#
    )
    } else {
        format!(
            r#"{doc}pub {modifiers}fn {name}(&self{args}) -> Result<{return_type}, InvocationError> {{
    {serialize_args}
    let result = self.{name}_raw({arg_names});
    {return_wrapper}result
}}
pub {modifiers}fn {name}_raw(&self{raw_args}) -> Result<{raw_return_type}, InvocationError> {{
    {serialize_raw_args}let function = self.instance
        .exports
        .get_native_function::<{wasm_args}, {wasm_return_type}>("__fp_gen_{name}")
        .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_{name}".to_owned()))?;
    let result = function.call({wasm_arg_names})?;
    {raw_return_wrapper}Ok(result)
}}"#
    )
    }
}

pub(crate) fn format_import_arg(name: &str, ty: &TypeIdent, types: &TypeMap) -> String {
    if ty.is_primitive() {
        format!("let {name} = WasmAbi::from_abi({name});")
    } else {
        let ty = format_ident(ty, types);
        format!("let {name} = import_from_guest::<{ty}>(env, {name});")
    }
}

pub(crate) fn format_import_function(function: &Function, types: &TypeMap) -> String {
    let name = &function.name;
    let wasm_args = function
        .args
        .iter()
        .map(|FunctionArg { name, ty }| format!(", {name}: {}", format_wasm_ident(ty)))
        .collect::<Vec<_>>()
        .join("");

    let wrapper_return_type = match (function.is_async, &function.return_type) {
        (true, _) => " -> FatPtr".to_owned(),
        (false, Some(ty)) => format!(" -> {}", format_wasm_ident(ty)),
        (false, None) => "".to_owned(),
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
        let return_type = match &function.return_type {
            None => "()".to_string(),
            Some(ty) => format!("{}", ty),
        };
        format!(
            r#"let env_clone = env.clone();
    let async_ptr = create_future_value(&env);
    let host_task = GLOBAL_HANDLER.get().unwrap().spawn_async(async move {{
        super::{name}({arg_names}).await
    }});

    let guest_task = async move {{
        let mut result = host_task.await;
        let res = match result.recv().await {{
            Some(result) => *result.downcast::<{return_type}>().unwrap(),
            None => panic!("FXIME: host_task return error"),
        }};

        let result_ptr = export_to_guest(&env_clone, &res);
        env_clone.guest_resolve_async_value(async_ptr, result_ptr);
    }};
    CURRENT_SPAWNER.spawn(guest_task);
    async_ptr"#
        )
    } else {
        match &function.return_type {
            None => format!("super::{name}({arg_names})"),
            Some(ty) if ty.is_primitive() => format!("super::{name}({arg_names}).to_abi()"),
            _ => format!("export_to_guest(env, &super::{name}({arg_names}))"),
        }
    };

    format!(
        r#"pub fn _{name}(env: &RuntimeInstanceData{wasm_args}){wrapper_return_type} {{
    {import_args}
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
        .map(|function| format_import_function(function, types))
        .collect::<Vec<_>>()
        .join("\n\n");
    let exports = export_functions
        .iter()
        .map(|function| format_export_function(function, types))
        .collect::<Vec<_>>()
        .join("\n\n");
    let new_func = r#"pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(module.store(), &env);
        let instance = Instance::new(&module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        Ok(Self { instance, env })
    }"#
    .to_string();
    let create_import_object_func = generate_create_import_object_func(&import_functions);

    write_bindings_file(
        format!("{path}/bindings.rs"),
        format_function_bindings(imports, exports, new_func, create_import_object_func),
    );
}

pub(crate) fn format_function_bindings(
    imports: String,
    exports: String,
    new_func: String,
    create_import_object_func: String,
) -> String {
    rustfmt_wrapper::rustfmt(format!(r#"#![allow(clippy::let_and_return, unused)]
use super::types::*;
use fp_bindgen_support::{{
    common::{{mem::FatPtr, abi::WasmAbi}},
    wasmer2_host::{{
        errors::{{InvocationError, RuntimeError}},
        mem::{{export_to_guest, export_to_guest_raw, import_from_guest, import_from_guest_raw, deserialize_from_slice, serialize_to_vec}},
        r#async::{{create_future_value, future::ModuleRawFuture, resolve_async_value}},
        runtime::RuntimeInstanceData,
    }},
}};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use wasmer::{{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv}};
use fp_bindgen_support::wasmer2_host::task_spawner::{{CurrentThreadSpawner, GlobalSpawner}};
use fp_bindgen_support::wasmer2_host::task_spawner::GLOBAL_HANDLER;
static CURRENT_SPAWNER: Lazy<CurrentThreadSpawner> = Lazy::new(|| {{
    CurrentThreadSpawner::new()
}});

#[derive(Clone)]
pub struct Runtime {{
    instance: Instance,
    env: RuntimeInstanceData,
}}

impl Runtime {{
    {new_func}

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn default_store() -> wasmer::Store {{
        let compiler = wasmer::Cranelift::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }}

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn default_store() -> wasmer::Store {{
        let compiler = wasmer::Singlepass::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }}

    {exports}
}}

{create_import_object_func}

{imports}
"#))
    .unwrap()
}

pub(crate) fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(file_path, &contents).expect("Could not write bindings file");
}
