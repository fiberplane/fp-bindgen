use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::types::{
    format_name_with_generics, EnumOptions, Field, GenericArgument, StructOptions, Type, Variant,
};
use std::collections::BTreeSet;
use std::fs;

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    path: &str,
) {
    let requires_async = import_functions.iter().any(|function| function.is_async);

    generate_type_bindings(
        serializable_types,
        deserializable_types,
        path,
        "rust_plugin",
    );
    generate_function_bindings(import_functions, export_functions, path, requires_async);

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
            "{}pub mod functions;
{}mod support;
{}pub mod types;

pub mod __fp_macro {{
{}    pub use super::support::{{
        FatPtr, __fp_free, __fp_malloc, export_value_to_host, from_fat_ptr, import_value_from_host,
        malloc, to_fat_ptr,
    }};
{}}}

pub mod prelude {{
    pub use super::__fp_macro;
    pub use super::functions::*;
    pub use super::types::*;
}}
",
            if requires_async { "mod r#async;\n" } else { "" },
            if requires_async { "mod queue;\n" } else { "" },
            if requires_async { "mod task;\n" } else { "" },
            if requires_async {
                "    pub use super::r#async::{AsyncValue, __fp_guest_resolve_async_value};\n"
            } else {
                ""
            },
            if requires_async {
                "    pub use super::task::Task;\n"
            } else {
                ""
            },
        ),
    );
}

pub fn generate_type_bindings(
    serializable_types: BTreeSet<Type>,
    mut deserializable_types: BTreeSet<Type>,
    path: &str,
    module_key: &str,
) {
    let mut all_types = serializable_types;
    all_types.append(&mut deserializable_types);

    let std_types = all_types
        .iter()
        .flat_map(|ty| collect_std_types(ty))
        .collect::<BTreeSet<_>>();
    let std_imports = if std_types.is_empty() {
        "".to_owned()
    } else if std_types.len() == 1 {
        format!("use std::{};\n", std_types.iter().next().unwrap())
    } else {
        format!(
            "use std::{{{}}};\n",
            std_types.into_iter().collect::<Vec<_>>().join(", ")
        )
    };

    let type_imports = all_types
        .iter()
        .filter_map(|ty| {
            let (name, native_modules) = match ty {
                Type::Enum(name, _, _, _, opts) => (name, &opts.native_modules),
                Type::Struct(name, _, _, _, opts) => (name, &opts.native_modules),
                _ => return None,
            };
            native_modules
                .get(module_key)
                .map(|module| format!("pub use {}::{};", module, name))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let type_imports = if type_imports.is_empty() {
        type_imports
    } else {
        format!("{}\n\n", type_imports)
    };

    let type_defs = all_types
        .into_iter()
        .filter_map(|ty| match ty {
            Type::Alias(name, ty) => {
                Some(format!("pub type {} = {};", name, format_type(ty.as_ref())))
            }
            Type::Enum(name, generic_args, doc_lines, variants, opts) => {
                if opts.native_modules.contains_key(module_key) || name == "Result" {
                    None
                } else {
                    Some(create_enum_definition(
                        name,
                        generic_args,
                        &doc_lines,
                        variants,
                        opts,
                    ))
                }
            }
            Type::Struct(name, generic_args, doc_lines, fields, opts) => {
                if opts.native_modules.contains_key(module_key) {
                    None
                } else {
                    Some(create_struct_definition(
                        name,
                        generic_args,
                        &doc_lines,
                        fields,
                        opts,
                    ))
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    write_bindings_file(
        format!("{}/types.rs", path),
        format!(
            "use serde::{{Deserialize, Serialize}};\n{}\n{}{}\n",
            std_imports, type_imports, type_defs
        ),
    );
}

pub fn generate_function_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    path: &str,
    requires_async: bool,
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
                        _ => "__fp_macro::FatPtr".to_owned(),
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
                        "let {} = unsafe {{ import_value_from_host::<{}>({}) }};",
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
                    "let len = std::mem::size_of::<AsyncValue>() as u32;".to_owned(),
                    "let ptr = malloc(len);".to_owned(),
                    "let fat_ptr = to_fat_ptr(ptr, len);".to_owned(),
                    "let ptr = ptr as *mut AsyncValue;".to_owned(),
                    "".to_owned(),
                    "Task::spawn(Box::pin(async move {".to_owned(),
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

                let result_ptr = if has_return_value {
                    format!(
                        "export_value_to_host::<{}>(&ret)",
                        format_type(&function.return_type)
                    )
                } else {
                    "0".to_owned()
                };

                // If there is a return type, put the result in the `AsyncValue`
                // referenced by `ptr`:
                async_body.append(&mut vec![
                    // We're done, notify the host:
                    "    unsafe {".to_owned(),
                    format!("        let result_ptr = {};", result_ptr),
                    "        __fp_host_resolve_async_value(fat_ptr, result_ptr);".to_owned(),
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
                    ty => body.push(format!("export_value_to_host::<{}>(&ret)", format_type(ty))),
                }
                body
            };

            format!(
                "    ({}fn {}$args:tt{} $body:block) => {{
        #[no_mangle]
        pub fn __fp_gen_{}({}){} {{
            use __fp_macro::*;
{}
        }}

        {}fn {}$args{} $body
    }};",
                modifiers,
                name,
                if has_return_value { " -> $ret:ty" } else { "" },
                name,
                args_with_ptr_types,
                if function.is_async {
                    " -> __fp_macro::FatPtr"
                } else {
                    match &function.return_type {
                        Type::Unit => "",
                        Type::Primitive(_) => " -> $ret",
                        _ => " -> __fp_macro::FatPtr",
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
    let export_macro = format!(
        "#[macro_export]\nmacro_rules! fp_export {{\n{}\n}}",
        macro_rules
    );

    write_bindings_file(
        format!("{}/functions.rs", path),
        format!(
            "{}use super::support::*;\n\
            use super::types::*;\n\
            \n\
            #[link(wasm_import_module = \"fp\")]\n\
            extern \"C\" {{\n\
                {}\n\
            {}}}\n\
            \n\
            {}\n\
            \n\
            {}\n",
            if requires_async {
                "use super::r#async::*;\n"
            } else {
                ""
            },
            extern_decls,
            if requires_async {
                "\n    pub fn __fp_host_resolve_async_value(async_value_ptr: FatPtr, result_ptr: FatPtr);\n"
            } else {
                ""
            },
            fn_defs,
            export_macro
        ),
    );
}

fn collect_std_types(ty: &Type) -> BTreeSet<String> {
    match ty {
        Type::Alias(_, ty) => collect_std_types(ty),
        Type::Container(name, ty) => {
            let mut types = collect_std_types(ty);
            if name == "Rc" {
                types.insert("rc::Rc".to_owned());
            }
            types
        }
        Type::Custom(_) => BTreeSet::new(),
        Type::Enum(_, _, _, variants, _) => {
            let mut types = BTreeSet::new();
            for variant in variants {
                types.append(&mut collect_std_types(&variant.ty));
            }
            types
        }
        Type::GenericArgument(arg) => match &arg.ty {
            Some(ty) => collect_std_types(ty),
            None => BTreeSet::new(),
        },
        Type::List(name, ty) => {
            let mut types = collect_std_types(ty);
            if name == "BTreeSet" || name == "HashSet" {
                types.insert(format!("collections::{}", name));
            }
            types
        }
        Type::Map(name, key, value) => {
            let mut types = collect_std_types(key);
            types.append(&mut collect_std_types(value));
            if name == "BTreeMap" || name == "HashMap" {
                types.insert(format!("collections::{}", name));
            }
            types
        }
        Type::Primitive(_) => BTreeSet::new(),
        Type::String => BTreeSet::new(),
        Type::Struct(_, _, _, fields, _) => {
            let mut types = BTreeSet::new();
            for field in fields {
                types.append(&mut collect_std_types(&field.ty));
            }
            types
        }
        Type::Tuple(items) => {
            let mut types = BTreeSet::new();
            for item in items {
                types.append(&mut collect_std_types(item));
            }
            types
        }
        Type::Unit => BTreeSet::new(),
    }
}

fn create_enum_definition(
    name: String,
    generic_args: Vec<GenericArgument>,
    doc_lines: &[String],
    variants: Vec<Variant>,
    opts: EnumOptions,
) -> String {
    let variants = variants
        .into_iter()
        .flat_map(|variant| {
            let variant_decl = match variant.ty {
                Type::Unit => format!("{},", variant.name),
                Type::Struct(_, _, _, fields, _) => {
                    let fields = format_struct_fields(&fields);
                    let has_multiple_lines = fields.iter().any(|field| field.contains('\n'));
                    let fields = if has_multiple_lines {
                        format!(
                            "\n{}\n",
                            fields
                                .iter()
                                .flat_map(|field| field.split('\n'))
                                .map(|line| if line.is_empty() {
                                    line.to_owned()
                                } else {
                                    format!("    {}", line)
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                                .trim_start_matches('\n'),
                        )
                    } else {
                        let fields = fields.join(" ");
                        format!(" {} ", &fields.trim_end_matches(','))
                    };
                    format!(
                        "#[serde(rename_all = \"camelCase\")]\n{} {{{}}},",
                        variant.name, fields
                    )
                }
                Type::Tuple(items) => {
                    let items = items
                        .iter()
                        .map(|item| format_type(item))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}({}),", variant.name, items)
                }
                other => panic!("Unsupported type for enum variant: {:?}", other),
            };

            let lines = if variant.doc_lines.is_empty() {
                variant_decl
                    .split('\n')
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            } else {
                let mut lines = format_docs(&variant.doc_lines)
                    .trim_end_matches('\n')
                    .split('\n')
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                lines.append(
                    &mut variant_decl
                        .split('\n')
                        .map(str::to_owned)
                        .collect::<Vec<_>>(),
                );
                lines
            };

            lines
                .iter()
                .map(|line| {
                    if line.is_empty() {
                        line.clone()
                    } else {
                        format!("    {}", line)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{}#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]\n\
        #[serde({})]\n\
        pub enum {} {{\n\
            {}\n\
        }}",
        format_docs(doc_lines),
        opts.to_serde_attrs().join(", "),
        format_name_with_generics(&name, &generic_args),
        variants
    )
}

fn create_struct_definition(
    name: String,
    generic_args: Vec<GenericArgument>,
    doc_lines: &[String],
    fields: Vec<Field>,
    opts: StructOptions,
) -> String {
    let fields = format_struct_fields(&fields)
        .iter()
        .flat_map(|field| field.split('\n'))
        .map(|line| {
            if line.is_empty() {
                line.to_owned()
            } else {
                format!(
                    "    {}{}",
                    if line.starts_with('#') || line.starts_with("///") {
                        ""
                    } else {
                        "pub "
                    },
                    line
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "{}#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]\n\
        #[serde({})]\n\
        pub struct {} {{\n\
            {}\n\
        }}",
        format_docs(doc_lines),
        opts.to_serde_attrs().join(", "),
        format_name_with_generics(&name, &generic_args),
        fields.trim_start_matches('\n')
    )
}

fn format_docs(doc_lines: &[String]) -> String {
    doc_lines
        .iter()
        .map(|line| format!("///{}\n", line))
        .collect::<Vec<_>>()
        .join("")
}

fn format_name_with_types(name: &str, generic_args: &[GenericArgument]) -> String {
    if generic_args.is_empty() {
        name.to_owned()
    } else {
        format!(
            "{}<{}>",
            name,
            generic_args
                .iter()
                .map(|arg| match &arg.ty {
                    Some(ty) => format_type(ty),
                    None => arg.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn format_struct_fields(fields: &[Field]) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let mut serde_attrs = vec![];
            if matches!(&field.ty, Type::Container(name, _) if name == "Option") {
                serde_attrs.push("skip_serializing_if = \"Option::is_none\"");
            }

            if is_binary_type(&field.ty) {
                serde_attrs.push("with = \"serde_bytes\"");
            }

            let docs = if field.doc_lines.is_empty() {
                "".to_owned()
            } else {
                format!(
                    "\n{}",
                    field
                        .doc_lines
                        .iter()
                        .map(|line| format!("///{}\n", line))
                        .collect::<Vec<_>>()
                        .join("")
                )
            };

            let annotations = if serde_attrs.is_empty() {
                "".to_owned()
            } else {
                format!("#[serde({})]\n", serde_attrs.join(", "))
            };

            format!(
                "{}{}{}: {},",
                docs,
                annotations,
                field.name,
                format_type(&field.ty)
            )
        })
        .collect()
}

/// Formats a type so it's valid Rust again.
pub fn format_type(ty: &Type) -> String {
    match ty {
        Type::Alias(name, _) => name.clone(),
        Type::Container(name, ty) => format!("{}<{}>", name, format_type(ty)),
        Type::Custom(custom) => custom.rs_ty.clone(),
        Type::Enum(name, generic_args, _, _, _) => format_name_with_types(name, generic_args),
        Type::GenericArgument(arg) => arg.name.clone(),
        Type::List(name, ty) => format!("{}<{}>", name, format_type(ty)),
        Type::Map(name, k, v) => format!("{}<{}, {}>", name, format_type(k), format_type(v)),
        Type::Primitive(primitive) => format_primitive(*primitive),
        Type::String => "String".to_owned(),
        Type::Struct(name, generic_args, _, _, _) => format_name_with_types(name, generic_args),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(|item| item.name())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Unit => "()".to_owned(),
    }
}

pub fn format_primitive(primitive: Primitive) -> String {
    let string = match primitive {
        Primitive::Bool => "bool",
        Primitive::F32 => "f32",
        Primitive::F64 => "f64",
        Primitive::I8 => "i8",
        Primitive::I16 => "i16",
        Primitive::I32 => "i32",
        Primitive::I64 => "i64",
        Primitive::U8 => "u8",
        Primitive::U16 => "u16",
        Primitive::U32 => "u32",
        Primitive::U64 => "u64",
    };
    string.to_owned()
}

/// Detects types that can be encoded as a binary blob.
fn is_binary_type(ty: &Type) -> bool {
    match ty {
        Type::List(name, ty) if name == "Vec" && ty.as_ref() == &Type::Primitive(Primitive::U8) => {
            true
        }
        Type::Container(name, ty) if (name == "Box" || name == "Option") => {
            is_binary_type(ty.as_ref())
        }
        _ => false,
    }
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
