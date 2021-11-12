use crate::functions::{Function, FunctionList};
use crate::generators::rust_plugin::{format_raw_type, format_type, generate_type_bindings};
use crate::primitives::Primitive;
use crate::types::Type;
use crate::WasmerRuntimeConfig;
use std::collections::BTreeSet;
use std::fs;
use std::iter;

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    runtime_config: WasmerRuntimeConfig,
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
        runtime_config.generate_raw_export_wrappers,
        runtime_config.plugin_crate_name,
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

pub fn generate_function_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    generate_raw_export_wrappers: bool,
    plugin_crate_name: &str,
    path: &str,
) {
    let imports_map = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            format!(
                "            \"__fp_gen_{0}\" => Function::new_native_with_env(store, env.clone(), super::__fp_gen_{0}),",
                name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let exports = export_functions
        .iter()
        .map(export_function)
        .collect::<Vec<_>>()
        .join("\n\n");
    let raw_exports = if generate_raw_export_wrappers {
        // add a newline between the raw exports and the exports
        iter::once("".to_string())
            .chain(export_functions.iter().map(export_raw_function))
            .collect::<Vec<_>>()
            .join("\n\n")
    } else {
        String::new()
    };

    write_bindings_file(
        format!("{}/bindings.rs", path),
        format!(
            "use {}::*;
use crate::errors::InvocationError;
use crate::{{
    support::{{
        create_future_value, export_value_to_guest, export_to_guest_raw, import_value_from_guest,
        resolve_async_value, FatPtr, ModuleRawFuture,
    }},
    Runtime, RuntimeInstanceData,
}};
use wasmer::{{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv}};

impl Runtime {{
{}{}
}}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {{
    imports! {{
        \"fp\" => {{
            \"__fp_host_resolve_async_value\" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
{}
        }}
    }}
}}
",
            plugin_crate_name, exports, raw_exports, imports_map,
        ),
    );
}

fn export_function(function: &Function) -> String {
    let doc = function
        .doc_lines
        .iter()
        .map(|line| format!("    ///{}\n", line))
        .collect::<Vec<_>>()
        .join("");
    let modifiers = if function.is_async { "async " } else { "" };
    let args_with_types = function
        .args
        .iter()
        .map(|arg| format!(", {}: {}", arg.name, format_type(&arg.ty)))
        .collect::<Vec<_>>()
        .join("");
    let return_type = format!(
        " -> Result<{}, InvocationError>",
        format_type(&function.return_type)
    );
    let export_args = function
        .args
        .iter()
        .map(|arg| match &arg.ty {
            Type::Primitive(_) => "".to_owned(),
            _ => format!(
                "        let {} = export_value_to_guest(&env, &{});\n",
                arg.name, arg.name
            ),
        })
        .collect::<Vec<_>>()
        .join("");
    let args = function
        .args
        .iter()
        .map(|arg| format!("{}.into()", arg.name))
        .collect::<Vec<_>>()
        .join(", ");
    let call_and_return = if function.is_async {
        format!(
            "let result = function.call(&[{}])?;

        let async_ptr: FatPtr = match result[0] {{
            Value::I64(v) => unsafe {{ std::mem::transmute(v) }},
            _ => return Err(InvocationError::UnexpectedReturnType),
        }};

        let raw_result = ModuleRawFuture::new(env.clone(), async_ptr).await;
        Ok(rmp_serde::from_slice(&raw_result).unwrap())",
            args
        )
    } else {
        match function.return_type {
            Type::Unit => format!("function.call(&[{}])?;", args),
            Type::Primitive(primitive) => {
                use Primitive::*;
                let transmute = match primitive {
                    Bool => "Value::I32(v) => v as bool",
                    F32 => "Value::F32(v) => v",
                    F64 => "Value::F64(v) => v",
                    I8 => "Value::I32(v) => v as i8",
                    I16 => "Value::I32(v) => v as i16",
                    I32 => "Value::I32(v) => v",
                    I64 => "Value::I64(v) => v",
                    U8 => "Value::I32(v) => v as u8",
                    U16 => "Value::I32(v) => v as u16",
                    U32 => "Value::I32(v) => unsafe { std::mem::transmute(v) }",
                    U64 => "Value::I64(v) => unsafe { std::mem::transmute(v) }",
                };

                format!(
                    "let result = function.call(&[{}])?;

        match result[0] {{
            {},
            _ => return Err(InvocationError::UnexpectedReturnType),
        }}",
                    args, transmute
                )
            }
            _ => format!(
                "let result = function.call(&[{}])?;

        let ptr: FatPtr = match result[0] {{
            Value::I64(v) => unsafe {{ std::mem::transmute(v) }},
            _ => return Err(InvocationError::UnexpectedReturnType),
        }};

        Ok(import_value_from_guest(&env, ptr))",
                args
            ),
        }
    };
    format!(
        "{}    pub {}fn {}(&self{}){} {{
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

{}{}        let function = instance
            .exports
            .get_function(\"__fp_gen_{}\")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        {}
    }}",
        doc,
        modifiers,
        function.name,
        args_with_types,
        return_type,
        export_args,
        if export_args.is_empty() { "" } else { "\n" },
        function.name,
        call_and_return
    )
}

fn export_raw_function(function: &Function) -> String {
    let doc = function
        .doc_lines
        .iter()
        .map(|line| format!("    ///{}\n", line))
        .collect::<Vec<_>>()
        .join("");
    let modifiers = if function.is_async { "async " } else { "" };
    let args_with_types = function
        .args
        .iter()
        .map(|arg| format!(", {}: {}", arg.name, format_raw_type(&arg.ty)))
        .collect::<Vec<_>>()
        .join("");
    let return_type = format!(
        " -> Result<{}, InvocationError>",
        format_raw_type(&function.return_type)
    );
    let export_args = function
        .args
        .iter()
        .map(|arg| match &arg.ty {
            Type::Primitive(_) => "".to_owned(),
            _ => format!(
                "        let {} = export_to_guest_raw(&env, {});\n",
                arg.name, arg.name
            ),
        })
        .collect::<Vec<_>>()
        .join("");
    let args = function
        .args
        .iter()
        .map(|arg| format!("{}.into()", arg.name))
        .collect::<Vec<_>>()
        .join(", ");
    let call_and_return = if function.is_async {
        format!(
            "let result = function.call(&[{}])?;

        let async_ptr: FatPtr = match result[0] {{
            Value::I64(v) => unsafe {{ std::mem::transmute(v) }},
            _ => return Err(InvocationError::UnexpectedReturnType),
        }};

        Ok(ModuleRawFuture::new(env.clone(), async_ptr).await)",
            args
        )
    } else {
        match function.return_type {
            Type::Unit => format!("function.call(&[{}])?;", args),
            Type::Primitive(primitive) => {
                use Primitive::*;
                let transmute = match primitive {
                    Bool => "Value::I32(v) => v as bool",
                    F32 => "Value::F32(v) => v",
                    F64 => "Value::F64(v) => v",
                    I8 => "Value::I32(v) => v as i8",
                    I16 => "Value::I32(v) => v as i16",
                    I32 => "Value::I32(v) => v",
                    I64 => "Value::I64(v) => v",
                    U8 => "Value::I32(v) => v as u8",
                    U16 => "Value::I32(v) => v as u16",
                    U32 => "Value::I32(v) => unsafe { std::mem::transmute(v) }",
                    U64 => "Value::I64(v) => unsafe { std::mem::transmute(v) }",
                };

                format!(
                    "let result = function.call(&[{}])?;

        match result[0] {{
            {},
            _ => return Err(InvocationError::UnexpectedReturnType),
        }}",
                    args, transmute
                )
            }
            _ => format!(
                "let result = function.call(&[{}])?;

        let ptr: FatPtr = match result[0] {{
            Value::I64(v) => unsafe {{ std::mem::transmute(v) }},
            _ => return Err(InvocationError::UnexpectedReturnType),
        }};

        Ok(import_from_guest_raw(&env, ptr))",
                args
            ),
        }
    };
    format!(
        "{}    pub {}fn {}_raw(&self{}){} {{
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

{}{}        let function = instance
            .exports
            .get_function(\"__fp_gen_{}\")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        {}
    }}",
        doc,
        modifiers,
        function.name,
        args_with_types,
        return_type,
        export_args,
        if export_args.is_empty() { "" } else { "\n" },
        function.name,
        call_and_return
    )
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
