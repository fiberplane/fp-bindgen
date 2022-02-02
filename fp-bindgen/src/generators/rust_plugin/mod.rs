use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::types::{
    CargoDependency, Enum, EnumOptions, Field, Struct, StructOptions, Type, Variant,
};
use crate::RustPluginConfig;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: BTreeSet<Type>,
    config: RustPluginConfig,
    path: &str,
) {
    let src_path = format!("{}/src", path);
    fs::create_dir_all(&src_path).expect("Could not create output directory");

    generate_cargo_file(config, &import_functions, &types, path);

    generate_type_bindings(types, &src_path, "rust_plugin");
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
    types: &BTreeSet<Type>,
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
    for ty in types.iter() {
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

pub fn generate_type_bindings(types: BTreeSet<Type>, path: &str, module_key: &str) {
    let std_types = types
        .iter()
        .filter_map(collect_std_types)
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

    let mut type_imports = types
        .iter()
        .filter_map(|ty| {
            let (ident, native_modules) = match ty {
                Type::Enum(Enum { ident, options, .. }) => (ident, &options.native_modules),
                Type::Struct(Struct { ident, options, .. }) => (ident, &options.native_modules),
                _ => return None,
            };
            native_modules
                .get(module_key)
                .map(|module| format!("pub use {}::{};", module, ident.name))
        })
        .collect::<Vec<_>>();
    let type_imports = if type_imports.is_empty() {
        "".to_owned()
    } else {
        type_imports.dedup();
        format!("{}\n\n", type_imports.join("\n"))
    };

    let mut type_defs = types
        .into_iter()
        .filter_map(|ty| match ty {
            Type::Alias(name, ty) => Some(format!("pub type {} = {};", name, ty)),
            Type::Enum(ty) => {
                if ty.options.native_modules.contains_key(module_key) || ty.ident.name == "Result" {
                    None
                } else {
                    Some(create_enum_definition(ty))
                }
            }
            Type::Struct(ty) => {
                if ty.options.native_modules.contains_key(module_key) {
                    None
                } else {
                    Some(create_struct_definition(ty))
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

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
                .map(|arg| format!("{}: {}", arg.name, arg.ty))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &func.return_type {
                Some(ty) => format!(" -> {}", ty),
                None => "".to_owned(),
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

fn collect_std_types(ty: &Type) -> Option<String> {
    match ty {
        Type::Container(name, ty) if name == "Rc" => Some("rc::Rc".to_owned()),
        Type::List(name, ty) if (name == "BTreeSet" || name == "HashSet") => {
            Some(format!("collections::{}", name))
        }
        Type::Map(name, key, value) if (name == "BTreeMap" || name == "HashMap") => {
            Some(format!("collections::{}", name))
        }
        _ => None,
    }
}

fn create_enum_definition(ty: Enum) -> String {
    let variants = ty
        .variants
        .into_iter()
        .flat_map(|variant| {
            let mut serde_attrs = variant.attrs.to_serde_attrs();
            let mut variant_decl = match variant.ty {
                Type::Unit => format!("{},", variant.name),
                Type::Struct(variant) => {
                    let fields = format_struct_fields(&variant.fields);
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
                    format!("{} {{{}}},", variant.ident.name, fields)
                }
                Type::Tuple(items) => {
                    let items = items
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
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
        format_docs(&ty.doc_lines),
        ty.options.to_serde_attrs().join(", "),
        ty.ident,
        variants
    )
}

fn create_struct_definition(ty: Struct) -> String {
    let fields = format_struct_fields(&ty.fields)
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
        format_docs(&ty.doc_lines),
        ty.options.to_serde_attrs().join(", "),
        ty.ident,
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

fn format_struct_fields(fields: &[Field], types: &BTreeSet<Type>) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let mut serde_attrs = field.attrs.to_serde_attrs();

            if let Some(ty) = types.get(&field.ty) {
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
                            serde_attrs
                                .push("skip_serializing_if = \"Option::is_none\"".to_owned());
                        }
                    }
                    Type::Custom(custom_type) => {
                        for attr in custom_type.serde_attrs.iter() {
                            serde_attrs.push(attr.clone());
                        }
                    }
                    _ => {}
                }
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

            format!("{}{}{}: {},", docs, annotations, field.name, field.ty)
        })
        .collect()
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
