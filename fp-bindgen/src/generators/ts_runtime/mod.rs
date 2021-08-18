use crate::functions::FunctionList;
use crate::prelude::Primitive;
use crate::types::{Field, Type, Variant};
use std::collections::BTreeSet;
use std::fs;

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    mut deserializable_types: BTreeSet<Type>,
    path: &str,
) {
    let mut all_types = serializable_types;
    all_types.append(&mut deserializable_types);

    let type_defs = all_types
        .into_iter()
        .filter_map(|ty| match ty {
            Type::Enum(name, variants) => Some(create_enum_definition(name, variants)),
            Type::Struct(name, fields) => Some(create_struct_definition(name, fields)),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let import_defs = import_functions
        .into_iter()
        .map(|function| {
            let modifiers = if function.is_async { "async " } else { "" };
            let args = function
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name, format_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match function.return_type {
                None => "".to_owned(),
                Some(ty) => format!(": {}", format_type(&ty)),
            };
            format!(
                "export {}function {}({}){} {{\n    // TODO: Impl body\n}}",
                modifiers, function.name, args, return_type
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // FIXME: This is just a copy of import_defs...
    let export_defs = export_functions
        .into_iter()
        .map(|function| {
            let modifiers = if function.is_async { "async " } else { "" };
            let args = function
                .args
                .iter()
                .map(|arg| format!("{}: {}", arg.name, format_type(&arg.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_type = match function.return_type {
                None => "".to_owned(),
                Some(ty) => format!(": {}", format_type(&ty)),
            };
            format!(
                "{}function {}({}){} {{\n    // TODO: Impl body\n}}",
                modifiers, function.name, args, return_type
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let file_path = format!("{}/index.ts", path);
    let contents = format!("{}\n\n{}\n\n{}\n", type_defs, import_defs, export_defs);
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}

fn create_enum_definition(name: String, variants: Vec<Variant>) -> String {
    "TODO".to_owned() // TODO
}

fn create_struct_definition(name: String, fields: Vec<Field>) -> String {
    let fields = fields
        .into_iter()
        .map(|field| format!("    {}: {};", field.name, format_type(&field.ty)))
        .collect::<Vec<_>>()
        .join("\n");

    format!("export type {} = {{\n{}\n}};", name, fields)
}

/// Formats a type so it's valid TypeScript.
fn format_type(ty: &Type) -> String {
    match ty {
        Type::Enum(name, _) => name.clone(),
        Type::List(_, ty) => format!("Array<{}>", format_type(ty)),
        Type::Map(_, k, v) => format!("Record<{}, {}>", format_type(k), format_type(v)),
        Type::Option(ty) => format!("{} | undefined", format_type(ty)),
        Type::Primitive(primitive) => format_primitive(*primitive),
        Type::Struct(name, _) => name.clone(),
    }
}

fn format_primitive(primitive: Primitive) -> String {
    let string = match primitive {
        Primitive::Bool => "boolean",
        Primitive::F32 => "number",
        Primitive::F64 => "number",
        Primitive::I8 => "number",
        Primitive::I16 => "number",
        Primitive::I32 => "number",
        Primitive::I64 => "number",
        Primitive::I128 => "number",
        Primitive::String => "string",
        Primitive::U8 => "number",
        Primitive::U16 => "number",
        Primitive::U32 => "number",
        Primitive::U64 => "number",
        Primitive::U128 => "number",
    };
    string.to_owned()
}
