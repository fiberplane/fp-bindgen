use crate::{
    functions::FunctionList,
    types::{CargoDependency, TypeMap},
};
use std::{collections::BTreeMap, fmt::Display, fs};

pub mod rust_plugin;
pub mod rust_wasmer_runtime;
pub mod ts_runtime;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BindingsType<'a> {
    RustPlugin(RustPluginConfig<'a>),
    RustWasmerRuntime,
    #[deprecated(note = "Please use `BindingsType::TsRuntimeWithExtendedConfig` instead.")]
    TsRuntime(TsRuntimeConfig),
    TsRuntimeWithExtendedConfig(TsExtendedRuntimeConfig),
}

impl<'a> Display for BindingsType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BindingsType::RustPlugin { .. } => "rust-plugin",
            BindingsType::RustWasmerRuntime { .. } => "rust-wasmer-runtime",
            BindingsType::TsRuntime { .. } => "ts-runtime",
            BindingsType::TsRuntimeWithExtendedConfig { .. } => "ts-runtime",
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
    /// Name of the plugin crate that will be generated.
    pub name: &'a str,

    /// Authors to be listed in the plugin crate that will be generated.
    pub authors: &'a str,

    /// Version of the plugin crate that will be generated.
    pub version: &'a str,

    /// *Additional* dependencies to be listed in the plugin crate that will be
    /// generated.
    ///
    /// These are merged with a small set of dependencies that are necessary
    /// for the plugin to work and which will always be included. Specifying
    /// these dependencies yourself can be useful if you want to explicitly bump
    /// a dependency version or you want to enable a Cargo feature in them.
    pub dependencies: BTreeMap<&'a str, CargoDependency>,
}

#[derive(Debug, Clone)]
pub struct TsRuntimeConfig {
    pub generate_raw_export_wrappers: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct TsExtendedRuntimeConfig {
    /// The module from which to import the MessagePack dependency.
    ///
    /// By default, "@msgpack/msgpack" is used, which should work with Node.js
    /// and most NPM-based bundlers. If you use Deno, you may wish to specify
    /// "https://unpkg.com/@msgpack/msgpack/mod.ts".
    pub msgpack_module: String,

    /// Whether or not to generate raw export wrappers.
    ///
    /// Raw export wrappers allow you to call `fp_export!` functions from the
    /// runtime while passing raw MessagePack data, which you can use in some
    /// situations to avoid (de)serialization overhead. If you don't need these
    /// wrappers, you can omit them to optimize your bundle size.
    ///
    /// Raw export wrappers are named similarly to the regular wrappers (which
    /// are generated in any case), but with a `Raw` suffix.
    pub generate_raw_export_wrappers: bool,
}

impl TsExtendedRuntimeConfig {
    /// Returns a new config instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `msgpack_module` setting.
    pub fn with_msgpack_module(mut self, msgpack_module: &str) -> Self {
        self.msgpack_module = msgpack_module.to_owned();
        self
    }

    /// Enables the `generate_raw_export_wrappers` setting.
    pub fn with_raw_export_wrappers(mut self) -> Self {
        self.generate_raw_export_wrappers = true;
        self
    }
}

impl Default for TsExtendedRuntimeConfig {
    fn default() -> Self {
        Self {
            generate_raw_export_wrappers: false,
            msgpack_module: "@msgpack/msgpack".to_owned(),
        }
    }
}

impl TsExtendedRuntimeConfig {}

pub fn generate_bindings(
    import_functions: FunctionList,
    export_functions: FunctionList,
    types: TypeMap,
    config: BindingConfig,
) {
    fs::create_dir_all(config.path).expect("Could not create output directory");

    match config.bindings_type {
        BindingsType::RustPlugin(plugin_config) => rust_plugin::generate_bindings(
            import_functions,
            export_functions,
            types,
            plugin_config,
            config.path,
        ),
        BindingsType::RustWasmerRuntime => rust_wasmer_runtime::generate_bindings(
            import_functions,
            export_functions,
            types,
            config.path,
        ),
        BindingsType::TsRuntime(runtime_config) => ts_runtime::generate_bindings(
            import_functions,
            export_functions,
            types,
            TsExtendedRuntimeConfig {
                generate_raw_export_wrappers: runtime_config.generate_raw_export_wrappers,
                ..Default::default()
            },
            config.path,
        ),
        BindingsType::TsRuntimeWithExtendedConfig(runtime_config) => ts_runtime::generate_bindings(
            import_functions,
            export_functions,
            types,
            runtime_config,
            config.path,
        ),
    };
}
