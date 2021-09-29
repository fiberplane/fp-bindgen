use crate::functions::FunctionList;
use crate::generators::rust_plugin::{format_primitive, format_type, generate_type_bindings};
use crate::primitives::Primitive;
use crate::types::Type;
use std::collections::BTreeSet;
use std::fs;

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
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

    generate_function_bindings(import_functions, export_functions, &spec_path);

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
    path: &str,
) {
    let imports_map = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            format!(
                "            \"__fp_gen_{}\" => Function::new_native_with_env(store, env.clone(), _{}),",
                name, name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let imports = import_functions
        .iter()
        .map(|function| {
            let name = &function.name;
            let args_with_types = function
                .args
                .iter()
                .map(|arg| {
                    format!(
                        ", {}: {}",
                        arg.name,
                        match arg.ty {
                            Type::Primitive(primitive) => format_primitive(primitive),
                            _ => "FatPtr".to_owned(),
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join("");
            let import_args = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(_) => "".to_owned(),
                    _ => format!(
                        "    let {} = import_from_guest::<{}>(env, {});\n",
                        arg.name,
                        format_type(&arg.ty),
                        arg.name
                    ),
                })
                .collect::<Vec<_>>()
                .join("");
            let return_type = if function.is_async {
                " -> FatPtr".to_owned()
            } else {
                match &function.return_type {
                    Type::Unit => "".to_owned(),
                    ty => format!(
                        " -> {}",
                        match ty {
                            Type::Primitive(primitive) => format_primitive(*primitive),
                            _ => "FatPtr".to_owned(),
                        }
                    ),
                }
            };
            let args = function
                .args
                .iter()
                .map(|arg| arg.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let call_fn = if function.is_async {
                let call_async_fn = match &function.return_type {
                    Type::Unit => format!(
                        "super::{}({}).await;\n        let result_ptr = 0;",
                        name, args
                    ),
                    _ => format!(
                        "let result_ptr = export_to_guest(&env, &super::{}({}).await);",
                        name, args
                    ),
                };

                format!(
                    "let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {{
        {}

        unsafe {{
            env.__fp_guest_resolve_async_value
                .get_unchecked()
                .call(async_ptr, result_ptr)
                .expect(\"Runtime error: Cannot resolve async value\");
        }}
    }});

    async_ptr",
                    call_async_fn
                )
            } else {
                match &function.return_type {
                    Type::Unit => format!("super::{}({});", name, args),
                    Type::Primitive(_) => format!("super::{}({})", name, args),
                    _ => format!("export_to_guest(env, &super::{}({}))", name, args),
                }
            };
            format!(
                "pub fn _{}(env: &RuntimeInstanceData{}){} {{
{}{}    {}
}}",
                name,
                args_with_types,
                return_type,
                import_args,
                if import_args.is_empty() { "" } else { "\n" },
                call_fn
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let exports = export_functions
        .into_iter()
        .map(|function| {
            let name = function.name;
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
                        "        let {} = export_to_guest(&env, &{});\n",
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

        Ok(ModuleFuture::new(env.clone(), async_ptr).await)",
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

        Ok(import_from_guest(&env, ptr))",
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
                name,
                args_with_types,
                return_type,
                export_args,
                if export_args.is_empty() { "" } else { "\n" },
                name,
                call_and_return
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    write_bindings_file(
        format!("{}/bindings.rs", path),
        format!(
            "use super::types::*;
use crate::errors::InvocationError;
use crate::{{
    support::{{
        create_future_value, export_to_guest, import_from_guest, resolve_async_value,
        FatPtr, ModuleFuture,
    }},
    Runtime, RuntimeInstanceData,
}};
use wasmer::{{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv}};

impl Runtime {{
{}
}}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {{
    imports! {{
        \"fp\" => {{
            \"__fp_host_resolve_async_value\" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
{}
        }}
    }}
}}

{}
",
            exports, imports_map, imports,
        ),
    );
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
