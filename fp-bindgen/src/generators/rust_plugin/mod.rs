use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::types::{Field, Type, Variant};
use std::collections::BTreeSet;
use std::fs;

enum SerializationRequirements {
    Serialize,
    Deserialize,
    Both,
}

impl SerializationRequirements {
    pub fn from_sets(
        ty: &Type,
        serializable_types: &BTreeSet<Type>,
        deserializable_types: &BTreeSet<Type>,
    ) -> Self {
        let needs_serialization = serializable_types.contains(ty);
        let needs_deserialization = deserializable_types.contains(ty);
        match (needs_serialization, needs_deserialization) {
            (true, true) => SerializationRequirements::Both,
            (true, false) => SerializationRequirements::Serialize,
            (false, true) => SerializationRequirements::Deserialize,
            _ => panic!("Type cannot be (de)serialized: {:?}", ty),
        }
    }
}

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    path: &str,
) {
    let requires_async = import_functions.iter().any(|function| function.is_async);

    generate_type_bindings(serializable_types, deserializable_types, path);
    generate_function_bindings(import_functions, export_functions, path);

    write_bindings_file(
        format!("{}/support.rs", path),
        include_bytes!("assets/support.rs"),
    );

    if requires_async {
        write_bindings_file(
            format!("{}/async.rs", path),
            include_bytes!("assets/async.rs"),
        );
        write_bindings_file(
            format!("{}/queue.rs", path),
            include_bytes!("assets/queue.rs"),
        );
        write_bindings_file(
            format!("{}/task.rs", path),
            include_bytes!("assets/task.rs"),
        );
    }

    write_bindings_file(
        format!("{}/mod.rs", path),
        format!(
            "{}mod functions;
{}mod support;
{}mod types;

pub use functions::*;
{}pub use support::{{__fp_free, __fp_malloc}};
pub use types::*;
",
            if requires_async { "mod r#async;\n" } else { "" },
            if requires_async { "mod queue;\n" } else { "" },
            if requires_async { "mod task;\n" } else { "" },
            if requires_async {
                "pub use r#async::__fp_guest_resolve_async_value;\n"
            } else {
                ""
            }
        ),
    );
}

pub fn generate_type_bindings(
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    path: &str,
) {
    let mut all_types = serializable_types.clone();
    all_types.append(&mut deserializable_types.clone());

    let type_defs = all_types
        .into_iter()
        .filter_map(|ty| {
            let serde_reqs = SerializationRequirements::from_sets(
                &ty,
                &serializable_types,
                &deserializable_types,
            );
            match ty {
                Type::Enum(name, variants) => {
                    Some(create_enum_definition(name, variants, &serde_reqs))
                }
                Type::Struct(name, fields) => {
                    Some(create_struct_definition(name, fields, &serde_reqs))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    write_bindings_file(
        format!("{}/types.rs", path),
        format!("use serde::{{Deserialize, Serialize}};\n\n{}\n", type_defs),
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
                None => "".to_owned(),
                Some(ty) => format!(
                    " -> {}",
                    match ty {
                        Type::Primitive(primitive) => format_primitive(*primitive),
                        _ => "FatPtr".to_owned(),
                    }
                ),
            };
            format!("    fn __gen_{}({}){};", function.name, args, return_type)
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
                None => "".to_owned(),
                Some(ty) => format!(" -> {}", format_type(ty)),
            };
            let export_args = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(_) => "".to_owned(),
                    _ => format!(
                        "    let {} = export_value_to_host({});\n",
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
                None => format!("__gen_{}({});", name, args),
                Some(Type::Primitive(_)) => format!("__gen_{}({})", name, args),
                Some(_) => format!("let ret = __gen_{}({});", name, args),
            };
            let import_return_value = match &function.return_type {
                None | Some(Type::Primitive(_)) => "",
                Some(_) => {
                    if function.is_async {
                        "        let result_ptr = HostFuture::new(ret).await;\n        import_value_from_host(result_ptr)\n"
                    } else {
                        "        import_value_from_host(ret)\n"
                    }
                }
            };
            format!(
                "{}pub {}fn {}({}){} {{\n{}    unsafe {{\n        {}\n{}    }}\n}}",
                doc,
                modifiers,
                name,
                args_with_types,
                return_type,
                export_args,
                call_fn,
                import_return_value
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let macro_rules = export_functions
        .into_iter()
        .map(|function| {
            let name = function.name;
            let modifiers = if function.is_async { "async " } else { "" };
            let has_return_value = function.return_type.is_some();
            let args_with_ptr_types = function
                .args
                .iter()
                .map(|arg| {
                    let ty = match &arg.ty {
                        Type::Primitive(primitive) => format_primitive(*primitive),
                        _ => "FatPtr".to_owned(),
                    };
                    format!("{}: {}", arg.name, ty)
                })
                .collect::<Vec<_>>()
                .join(", ");
            let import_args = function
                .args
                .iter()
                .map(|arg| match &arg.ty {
                    Type::Primitive(_) => "".to_owned(),
                    ty => format!(
                        "let {} = unsafe {{ import_value_from_host::<{}>({}) }};",
                        arg.name,
                        format_type(ty),
                        arg.name
                    ),
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
                    "let len = size_of::<r#async::AsyncValue>() as u32;".to_owned(),
                    "let ptr = malloc(len);".to_owned(),
                    "let fat_ptr = to_fat_ptr(ptr, len);".to_owned(),
                    "let ptr = ptr as *mut r#async::AsyncValue;".to_owned(),
                    "".to_owned(),
                    "task::Task::spawn(Box::pin(async move {".to_owned(),
                ];

                async_body.append(
                    &mut import_args
                        .iter()
                        .map(|import_arg| format!("    {}", import_arg))
                        .collect(),
                );

                // Call the actual async function:
                async_body.push(match &function.return_type {
                    None => format!("    {}({}).await;", name, args),
                    Some(_) => format!("    let ret = {}({}).await;", name, args),
                });

                async_body.push("    unsafe {".to_owned());

                // If there is a return type, put the result in the `AsyncValue`
                // referenced by `ptr`:
                match &function.return_type {
                    None => {}
                    Some(ty) => {
                        async_body.append(&mut vec![
                            "        let (result_ptr, result_len) =".to_owned(),
                            format!(
                                "            from_fat_ptr(export_value_to_host::<{}>(&ret));",
                                format_type(ty)
                            ),
                            "        (*ptr).ptr = result_ptr as u32;".to_owned(),
                            "        (*ptr).len = result_len;".to_owned(),
                        ]);
                    }
                };

                async_body.append(&mut vec![
                    // We're done, notify the host:
                    "        (*ptr).status = STATUS_READY;".to_owned(),
                    "        __fp_host_resolve_async_value(fat_ptr);".to_owned(),
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
                    None => format!("{}({});", name, args),
                    Some(Type::Primitive(_)) => format!("{}({})", name, args),
                    Some(_) => format!("let ret = {}({});", name, args),
                });
                match &function.return_type {
                    None | Some(Type::Primitive(_)) => {}
                    Some(ty) => {
                        body.push(format!("export_value_to_host::<{}>(ret)", format_type(ty)))
                    }
                };
                body
            };

            format!(
                "    ({}fn {}($($param:ident: $ty:ty),+){} $body:block) => {{
        #[no_mangle]
        pub __fp_gen_{}({}){} {{
{}
        }}

        {}fn {}($($param: $ty),+){} $body
    }};",
                modifiers,
                name,
                if has_return_value { " -> $ret:ty" } else { "" },
                name,
                args_with_ptr_types,
                if function.is_async {
                    " -> FatPtr"
                } else {
                    match &function.return_type {
                        None => "",
                        Some(Type::Primitive(_)) => " -> $ret",
                        Some(_) => " -> FatPtr",
                    }
                },
                body.iter()
                    .map(|line| format!("            {}", line))
                    .collect::<Vec<_>>()
                    .join("\n"),
                modifiers,
                name,
                if has_return_value { " -> $ret" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let export_macro = format!("macro_rules! fp_export {{\n{}\n}}", macro_rules);

    write_bindings_file(
        format!("{}/functions.rs", path),
        format!(
            "use super::support::*;\n\
            use super::types::*;\n\
            \n\
            #[link(wasm_import_module = \"fp\")]\n\
            extern \"C\" {{\n\
                {}\n\
            }}\n\
            \n\
            {}\n\
            \n\
            {}\n",
            extern_decls, fn_defs, export_macro
        ),
    );
}

fn create_enum_definition(
    name: String,
    variants: Vec<Variant>,
    serde_reqs: &SerializationRequirements,
) -> String {
    "TODO".to_owned() // TODO
}

fn create_struct_definition(
    name: String,
    fields: Vec<Field>,
    serde_reqs: &SerializationRequirements,
) -> String {
    let derives = match serde_reqs {
        SerializationRequirements::Serialize => "Serialize",
        SerializationRequirements::Deserialize => "Deserialize",
        SerializationRequirements::Both => "Serialize, Deserialize",
    };
    let fields = fields
        .into_iter()
        .map(|field| format!("    {}: {},", field.name, format_type(&field.ty)))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "#[derive(Clone, Debug, PartialEq, {})]\n\
        #[serde(rename_all = \"camelCase\")]\n\
        pub struct {} {{\n\
            {}\n\
        }}",
        derives, name, fields
    )
}

/// Formats a type so it's valid Rust again.
fn format_type(ty: &Type) -> String {
    match ty {
        Type::Enum(name, _) => name.clone(),
        Type::List(name, ty) => format!("{}<{}>", name, format_type(ty)),
        Type::Map(name, k, v) => format!("{}<{}, {}>", name, format_type(k), format_type(v)),
        Type::Option(ty) => format!("Option<{}>", format_type(ty)),
        Type::Primitive(primitive) => format_primitive(*primitive),
        Type::Struct(name, _) => name.clone(),
    }
}

fn format_primitive(primitive: Primitive) -> String {
    let string = match primitive {
        Primitive::Bool => "bool",
        Primitive::F32 => "f32",
        Primitive::F64 => "f64",
        Primitive::I8 => "i8",
        Primitive::I16 => "i16",
        Primitive::I32 => "i32",
        Primitive::I64 => "i64",
        Primitive::I128 => "i128",
        Primitive::String => "String",
        Primitive::U8 => "u8",
        Primitive::U16 => "u16",
        Primitive::U32 => "u32",
        Primitive::U64 => "u64",
        Primitive::U128 => "u128",
    };
    string.to_owned()
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
