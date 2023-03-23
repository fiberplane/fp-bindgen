use crate::{
    functions::{Function, FunctionList},
    generators::{
        rust_plugin::generate_type_bindings,
        rust_wasmer_runtime::{
            format_function_bindings, format_import_function, generate_export_function_variables,
        },
    },
    types::TypeMap,
};
use std::fs;

pub(crate) fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: TypeMap,
    path: &str,
) {
    fs::create_dir_all(path).expect("Could not create output directory");

    // We use the same type generation as for the Rust plugin, only with the
    // serializable and deserializable types inverted:
    generate_type_bindings(&types, path);

    generate_function_bindings(import_functions, export_functions, &types, path);
}

fn generate_create_import_object_func(import_functions: &FunctionList) -> String {
    let imports = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            format!(
                r#"namespace.insert(
            "__fp_gen_{name}",
            Function::new_native_with_env(store, env.clone(), _{name})
    );"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        r#"fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> wasmer::Exports {{
    let mut namespace = wasmer::Exports::new();
    namespace.insert(
            "__fp_host_resolve_async_value",
            Function::new_native_with_env(store, env.clone(), resolve_async_value)
    );
    {imports}
    namespace
}}"#
    )
}

fn format_export_function(function: &Function, types: &TypeMap) -> String {
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
    ) = generate_export_function_variables(function, types);

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
        let mut wasi_env = wasmer_wasi::WasiState::new("fp").finalize().unwrap();
        let mut import_object = wasi_env.import_object(&module).unwrap();
        let namespace = create_import_object(module.store(), &env);
        import_object.register("fp", namespace);
        let instance = Instance::new(&module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        Ok(Self { instance, env })
    }"#
    .to_string();
    let create_import_object_func = generate_create_import_object_func(&import_functions);
    format_function_bindings(imports, exports, new_func, create_import_object_func, path);
}
