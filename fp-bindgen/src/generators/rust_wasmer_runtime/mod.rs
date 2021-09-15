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
    let spec_path = format!("{}/spec", path);
    fs::create_dir_all(&spec_path).expect("Could not create spec/ directory");

    // We use the same type generation as for the Rust plugin, only with the
    // serializable and deserializable types inverted:
    generate_type_bindings(deserializable_types, serializable_types, &spec_path);

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
                "            \"{}\" => Function::new_native_with_env(store, env.clone(), _{}),",
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
            let args = function
                .args
                .iter()
                .map(|arg| arg.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let call_fn = match &function.return_type {
                Type::Unit => format!("super::{}({});", name, args),
                Type::Primitive(_) => format!("super::{}({})", name, args),
                _ => format!("export_to_host(env, &super::{}({}))", name, args),
            };
            format!(
                "pub fn _{}(env: &RuntimeInstanceData{}){} {{
{}    {}
}}",
                name, args_with_types, return_type, import_args, call_fn
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let exports = export_functions
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

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {{
    imports! {{
        \"fp\" => {{
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
