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
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs,
    str::FromStr,
};

primitive_impls!();

#[derive(Debug, Clone, Copy)]
pub enum BindingsType {
    RustPlugin,
    RustWasmerRuntime,
    TsRuntime,
}

impl Display for BindingsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BindingsType::RustPlugin => "rust-plugin",
            BindingsType::RustWasmerRuntime => "rust-wasmer-runtime",
            BindingsType::TsRuntime => "ts-runtime",
        })
    }
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

#[derive(Debug)]
pub struct BindingConfig<'a> {
    pub bindings_type: BindingsType,
    pub path: &'a str,
    pub rust_plugin_config: Option<RustPluginConfig<'a>>,
}

#[derive(Debug)]
pub struct RustPluginConfig<'a> {
    pub name: &'a str,
    pub authors: &'a str,
    pub version: &'a str,
    pub dependencies: BTreeMap<String, String>,
}

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    config: BindingConfig,
) {
    fs::create_dir_all(config.path).expect("Could not create output directory");

    let generate_bindings = match config.bindings_type {
        BindingsType::RustPlugin => generators::rust_plugin::generate_bindings,
        BindingsType::RustWasmerRuntime => generators::rust_wasmer_runtime::generate_bindings,
        BindingsType::TsRuntime => generators::ts_runtime::generate_bindings,
    };

    generate_bindings(
        import_functions,
        export_functions,
        serializable_types,
        deserializable_types,
        config,
    );
}
