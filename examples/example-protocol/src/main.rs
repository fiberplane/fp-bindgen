#![allow(dead_code)]

use bytes::Bytes;
use fp_bindgen::{prelude::*, types::CargoDependency};
use once_cell::sync::Lazy;
use redux_example::{ReduxAction, StateUpdate};
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, BTreeSet};

// Referencing types using their full module path can be problematic in some
// edge cases. If you want to use types from other modules in your protocol,
// it's best to import them with a `use` statement and refer to them by their
// name only.
mod types;
use types::*;

fp_import! {
    // Aliases need to be explicitly mentioned in either `fp_import!` or
    // `fp_export!`.
    //
    // See `types/aliases.rs` for more info.
    type Body = ByteBuf;
    type FloatingPoint = Point<f64>;
    type HttpResult = Result<Response, RequestError>;
    type Int64 = u64;

    // Types that are not referenced by any of the protocol functions (either
    // directly as argument or return type, or indirectly through other types)
    // need to be explicitly "used" in order to be generated as part of the
    // generated bindings. Nested modules are supported in `use` statements.
    //
    // See `types/dead_code.rs` for more info.
    use ExplicitedlyImportedType;
    use submodule::{nested::GroupImportedType1, GroupImportedType2};
    use types::{DocExampleEnum, DocExampleStruct};

    // ===============================================================
    // Imported functions that we call as part of the end-to-end tests
    // ===============================================================

    // No arguments, no return type:
    fn import_void_function();

    // No arguments, empty return type:
    fn import_void_function_empty_return() -> ();

    // No arguments, generic empty result type:
    fn import_void_function_empty_result() -> Result<(), u32>;

    // Passing primitives:
    fn import_primitive_bool(arg: bool) -> bool;
    fn import_primitive_f32(arg: f32) -> f32;
    fn import_primitive_f64(arg: f64) -> f64;
    fn import_primitive_i8(arg: i8) -> i8;
    fn import_primitive_i16(arg: i16) -> i16;
    fn import_primitive_i32(arg: i32) -> i32;
    fn import_primitive_i64(arg: i64) -> i64;
    fn import_primitive_u8(arg: u8) -> u8;
    fn import_primitive_u16(arg: u16) -> u16;
    fn import_primitive_u32(arg: u32) -> u32;
    fn import_primitive_u64(arg: u64) -> u64;

    // Passing arrays:
    fn import_array_u8(arg: [u8; 3]) -> [u8; 3];
    fn import_array_u16(arg: [u16; 3]) -> [u16; 3];
    fn import_array_u32(arg: [u32; 3]) -> [u32; 3];
    fn import_array_i8(arg: [i8; 3]) -> [i8; 3];
    fn import_array_i16(arg: [i16; 3]) -> [i16; 3];
    fn import_array_i32(arg: [i32; 3]) -> [i32; 3];
    fn import_array_f32(arg: [f32; 3]) -> [f32; 3];
    fn import_array_f64(arg: [f64; 3]) -> [f64; 3];

    // Passing strings:
    fn import_string(arg: String) -> String;

    // Multiple arguments:
    fn import_multiple_primitives(arg1: i8, arg2: String) -> i64;

    // Integration with the `time` crate:
    fn import_timestamp(arg: MyDateTime) -> MyDateTime;

    // Passing custom types with flattened properties.
    //
    // See `types/flattening.rs` for more info.
    fn import_fp_flatten(arg: FpFlatten) -> FpFlatten;
    fn import_serde_flatten(arg: SerdeFlatten) -> SerdeFlatten;

    // Generics.
    //
    // See `types/generics.rs` for more info.
    fn import_generics(arg: StructWithGenerics<u64>) -> StructWithGenerics<u64>;
    fn import_explicit_bound_point(arg: ExplicitBoundPoint<u64>);

    // Options
    fn import_struct_with_options(arg: StructWithOptions) -> StructWithOptions;

    // Custom type in a generic position.
    fn import_get_bytes() -> Result<Bytes, String>;
    fn import_get_serde_bytes() -> Result<ByteBuf, String>;

    // Passing custom types with property/variant renaming.
    //
    // See `types/renaming.rs` for more info.
    fn import_fp_struct(arg: FpPropertyRenaming) -> FpPropertyRenaming;
    fn import_fp_enum(arg: FpVariantRenaming) -> FpVariantRenaming;
    fn import_serde_struct(arg: SerdePropertyRenaming) -> SerdePropertyRenaming;
    fn import_serde_enum(arg: SerdeVariantRenaming) -> SerdeVariantRenaming;

    // Passing custom enums with different tagging options.
    //
    // See `types/tagged_enums.rs` for more info.
    fn import_fp_internally_tagged(arg: FpInternallyTagged) -> FpInternallyTagged;
    fn import_fp_adjacently_tagged(arg: FpAdjacentlyTagged) -> FpAdjacentlyTagged;
    fn import_fp_untagged(arg: FpUntagged) -> FpUntagged;
    fn import_serde_internally_tagged(arg: SerdeInternallyTagged) -> SerdeInternallyTagged;
    fn import_serde_adjacently_tagged(arg: SerdeAdjacentlyTagged) -> SerdeAdjacentlyTagged;
    fn import_serde_untagged(arg: SerdeUntagged) -> SerdeUntagged;

    // Async function:
    async fn import_fp_struct(arg1: FpPropertyRenaming, arg2: u64) -> FpPropertyRenaming;

    /// Logs a message to the (development) console.
    fn log(message: String);

    /// Example how a runtime could expose a `Fetch`-like function to plugins.
    ///
    /// See `types/http.rs` for more info.
    async fn make_http_request(request: Request) -> HttpResult;
}

fp_export! {
    // ===============================================================
    // Exported functions that we call as part of the end-to-end tests
    // ===============================================================

    // No arguments, no return type:
    fn export_void_function();

    // Passing primitives:
    fn export_primitive_bool(arg: bool) -> bool;
    fn export_primitive_f32(arg: f32) -> f32;
    fn export_primitive_f64(arg: f64) -> f64;
    fn export_primitive_i8(arg: i8) -> i8;
    fn export_primitive_i16(arg: i16) -> i16;
    fn export_primitive_i32(arg: i32) -> i32;
    fn export_primitive_i64(arg: i64) -> i64;
    fn export_primitive_u8(arg: u8) -> u8;
    fn export_primitive_u16(arg: u16) -> u16;
    fn export_primitive_u32(arg: u32) -> u32;
    fn export_primitive_u64(arg: u64) -> u64;

    // Passing arrays:
    fn export_array_u8(arg: [u8; 3]) -> [u8; 3];
    fn export_array_u16(arg: [u16; 3]) -> [u16; 3];
    fn export_array_u32(arg: [u32; 3]) -> [u32; 3];
    fn export_array_i8(arg: [i8; 3]) -> [i8; 3];
    fn export_array_i16(arg: [i16; 3]) -> [i16; 3];
    fn export_array_i32(arg: [i32; 3]) -> [i32; 3];
    fn export_array_f32(arg: [f32; 3]) -> [f32; 3];
    fn export_array_f64(arg: [f64; 3]) -> [f64; 3];

    // Passing strings:
    fn export_string(arg: String) -> String;

    // Multiple arguments:
    fn export_multiple_primitives(arg1: i8, arg2: String) -> i64;

    // Integration with the `time` crate:
    fn export_timestamp(arg: MyDateTime) -> MyDateTime;

    // Passing custom types with flattened properties.
    //
    // See `types/flattening.rs` for more info.
    fn export_fp_flatten(arg: FpFlatten) -> FpFlatten;
    fn export_serde_flatten(arg: SerdeFlatten) -> SerdeFlatten;

    // Generics.
    //
    // See `types/generics.rs` for more info.
    fn export_generics(arg: StructWithGenerics<u64>) -> StructWithGenerics<u64>;

    // Options
    fn export_struct_with_options(arg: StructWithOptions) -> StructWithOptions;

    // Custom type in a generic position.
    fn export_get_bytes() -> Result<Bytes, String>;
    fn export_get_serde_bytes() -> Result<ByteBuf, String>;

    // Passing custom types with property/variant renaming.
    //
    // See `types/renaming.rs` for more info.
    fn export_fp_struct(arg: FpPropertyRenaming) -> FpPropertyRenaming;
    fn export_fp_enum(arg: FpVariantRenaming) -> FpVariantRenaming;
    fn export_serde_struct(arg: SerdePropertyRenaming) -> SerdePropertyRenaming;
    fn export_serde_enum(arg: SerdeVariantRenaming) -> SerdeVariantRenaming;

    // Passing custom enums with different tagging options.
    //
    // See `types/tagged_enums.rs` for more info.
    fn export_fp_internally_tagged(arg: FpInternallyTagged) -> FpInternallyTagged;
    fn export_fp_adjacently_tagged(arg: FpAdjacentlyTagged) -> FpAdjacentlyTagged;
    fn export_fp_untagged(arg: FpUntagged) -> FpUntagged;
    fn export_serde_internally_tagged(arg: SerdeInternallyTagged) -> SerdeInternallyTagged;
    fn export_serde_adjacently_tagged(arg: SerdeAdjacentlyTagged) -> SerdeAdjacentlyTagged;
    fn export_serde_untagged(arg: SerdeUntagged) -> SerdeUntagged;

    // Async function:
    async fn export_async_struct(arg1: FpPropertyRenaming, arg2: u64) -> FpPropertyRenaming;

    /// Example how plugin could expose async data-fetching capabilities.
    async fn fetch_data(r#type: String) -> Result<String, String>;

    /// Called on the plugin to give it a chance to initialize.
    fn init();

    /// Example how plugin could expose a reducer.
    fn reducer_bridge(action: ReduxAction) -> StateUpdate;
}

const VERSION: &str = "1.0.0";
const AUTHORS: &str = r#"["Fiberplane <info@fiberplane.com>"]"#;
const NAME: &str = "example-bindings";

static PLUGIN_DEPENDENCIES: Lazy<BTreeMap<&str, CargoDependency>> = Lazy::new(|| {
    BTreeMap::from([
        (
            "redux-example",
            CargoDependency {
                path: Some("../../../redux-example"),
                features: BTreeSet::default(),
                ..Default::default()
            },
        ),
        (
            "fp-bindgen-support",
            CargoDependency {
                path: Some("../../../../fp-bindgen-support"),
                features: BTreeSet::from(["async", "guest"]),
                ..CargoDependency::default()
            },
        ),
        (
            "time",
            CargoDependency {
                version: Some("0.3"),
                features: BTreeSet::from(["macros"]),
                ..CargoDependency::default()
            },
        ),
    ])
});

fn main() {
    for bindings_type in [
        BindingsType::RustPlugin(RustPluginConfig {
            name: NAME,
            authors: AUTHORS,
            version: VERSION,
            dependencies: PLUGIN_DEPENDENCIES.clone(),
        }),
        BindingsType::RustWasmerRuntime,
        BindingsType::RustWasmerWasiRuntime,
        BindingsType::TsRuntimeWithExtendedConfig(
            TsExtendedRuntimeConfig::new()
                .with_msgpack_module("https://unpkg.com/@msgpack/msgpack@2.7.2/mod.ts")
                .with_raw_export_wrappers(),
        ),
    ] {
        let output_path = format!("bindings/{bindings_type}");

        fp_bindgen!(BindingConfig {
            bindings_type,
            path: &output_path,
        });
        println!("Generated bindings written to `{output_path}/`.");
    }
}

#[test]
fn test_generate_rust_plugin() {
    static FILES: &[(&str, &[u8])] = &[
        (
            "bindings/rust-plugin/src/types.rs",
            include_bytes!("assets/rust_plugin_test/expected_types.rs"),
        ),
        (
            "bindings/rust-plugin/src/lib.rs",
            include_bytes!("assets/rust_plugin_test/expected_lib.rs"),
        ),
        (
            "bindings/rust-plugin/src/export.rs",
            include_bytes!("assets/rust_plugin_test/expected_export.rs"),
        ),
        (
            "bindings/rust-plugin/src/import.rs",
            include_bytes!("assets/rust_plugin_test/expected_import.rs"),
        ),
        (
            "bindings/rust-plugin/Cargo.toml",
            include_bytes!("assets/rust_plugin_test/expected_Cargo.toml"),
        ),
    ];

    fp_bindgen!(BindingConfig {
        bindings_type: BindingsType::RustPlugin(RustPluginConfig {
            name: NAME,
            authors: AUTHORS,
            version: VERSION,
            dependencies: PLUGIN_DEPENDENCIES.clone(),
        }),
        path: "bindings/rust-plugin",
    });

    for (path, expected) in FILES {
        tests::assert_file_eq(path, expected)
    }
}

#[test]
fn test_generate_rust_wasmer_runtime() {
    static FILES: &[(&str, &[u8])] = &[
        (
            "bindings/rust-wasmer-runtime/bindings.rs",
            include_bytes!("assets/rust_wasmer_runtime_test/expected_bindings.rs"),
        ),
        (
            "bindings/rust-wasmer-runtime/types.rs",
            include_bytes!("assets/rust_wasmer_runtime_test/expected_types.rs"),
        ),
    ];
    fp_bindgen!(BindingConfig {
        bindings_type: BindingsType::RustWasmerRuntime,
        path: "bindings/rust-wasmer-runtime",
    });
    for (path, expected) in FILES {
        tests::assert_file_eq(path, expected)
    }
}

#[test]
fn test_generate_rust_wasmer_wasi_runtime() {
    static FILES: &[(&str, &[u8])] = &[
        (
            "bindings/rust-wasmer-wasi-runtime/bindings.rs",
            include_bytes!("assets/rust_wasmer_wasi_runtime_test/expected_bindings.rs"),
        ),
        (
            "bindings/rust-wasmer-wasi-runtime/types.rs",
            include_bytes!("assets/rust_wasmer_wasi_runtime_test/expected_types.rs"),
        ),
    ];
    fp_bindgen!(BindingConfig {
        bindings_type: BindingsType::RustWasmerWasiRuntime,
        path: "bindings/rust-wasmer-wasi-runtime",
    });
    for (path, expected) in FILES {
        tests::assert_file_eq(path, expected)
    }
}

#[test]
fn test_generate_ts_runtime() {
    static FILES: &[(&str, &[u8])] = &[
        (
            "bindings/ts-runtime/types.ts",
            include_bytes!("assets/ts_runtime_test/expected_types.ts"),
        ),
        (
            "bindings/ts-runtime/index.ts",
            include_bytes!("assets/ts_runtime_test/expected_index.ts"),
        ),
    ];

    fp_bindgen!(BindingConfig {
        bindings_type: BindingsType::TsRuntimeWithExtendedConfig(
            TsExtendedRuntimeConfig::new()
                .with_msgpack_module("https://unpkg.com/@msgpack/msgpack@2.7.2/mod.ts")
                .with_raw_export_wrappers()
        ),
        path: "bindings/ts-runtime",
    });

    for (path, expected) in FILES {
        tests::assert_file_eq(path, expected)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    pub fn assert_file_eq(path_of_actual: impl AsRef<Path>, expected_bytes: &[u8]) {
        let actual = std::fs::read_to_string(path_of_actual).expect("Cannot read `actual` file");
        let expected_code = String::from_utf8_lossy(expected_bytes);

        let actual_lines = actual.lines().collect::<Vec<_>>();
        let expected_lines = expected_code.lines().collect::<Vec<_>>();
        pretty_assertions::assert_eq!(actual_lines, expected_lines);
    }
}
