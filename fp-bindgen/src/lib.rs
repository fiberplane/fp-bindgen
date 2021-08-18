mod generators;
mod traits;

pub mod prelude;

use fp_bindgen_common::FunctionMap;
use std::{fs, str::FromStr};

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

    match bindings_type {
        BindingsType::TsRuntime => {
            generators::ts_runtime::generate_bindings(import_functions, export_functions, path)
        }
    }
}
