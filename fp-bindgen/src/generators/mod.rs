use crate::{
    functions::FunctionList,
    types::{CargoDependency, Type},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs,
};

pub mod rust_plugin;
pub mod rust_wasmer_runtime;
pub mod ts_runtime;

#[derive(Debug, Clone)]
pub enum BindingsType<'a> {
    RustPlugin(RustPluginConfig<'a>),
    RustWasmerRuntime,
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
    pub dependencies: BTreeMap<&'a str, CargoDependency>,
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
        BindingsType::RustPlugin(plugin_config) => rust_plugin::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            plugin_config,
            config.path,
        ),
        BindingsType::RustWasmerRuntime => rust_wasmer_runtime::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            config.path,
        ),
        BindingsType::TsRuntime(runtime_config) => ts_runtime::generate_bindings(
            import_functions,
            export_functions,
            serializable_types,
            deserializable_types,
            runtime_config,
            config.path,
        ),
    };
}
