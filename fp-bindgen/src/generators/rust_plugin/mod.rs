use crate::functions::Function;
use crate::types::is_runtime_bound;
use crate::{
    functions::FunctionList,
    types::{CargoDependency, Enum, Field, Struct, Type, TypeIdent, TypeMap},
    RustPluginConfig,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

pub(crate) fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: TypeMap,
    config: RustPluginConfig,
    path: &str,
) {
    let src_path = format!("{path}/src");
    fs::create_dir_all(&src_path).expect("Could not create output directory");

    generate_cargo_file(config, &import_functions, &types, path);

    generate_type_bindings(&types, &src_path);
    generate_imported_function_bindings(import_functions, &types, &src_path);
    generate_exported_function_bindings(export_functions, &types, &src_path);

    write_bindings_file(
        format!("{src_path}/lib.rs"),
        "#![allow(unused_imports)]
#[rustfmt::skip]
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
    types: &TypeMap,
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
                version: Some(env!("CARGO_PKG_VERSION")),
                features: support_features,
                ..CargoDependency::default()
            },
        ),
        ("once_cell", CargoDependency::with_version("1")),
        ("rmp-serde", CargoDependency::with_version("1.0")),
        (
            "serde",
            CargoDependency::with_version_and_features("1.0", BTreeSet::from(["derive"])),
        ),
    ]);

    // Inject dependencies from custom types:
    for ty in types.values() {
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
        format!("{path}/Cargo.toml"),
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
                .map(|(name, value)| format!("{name} = {value}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    );
}

pub fn generate_type_bindings(types: &TypeMap, path: &str) {
    let std_types: BTreeSet<_> = types.values().filter_map(collect_std_types).collect();
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

    let type_imports = types
        .values()
        .filter_map(|ty| {
            let (ident, rust_module) = match ty {
                Type::Enum(Enum { ident, options, .. }) => (ident, &options.rust_module),
                Type::Struct(Struct { ident, options, .. }) => (ident, &options.rust_module),
                _ => return None,
            };

            rust_module
                .as_ref()
                .map(|module| format!("pub use {}::{};", module, ident.name))
        })
        .collect::<Vec<_>>();
    let type_imports = if type_imports.is_empty() {
        "".to_owned()
    } else {
        format!("{}\n\n", type_imports.join("\n"))
    };

    let type_defs = types
        .values()
        .filter_map(|ty| match ty {
            Type::Alias(name, ty) => {
                Some(format!("pub type {} = {};", name, format_ident(ty, types)))
            }
            Type::Enum(ty) => {
                if ty.options.rust_module.is_some() || ty.ident.name == "Result" {
                    None
                } else {
                    Some(create_enum_definition(ty, types))
                }
            }
            Type::Struct(ty) => {
                if ty.options.rust_module.is_some() {
                    None
                } else {
                    Some(create_struct_definition(ty, types))
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    write_bindings_file(
        format!("{path}/types.rs"),
        format!(
            "#![allow(unused_imports)]\n\
            use serde::{{Deserialize, Serialize}};\n{}\n{}{}\n",
            std_imports,
            type_imports,
            type_defs.join("\n\n")
        ),
    );
}

pub fn format_doc_lines(doc_lines: &[String]) -> String {
    doc_lines
        .iter()
        .map(|line| format!("///{line}\n"))
        .collect::<Vec<_>>()
        .join("")
}

pub fn format_modifiers(function: &Function) -> String {
    if function.is_async { "async " } else { "" }.to_owned()
}

fn format_functions(functions: FunctionList, types: &TypeMap, macro_path: &str) -> String {
    functions
        .iter()
        .map(|func| {
            let name = &func.name;
            let doc = format_doc_lines(&func.doc_lines);
            let modifiers = format_modifiers(func);
            let args_with_types = func
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name, format_ident(&arg.ty, types)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match &func.return_type {
                Some(ty) => format!(" -> {}", format_ident(ty, types)),
                None => "".to_owned(),
            };
            format!(
                "{doc}#[{macro_path}]\npub {modifiers}fn {name}({args_with_types}){return_type};",
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn format_ident(ident: &TypeIdent, types: &TypeMap) -> String {
    match types.get(ident) {
        Some(ty) => format_type_with_ident(ty, ident, types),
        None => ident.to_string(), // Must be a generic.
    }
}

fn format_type_with_ident(ty: &Type, ident: &TypeIdent, types: &TypeMap) -> String {
    let format_name_with_args = |name: &str, maybe_num_expected_args: Option<usize>| {
        let generic_args: Vec<_> = ident
            .generic_args
            .iter()
            .map(|(arg, bounds)| format!("{}{}", format_ident(arg, types), format_bounds(bounds)))
            .collect();
        if let Some(num_expected_args) = maybe_num_expected_args {
            if generic_args.len() != num_expected_args {
                panic!(
                    "Expected {} generic arguments, but found {}",
                    num_expected_args,
                    generic_args.len()
                );
            }
        }

        if generic_args.is_empty() {
            name.to_owned()
        } else {
            format!("{}<{}>", name, generic_args.join(", "))
        }
    };

    match ty {
        Type::Alias(name, _) => name.clone(),
        Type::Container(name, _) | Type::List(name, _) => format_name_with_args(name, Some(1)),
        Type::Custom(custom) => custom.rs_ty.clone(),
        Type::Enum(Enum { ident, .. }) => format_name_with_args(&ident.name, None),
        Type::Map(name, _, _) => format_name_with_args(name, Some(2)),
        Type::Struct(Struct { ident, .. }) => format_name_with_args(&ident.name, None),
        Type::Tuple(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|item| format_ident(item, types))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Unit => "()".to_owned(),
        _ => ident.to_string(),
    }
}

fn format_bounds(bounds: &[String]) -> String {
    bounds
        .iter()
        .filter(|bound| is_runtime_bound(bound))
        .cloned()
        .collect::<Vec<_>>()
        .join(" + ")
}

fn generate_imported_function_bindings(
    import_functions: FunctionList,
    types: &TypeMap,
    path: &str,
) {
    write_bindings_file(
        format!("{path}/import.rs"),
        format!(
            "use crate::types::*;\n\n{}\n",
            format_functions(
                import_functions,
                types,
                "fp_bindgen_support::fp_import_signature"
            )
        ),
    );
}

fn generate_exported_function_bindings(
    export_functions: FunctionList,
    types: &TypeMap,
    path: &str,
) {
    write_bindings_file(
        format!("{path}/export.rs"),
        format!(
            "use crate::types::*;\n\n{}\n",
            format_functions(
                export_functions,
                types,
                "fp_bindgen_support::fp_export_signature"
            )
        ),
    );
}

fn collect_std_types(ty: &Type) -> Option<String> {
    match ty {
        Type::Container(name, _) if name == "Rc" => Some("rc::Rc".to_owned()),
        Type::List(name, _) if (name == "BTreeSet" || name == "HashSet") => {
            Some(format!("collections::{name}"))
        }
        Type::Map(name, _, _) if (name == "BTreeMap" || name == "HashMap") => {
            Some(format!("collections::{name}"))
        }
        _ => None,
    }
}

fn create_enum_definition(ty: &Enum, types: &TypeMap) -> String {
    let variants = ty
        .variants
        .iter()
        .flat_map(|variant| {
            let mut serde_attrs = variant.attrs.to_serde_attrs();
            let mut variant_decl = match &variant.ty {
                Type::Unit => format!("{},", variant.name),
                Type::Struct(variant) => {
                    let fields = format_struct_fields(&variant.fields, types);
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
                                    format!("    {line}")
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                                .trim_start_matches('\n'),
                        )
                    } else {
                        let fields = fields.join(" ");
                        format!(" {} ", &fields.trim_end_matches(','))
                    };
                    format!("{} {{{}}},", variant.ident.name, fields)
                }
                Type::Tuple(items) => {
                    let items = items
                        .iter()
                        .map(|item| format_ident(item, types))
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
                        format!("    {line}")
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let serde_annotation = {
        let attrs = ty.options.to_serde_attrs();
        if attrs.is_empty() {
            "".to_owned()
        } else {
            format!("#[serde({})]\n", attrs.join(", "))
        }
    };

    format!(
        "{}#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]\n{}\
        pub enum {} {{\n\
            {}\n\
        }}",
        format_docs(&ty.doc_lines),
        serde_annotation,
        ty.ident,
        variants
    )
}

fn create_struct_definition(ty: &Struct, types: &TypeMap) -> String {
    let is_tuple_struct = ty
        .fields
        .first()
        .map(|field| field.name.is_none())
        .unwrap_or_default();

    let fields = format_struct_fields(&ty.fields, types)
        .iter()
        .flat_map(|field| field.split('\n'))
        .map(|line| {
            if line.is_empty() {
                line.to_owned()
            } else {
                format!(
                    "{}{}",
                    if line.starts_with('#') || line.starts_with("///") {
                        ""
                    } else {
                        "pub "
                    },
                    line
                )
            }
        })
        .collect::<Vec<_>>();
    let fields = if fields.len() > 1 || !is_tuple_struct {
        fields
            .into_iter()
            .map(|line| {
                if line.is_empty() {
                    line
                } else {
                    format!("    {line}")
                }
            })
            .collect::<Vec<_>>()
    } else {
        fields
    };

    let serde_annotation = {
        let attrs = ty.options.to_serde_attrs();
        if attrs.is_empty() {
            "".to_owned()
        } else {
            format!("#[serde({})]\n", attrs.join(", "))
        }
    };

    let annotations = format!(
        "{}#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]\n{}",
        format_docs(&ty.doc_lines),
        serde_annotation
    );

    // Format ident, include bounds and skip compile-time only bounds
    let ident = ty.ident.format(true);
    if is_tuple_struct {
        if fields.len() > 1 {
            format!(
                "{}pub struct {}(\n{}\n);",
                annotations,
                ident,
                fields.join("\n").trim_start_matches('\n')
            )
        } else {
            format!("{}pub struct {}({});", annotations, ident, fields.join(" "))
        }
    } else {
        format!(
            "{}pub struct {} {{\n{}\n}}",
            annotations,
            ident,
            fields.join("\n").trim_start_matches('\n')
        )
    }
}

fn format_docs(doc_lines: &[String]) -> String {
    doc_lines
        .iter()
        .map(|line| format!("///{line}\n"))
        .collect::<Vec<_>>()
        .join("")
}

fn format_struct_fields(fields: &[Field], types: &TypeMap) -> Vec<String> {
    fields
        .iter()
        .map(|field| {
            let mut serde_attrs = field.attrs.to_serde_attrs();

            if let Some(Type::Custom(custom_type)) = types.get(&field.ty) {
                for attr in custom_type.serde_attrs.iter() {
                    serde_attrs.push(attr.clone());
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
                        .map(|line| format!("///{line}\n"))
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

            if let Some(name) = field.name.as_ref() {
                format!(
                    "{}{}{}: {},",
                    docs,
                    annotations,
                    name,
                    format_ident(&field.ty, types)
                )
            } else {
                format!("{}{}{},", docs, annotations, format_ident(&field.ty, types))
            }
        })
        .collect()
}

fn write_bindings_file<C>(file_path: String, contents: C)
where
    C: AsRef<[u8]>,
{
    fs::write(file_path, &contents).expect("Could not write bindings file");
}
