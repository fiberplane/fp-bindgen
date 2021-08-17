use syn::__private::ToTokens;
use syn::{ItemEnum, ItemStruct};

use crate::functions::{Function, FunctionMap};
use crate::types::Type;
use std::collections::BTreeSet;
use std::{fs, str::FromStr};

pub fn generate_bindings(
    import_functions: FunctionMap,
    export_functions: FunctionMap,
    serializable_types: BTreeSet<Type>,
    mut deserializable_types: BTreeSet<Type>,
    path: &str,
) {
    let mut all_types = serializable_types;
    all_types.append(&mut deserializable_types);

    let type_defs = all_types
        .into_iter()
        .map(|ty| {
            match ty {
                Type::Enum(ty) => create_enum_declaration(ty),
                Type::List(_, _) => "".to_owned(), // Lists are transparent.
                Type::Map(_, _, _) => "".to_owned(), // Maps are transparent.
                Type::Option(_) => "".to_owned(),  // Options are transparent.
                Type::Primitive(_) => "".to_owned(), // Primitives don't require special processing.
                Type::Struct(ty) => create_struct_declaration(ty),
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let import_defs = import_functions
        .into_iter()
        .map(|(fn_name, fn_decl)| {
            let function = Function::from_str(&fn_decl).unwrap();
            let modifiers = if function.is_async() { "async " } else { "" };
            let args = function
                .args()
                .iter()
                // FIXME: This works so long as we only care about printing the identifier, but it's
                //        too late to try to resolve the type_path to do something actually useful.
                //        The reason it's too late is because we have the type path in a variable,
                //        only known at runtime, while we need to query the type information at
                //        compile time. So instead, we should resolve them when we're still inside
                //        the macro and pass something smarter down here.
                .map(|arg| format!("{}: {}", arg.name, arg.type_path.segments[0].ident))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "export {}function {}({}) {{\n    // TODO: Impl body\n}}",
                modifiers, fn_name, args
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // FIXME: This is just a copy of import_defs...
    let export_defs = export_functions
        .into_iter()
        .map(|(fn_name, fn_decl)| {
            let function = Function::from_str(&fn_decl).unwrap();
            let modifiers = if function.is_async() { "async " } else { "" };
            let args = function
                .args()
                .iter()
                .map(|arg| format!("{}: {}", arg.name, arg.type_path.segments[0].ident))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}function {}({}) {{\n    // TODO: Impl body\n}}",
                modifiers, fn_name, args
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let file_path = format!("{}/index.ts", path);
    let contents = format!("{}\n\n{}\n\n{}\n", type_defs, import_defs, export_defs);
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}

fn create_enum_declaration(ty: ItemEnum) -> String {
    "TODO".to_owned() // TODO
}

fn create_struct_declaration(ty: ItemStruct) -> String {
    let name = ty.ident.to_string();
    let fields = ty
        .fields
        .into_iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .expect("Struct fields must be named")
                .to_string();
            let ty = match field.ty {
                syn::Type::Path(path) if path.qself.is_none() => {
                    path.path.to_token_stream().to_string()
                }
                _ => panic!(
                    "Only value types are supported. Incompatible type in struct field: {:?}",
                    field
                ),
            };
            format!("    {}: {};", name, ty)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!("export type {} = {{\n{}\n}};", name, fields)
}
