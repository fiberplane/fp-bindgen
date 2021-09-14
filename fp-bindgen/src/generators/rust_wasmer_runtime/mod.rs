use crate::functions::FunctionList;
use crate::generators::rust_plugin::{format_primitive, format_type, generate_type_bindings};
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
    // We use the same type generation as for the Rust plugin, only with the
    // serializable and deserializable types inverted:
    generate_type_bindings(
        deserializable_types,
        serializable_types,
        &format!("{}/spec", path),
    );

    generate_function_bindings(
        import_functions,
        export_functions,
        &format!("{}/spec", path),
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
    path: &str,
) {
    let extern_decls = import_functions
        .iter()
        .map(|function| {
            let args = function
                .args
                .iter()
                .map(|arg| {
                    format!(
                        "{}: {}",
                        arg.name,
                        match arg.ty {
                            Type::Primitive(primitive) => format_primitive(primitive),
                            _ => "FatPtr".to_owned(),
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &function.return_type {
                Type::Unit => "".to_owned(),
                ty => format!(
                    " -> {}",
                    match ty {
                        Type::Primitive(primitive) => format_primitive(*primitive),
                        _ => "FatPtr".to_owned(),
                    }
                ),
            };
            format!(
                "    fn __fp_gen_{}({}){};",
                function.name, args, return_type
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let fn_defs = import_functions
        .into_iter()
        .map(|function| {
            let name = function.name;
            let doc = function
                .doc_lines
                .iter()
                .map(|line| format!("///{}\n", line))
                .collect::<Vec<_>>()
                .join("");
            let modifiers = if function.is_async { "async " } else { "" };
            let args_with_types = function
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name, format_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &function.return_type {
                Type::Unit => "".to_owned(),
                ty => format!(" -> {}", format_type(ty)),
            };
            let export_args = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(_) => "".to_owned(),
                    _ => format!(
                        "    let {} = export_value_to_host(&{});\n",
                        arg.name, arg.name
                    ),
                })
                .collect::<Vec<_>>()
                .join("");
            let args = function
                .args
                .iter()
                .map(|arg| arg.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let call_fn = match &function.return_type {
                Type::Unit => format!("__fp_gen_{}({});", name, args),
                Type::Primitive(_) => format!("__fp_gen_{}({})", name, args),
                _ => format!("let ret = __fp_gen_{}({});", name, args),
            };
            let import_return_value = match &function.return_type {
                Type::Unit | Type::Primitive(_) => "",
                _ => {
                    if function.is_async {
                        "        let result_ptr = HostFuture::new(ret).await;\n        import_value_from_host(result_ptr)\n"
                    } else {
                        "        import_value_from_host(ret)\n"
                    }
                }
            };
            let call_and_return = if import_return_value.is_empty() {
                format!("unsafe {{ {} }}", call_fn)
            } else {
                format!("unsafe {{\n        {}\n{}    }}", call_fn, import_return_value)
            };
            format!(
                "{}pub {}fn {}({}){} {{\n{}    {}\n}}",
                doc,
                modifiers,
                name,
                args_with_types,
                return_type,
                export_args,
                call_and_return
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let macro_rules = export_functions
        .into_iter()
        .map(|function| {
            let name = function.name;
            let modifiers = if function.is_async { "async " } else { "" };
            let has_return_value = function.return_type != Type::Unit;
            let args_with_ptr_types = function
                .args
                .iter()
                .map(|arg| {
                    let ty = match &arg.ty {
                        Type::Primitive(primitive) => format_primitive(*primitive),
                        _ => "_FP_FatPtr".to_owned(),
                    };
                    format!("{}: {}", arg.name, ty)
                })
                .collect::<Vec<_>>()
                .join(", ");
            let import_args = function
                .args
                .iter()
                .filter_map(|arg| match &arg.ty {
                    Type::Primitive(_) => None,
                    ty => Some(format!(
                        "let {} = unsafe {{ _fp_import_value_from_host::<{}>({}) }};",
                        arg.name,
                        format_type(ty),
                        arg.name
                    )),
                })
                .collect::<Vec<_>>();
            let args = function
                .args
                .iter()
                .map(|arg| arg.name.clone())
                .collect::<Vec<_>>()
                .join(", ");

            let body = if function.is_async {
                // Set up the `AsyncValue` to be synchronously returned and spawn a task
                // to execute the async function:
                let mut async_body = vec![
                    "let len = std::mem::size_of::<_FP_AsyncValue>() as u32;".to_owned(),
                    "let ptr = _fp_malloc(len);".to_owned(),
                    "let fat_ptr = _fp_to_fat_ptr(ptr, len);".to_owned(),
                    "let ptr = ptr as *mut _FP_AsyncValue;".to_owned(),
                    "".to_owned(),
                    "_FP_Task::spawn(Box::pin(async move {".to_owned(),
                ];

                async_body.append(
                    &mut import_args
                        .iter()
                        .map(|import_arg| format!("    {}", import_arg))
                        .collect(),
                );

                // Call the actual async function:
                async_body.push(match &function.return_type {
                    Type::Unit => format!("    {}({}).await;", name, args),
                    _ => format!("    let ret = {}({}).await;", name, args),
                });

                async_body.push("    unsafe {".to_owned());

                // If there is a return type, put the result in the `AsyncValue`
                // referenced by `ptr`:
                if has_return_value {
                    async_body.append(&mut vec![
                        "        let (result_ptr, result_len) =".to_owned(),
                        format!(
                            "            _fp_from_fat_ptr(_fp_export_value_to_host::<{}>(&ret));",
                            format_type(&function.return_type)
                        ),
                        "        (*ptr).ptr = result_ptr as u32;".to_owned(),
                        "        (*ptr).len = result_len;".to_owned(),
                    ]);
                }

                async_body.append(&mut vec![
                    // We're done, notify the host:
                    "        (*ptr).status = 1;".to_owned(), // 1 = STATUS_READY
                    "        _fp_host_resolve_async_value(fat_ptr);".to_owned(),
                    "    }".to_owned(),
                    "}));".to_owned(),
                    "".to_owned(),
                    // The `fat_ptr` is returned synchronously:
                    "fat_ptr".to_owned(),
                ]);

                async_body
            } else {
                let mut body = import_args;
                body.push(match &function.return_type {
                    Type::Unit => format!("{}({});", name, args),
                    Type::Primitive(_) => format!("{}({})", name, args),
                    _ => format!("let ret = {}({});", name, args),
                });
                match &function.return_type {
                    Type::Unit | Type::Primitive(_) => {}
                    ty => body.push(format!(
                        "_fp_export_value_to_host::<{}>(&ret)",
                        format_type(ty)
                    )),
                }
                body
            };

            format!(
                "    ({}fn {}($($param:ident: $ty:ty),*){} $body:block) => {{
        #[no_mangle]
        pub fn __fp_gen_{}({}){} {{
{}
        }}

        {}fn {}($($param: $ty),*){} $body
    }};",
                modifiers,
                name,
                if has_return_value { " -> $ret:ty" } else { "" },
                name,
                args_with_ptr_types,
                if function.is_async {
                    " -> _FP_FatPtr"
                } else {
                    match &function.return_type {
                        Type::Unit => "",
                        Type::Primitive(_) => " -> $ret",
                        _ => " -> _FP_FatPtr",
                    }
                },
                body.iter()
                    .map(|line| if line.is_empty() {
                        "".to_owned()
                    } else {
                        format!("            {}", line)
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                modifiers,
                name,
                if has_return_value { " -> $ret" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    write_bindings_file(
        format!("{}/bindings.rs", path),
        format!(
            "use super::types::*;
use crate::{{
    support::{{
        assign_async_value, create_future_value, export_to_host, import_from_guest,
        resolve_async_value, FatPtr, ModuleFuture, FUTURE_STATUS_READY,
    }},
    Runtime, RuntimeInstanceData,
}};
use wasmer::{{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv}};

impl Runtime {{
    {}
}}
",
        ),
    );
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
