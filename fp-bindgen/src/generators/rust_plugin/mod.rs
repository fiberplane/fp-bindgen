use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::types::{CargoDependency, EnumOptions, Field, StructOptions, Type, Variant};
use crate::RustPluginConfig;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    config: RustPluginConfig,
    path: &str,
) {
    let src_path = format!("{}/src", path);
    fs::create_dir_all(&src_path).expect("Could not create output directory");

    generate_cargo_file(
        config,
        &import_functions,
        &serializable_types,
        &deserializable_types,
        path,
    );

    generate_type_bindings(
        serializable_types,
        deserializable_types,
        &src_path,
        "rust_plugin",
    );
    generate_imported_function_bindings(import_functions, &src_path);
    generate_exported_function_bindings(export_functions, &src_path);

    write_bindings_file(
        format!("{}/lib.rs", src_path),
        "#[rustfmt::skip]
mod export;
#[rustfmt::skip]
mod import;
#[rustfmt::skip]
mod types;

pub use export::*;
pub use import::*;
pub use types::*;

pub use fp_bindgen_support::*;
",
    );
}

fn generate_cargo_file(
    config: RustPluginConfig,
    import_functions: &FunctionList,
    serializable_types: &BTreeSet<Type>,
    deserializable_types: &BTreeSet<Type>,
    path: &str,
) {
    let requires_async = import_functions.iter().any(|function| function.is_async);

    let mut support_features = BTreeSet::from(["guest"]);
    if requires_async {
        support_features.insert("async");
    }

    let mut dependencies = BTreeMap::from([
        (
            "fp-bindgen-support",
            CargoDependency {
                git: Some("ssh://git@github.com/fiberplane/fp-bindgen.git"),
                branch: Some("main"),
                path: None,
                version: None,
                features: support_features,
            },
        ),
        ("once_cell", CargoDependency::with_version("1")),
        ("rmp-serde", CargoDependency::with_version("1.0.0")),
        (
            "serde",
            CargoDependency::with_version_and_features("1.0", BTreeSet::from(["derive"])),
        ),
    ]);

    // Inject dependencies from custom types:
    for ty in serializable_types.iter().chain(deserializable_types.iter()) {
        if let Type::Custom(custom_type) = ty {
            for (name, dependency) in custom_type.rs_dependencies.iter() {
                let dependency = if let Some(existing_dependency) = dependencies.remove(name) {
                    existing_dependency.merge_or_replace_with(dependency)
                } else {
                    dependency.clone()
                };
                dependencies.insert(name, dependency);
            }
        }
    }

    // Inject dependencies passed through the config:
    for (name, dependency) in config.dependencies {
        let dependency = if let Some(existing_dependency) = dependencies.remove(name) {
            existing_dependency.merge_or_replace_with(&dependency)
        } else {
            dependency.clone()
        };
        dependencies.insert(name, dependency);
    }

    write_bindings_file(
        format!("{}/Cargo.toml", path),
        format!(
            "[package]
name = \"{}\"
version = \"{}\"
authors = {}
edition = \"2018\"

[dependencies]
{}
",
            config.name,
            config.version,
            config.authors,
            dependencies
                .iter()
                .map(|(name, value)| format!("{} = {}", name, value))
                .collect::<Vec<_>>()
                .join("\n")
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
        .flat_map(collect_std_types)
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

    let mut type_imports = all_types
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
        .collect::<Vec<_>>();
    let type_imports = if type_imports.is_empty() {
        "".to_owned()
    } else {
        type_imports.dedup();
        format!("{}\n\n", type_imports.join("\n"))
    };

    let mut type_defs = all_types
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
        .collect::<Vec<_>>();
    type_defs.dedup();

    write_bindings_file(
        format!("{}/types.rs", path),
        format!(
            "use serde::{{Deserialize, Serialize}};\n{}\n{}{}\n",
            std_imports,
            type_imports,
            type_defs.join("\n\n")
        ),
    );
}

fn format_functions(export_functions: FunctionList, macro_path: &str) -> String {
    export_functions
        .iter()
        .map(|func| {
            let name = &func.name;
            let doc = func
                .doc_lines
                .iter()
                .map(|line| format!("///{}\n", line))
                .collect::<Vec<_>>()
                .join("");
            let modifiers = if func.is_async { "async " } else { "" };
            let args_with_types = func
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name, format_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &func.return_type {
                Type::Unit => "".to_owned(),
                ty => format!(" -> {}", format_type(ty)),
            };
            format!(
                "#[{}]\n{}pub {}fn {}({}){};",
                macro_path, doc, modifiers, name, args_with_types, return_type,
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn generate_imported_function_bindings(import_functions: FunctionList, path: &str) {
    write_bindings_file(
        format!("{}/import.rs", path),
        format!(
            "use crate::types::*;\n\n{}\n",
            format_functions(import_functions, "fp_bindgen_support::fp_import_signature")
        ),
    );
}

fn generate_exported_function_bindings(export_functions: FunctionList, path: &str) {
    write_bindings_file(
        format!("{}/export.rs", path),
        format!(
            "use crate::types::*;\n\n{}\n",
            format_functions(export_functions, "fp_bindgen_support::fp_export_signature")
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
            let mut serde_attrs = variant.attrs.to_serde_attrs();
            let mut variant_decl = match variant.ty {
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
                    if !serde_attrs
                        .iter()
                        .any(|attr| attr.starts_with("rename_all ="))
                    {
                        serde_attrs.push("rename_all = \"camelCase\"".to_owned());
                    }
                    format!("{} {{{}}},", variant.name, fields)
                }
                Type::Tuple(items) => {
                    let items = items.iter().map(format_type).collect::<Vec<_>>().join(", ");
                    format!("{}({}),", variant.name, items)
                }
                other => panic!("Unsupported type for enum variant: {:?}", other),
            };

            if !serde_attrs.is_empty() {
                serde_attrs.sort();
                variant_decl = format!("#[serde({})]\n{}", serde_attrs.join(", "), variant_decl);
            }

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
            let mut serde_attrs = field.attrs.to_serde_attrs();
            match &field.ty {
                Type::Container(name, _) if name == "Option" => {
                    if !serde_attrs
                        .iter()
                        .any(|attr| attr == "default" || attr.starts_with("default = "))
                    {
                        serde_attrs.push("default".to_owned());
                    }
                    if !serde_attrs
                        .iter()
                        .any(|attr| attr.starts_with("skip_serializing_if ="))
                    {
                        serde_attrs.push("skip_serializing_if = \"Option::is_none\"".to_owned());
                    }
                }
                Type::Custom(custom_type) => {
                    for attr in custom_type.serde_attrs.iter() {
                        serde_attrs.push(attr.clone());
                    }
                }
                _ => {}
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
                serde_attrs.sort();
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
            items.iter().map(format_type).collect::<Vec<_>>().join(", ")
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

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
