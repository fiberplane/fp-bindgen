use crate::generators::rust_wasmer_runtime::format_import_function;
use crate::{
    functions::FunctionList,
    generators::{
        rust_plugin::generate_type_bindings,
        rust_wasmer_runtime::{format_export_function, format_function_bindings},
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

fn generate_create_imports_func(import_functions: &FunctionList) -> String {
    let imports = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            format!(
                r#"namespace.insert(
            "__fp_gen_{name}",
            Function::new_typed_with_env(store, env, _{name})
    );"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        r#"fn create_imports(store: &mut Store, env: &FunctionEnv<Arc<RuntimeInstanceData>>) -> wasmer::Exports {{
    let mut namespace = wasmer::Exports::new();
    namespace.insert(
            "__fp_host_resolve_async_value",
            Function::new_typed_with_env(store, env, resolve_async_value)
    );
    {imports}
    namespace
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
    let new_func = r#"pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let mut store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        let env = FunctionEnv::new(&mut store, Arc::new(RuntimeInstanceData::default()));
        let mut wasi_env = wasmer_wasi::WasiState::new("fp").finalize(&mut store).unwrap();
        let mut import_object = wasi_env.import_object(&mut store, &module).unwrap();
        import_object.register_namespace("fp", create_imports(&mut store, &env));
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();
        wasi_env.initialize(&mut store, &instance).unwrap();
        let env_from_instance = RuntimeInstanceData::from_instance(&mut store, &instance);
        Arc::get_mut(env.as_mut(&mut store)).unwrap().copy_from(env_from_instance);
        Ok(Self { store, instance, env })
    }"#
    .to_string();
    let create_imports_func = generate_create_imports_func(&import_functions);
    format_function_bindings(imports, exports, new_func, create_imports_func, path);
}
