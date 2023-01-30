use crate::{
    functions::FunctionList,
    types::{CargoDependency, Type, TypeIdent, TypeMap},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    fs,
};

pub mod rust_plugin;
pub mod rust_wasmer_runtime;
pub mod rust_wasmer_wasi_runtime;
pub mod ts_runtime;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum BindingsType<'a> {
    RustPlugin(RustPluginConfig<'a>),
    RustWasmerRuntime,
    RustWasmerWasiRuntime,
    TsRuntimeWithExtendedConfig(TsExtendedRuntimeConfig),
}

impl<'a> Display for BindingsType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BindingsType::RustPlugin { .. } => "rust-plugin",
            BindingsType::RustWasmerRuntime { .. } => "rust-wasmer-runtime",
            BindingsType::RustWasmerWasiRuntime { .. } => "rust-wasmer-wasi-runtime",
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

    display_warnings(&import_functions, &export_functions, &types);

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
        BindingsType::RustWasmerWasiRuntime => rust_wasmer_wasi_runtime::generate_bindings(
            import_functions,
            export_functions,
            types,
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

fn display_warnings(
    import_functions: &FunctionList,
    export_functions: &FunctionList,
    types: &TypeMap,
) {
    let all_functions = import_functions.iter().chain(export_functions.iter());
    let all_function_signature_types = all_functions.flat_map(|func| {
        func.args
            .iter()
            .map(|arg| &arg.ty)
            .chain(func.return_type.iter())
    });
    warn_about_custom_serializer_usage(
        all_function_signature_types.clone(),
        "function signature",
        types,
    );

    // Finding usages as generic arguments is more tricky, because we need to
    // find all places where generic arguments can be used.
    let all_idents = all_function_signature_types
        .chain(
            types
                .values()
                .filter_map(|ty| match ty {
                    Type::Struct(ty) => Some(ty),
                    _ => None,
                })
                .flat_map(|ty| ty.fields.iter().map(|field| &field.ty)),
        )
        .chain(
            types
                .values()
                .filter_map(|ty| match ty {
                    Type::Enum(ty) => Some(ty),
                    _ => None,
                })
                .flat_map(|ty| ty.variants.iter())
                .filter_map(|variant| match &variant.ty {
                    Type::Struct(ty) => Some(ty),
                    _ => None,
                })
                .flat_map(|ty| ty.fields.iter().map(|field| &field.ty)),
        );
    warn_about_custom_serializer_usage(
        all_idents.flat_map(|ident| ident.generic_args.iter().map(|(arg, _)| arg)),
        "generic argument",
        types,
    );
}

fn warn_about_custom_serializer_usage<'a, T>(idents: T, context: &str, types: &TypeMap)
where
    T: Iterator<Item = &'a TypeIdent>,
{
    let mut idents_with_custom_serializers = BTreeSet::new();

    for ident in idents {
        let ty = types.get(ident);
        if let Some(Type::Custom(ty)) = ty {
            if ty.serde_attrs.iter().any(|attr| {
                attr.starts_with("with = ")
                    || attr.starts_with("serialize_with = ")
                    || attr.starts_with("deserialize_with = ")
            }) {
                idents_with_custom_serializers.insert(ident);
            }
        }
    }

    for ident in idents_with_custom_serializers {
        println!(
            "WARNING: Type `{ident}` is used directly in a {context}, but relies on a custom Serde \
            (de)serializer. This (de)serializer is NOT used when using the type directly \
            in a {context}. This may result in unexpected (de)serialization issues, for instance \
            when passing data between Rust and TypeScript.\n\
            You may wish to create a newtype to avoid this warning.\n\
            See `examples/example-protocol/src/types/time.rs` for an example."
        );
    }
}
