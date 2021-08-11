use fp_bindgen_common::{FunctionItem, FunctionMap};
use std::{fs, str::FromStr};

pub fn generate_bindings(import_functions: FunctionMap, export_functions: FunctionMap, path: &str) {
    let import_defs = import_functions
        .into_iter()
        .map(|(fn_name, fn_decl)| {
            let function = FunctionItem::from_str(&fn_decl).unwrap();
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
            let function = FunctionItem::from_str(&fn_decl).unwrap();
            let modifiers = if function.is_async() { "async " } else { "" };
            let args = function
                .args()
                .iter()
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

    let file_path = format!("{}/index.ts", path);
    let contents = format!("{}\n\n{}\n", import_defs, export_defs);
    fs::write(&file_path, &contents).expect("Could not write bindings file");
}
