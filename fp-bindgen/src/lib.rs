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
};

primitive_impls!();

#[derive(Debug, Clone)]
pub enum BindingsType<'a> {
    RustPlugin(RustPluginConfig<'a>),
    RustWasmerRuntime(WasmerRuntimeConfig),
    TsRuntime(TsRuntimeConfig),
}

impl<'a> Display for BindingsType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BindingsType::RustPlugin { .. } => "rust-plugin",
            BindingsType::RustWasmerRuntime { .. } => "rust-wasmer-runtime",
            BindingsType::TsRuntime { .. } => "ts-runtime",
        })
    }
}

#[derive(Debug)]
pub struct BindingConfig<'a> {
    pub bindings_type: BindingsType<'a>,
    pub path: &'a str,
}

#[derive(Debug, Clone)]
pub struct RustPluginConfig<'a> {
    pub name: &'a str,
    pub authors: &'a str,
    pub version: &'a str,
    pub dependencies: BTreeMap<String, String>,
}
#[derive(Debug, Clone)]
pub struct WasmerRuntimeConfig {
    pub generate_raw_export_wrappers: bool,
}

#[derive(Debug, Clone)]
pub struct TsRuntimeConfig {
    pub generate_raw_export_wrappers: bool,
}

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    serializable_types: BTreeSet<Type>,
    deserializable_types: BTreeSet<Type>,
    config: BindingConfig,
) {
    fs::create_dir_all(config.path).expect("Could not create output directory");

    match config.bindings_type {
        BindingsType::RustPlugin(plugin_config) => generators::rust_plugin::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            plugin_config,
            config.path,
        ),
        BindingsType::RustWasmerRuntime(runtime_config) => {
            generators::rust_wasmer_runtime::generate_bindings(
                import_functions,
                export_functions,
                serializable_types,
                deserializable_types,
                runtime_config,
                config.path,
            )
        }
        BindingsType::TsRuntime(runtime_config) => generators::ts_runtime::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            runtime_config,
            config.path,
        ),
    };
}
