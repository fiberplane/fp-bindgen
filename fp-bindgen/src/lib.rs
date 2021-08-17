mod functions;
mod generators;
mod primitives;
mod serializable;
mod types;

pub mod prelude;

use fp_bindgen_macros::primitive_impls;
use prelude::*;
use std::{fs, str::FromStr};

primitive_impls!();

enum BindingsType {
    TsRuntime,
}

impl FromStr for BindingsType {
    type Err = String;

    fn from_str(bindings_type: &str) -> Result<Self, Self::Err> {
        match bindings_type {
            "ts-runtime" => Ok(Self::TsRuntime),
            other => Err(format!(
                "Bindings type must be one of \"ts-runtime\", was: \"{}\"",
                other
            )),
        }
    }
}

pub fn generate_bindings(
    import_functions: FunctionMap,
    export_functions: FunctionMap,
    bindings_type: &str,
    path: &str,
) {
    let bindings_type = BindingsType::from_str(bindings_type).expect("Unknown bindings type");

    fs::create_dir_all(path).expect("Could not create output directory");

    let (serializable_import_types, deserializable_import_types) =
        get_serializable_types(&import_functions);
    let (mut serializable_export_types, mut deserializable_export_types) =
        get_serializable_types(&export_functions);

    let mut serializable_types = serializable_import_types;
    serializable_types.append(&mut deserializable_export_types);

    let mut deserializable_types = deserializable_import_types;
    deserializable_types.append(&mut serializable_export_types);

    match bindings_type {
        BindingsType::TsRuntime => generators::ts_runtime::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            path,
        ),
    }
}

/// Returns the types for which serialization logic needs to be implemented in order to call the
/// given `functions`. The returned types are split into two categories: those which we need to be
/// able to serialize (arguments to the functions), and those we need to be able to deserialize
/// (the functions' return values).
fn get_serializable_types(functions: &FunctionMap) -> (Vec<Type>, Vec<Type>) {
    // TODO
    (vec![], vec![])
}
