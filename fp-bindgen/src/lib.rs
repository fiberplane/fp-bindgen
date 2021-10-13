mod casing;
mod docs;
mod functions;
mod generators;
mod serializable;

pub mod generics;
pub mod prelude;
pub mod primitives;
pub mod types;

use fp_bindgen_macros::primitive_impls;
use prelude::*;
use std::{collections::BTreeSet, fs, str::FromStr};

primitive_impls!();

enum BindingsType {
    RustPlugin,
    RustWasmerRuntime,
    TsRuntime,
}

impl FromStr for BindingsType {
    type Err = String;

    fn from_str(bindings_type: &str) -> Result<Self, Self::Err> {
        match bindings_type {
            "rust-plugin" => Ok(Self::RustPlugin),
            "rust-wasmer-runtime" => Ok(Self::RustWasmerRuntime),
            "ts-runtime" => Ok(Self::TsRuntime),
            other => Err(format!(
                "Bindings type must be one of `rust-plugin`, `rust-wasmer-runtime`, `ts-runtime`.
                Received: `{}`",
                other
            )),
        }
    }
}

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    bindings_type: &str,
    path: &str,
) {
    let bindings_type = BindingsType::from_str(bindings_type).expect("Unknown bindings type");

    fs::create_dir_all(path).expect("Could not create output directory");

    let generate_bindings = match bindings_type {
        BindingsType::RustPlugin => generators::rust_plugin::generate_bindings,
        BindingsType::RustWasmerRuntime => generators::rust_wasmer_runtime::generate_bindings,
        BindingsType::TsRuntime => generators::ts_runtime::generate_bindings,
    };

    generate_bindings(
        import_functions,
        export_functions,
        serializable_types,
        deserializable_types,
        path,
    );
}
