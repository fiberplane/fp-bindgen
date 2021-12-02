use fp_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use std::collections::{BTreeMap, HashMap};
use time::OffsetDateTime;
use uuid::Uuid;

pub type Body = ByteBuf;

#[derive(Serializable)]
pub struct DeadCode {
    pub you_wont_see_this: bool,
}

#[derive(Serializable)]
pub struct Point<T> {
    pub value: T,
}

#[derive(Serializable)]
pub struct Simple {
    pub foo: i32,
    pub bar: String,
}

/// Multi-line doc comment with complex characters
/// & " , \ ! '
#[derive(Serializable)]
pub struct ComplexHostToGuest {
    pub simple: Simple,
    pub list: Vec<f64>,
    pub points: Vec<Point<f64>>,
    pub recursive: Vec<Point<Point<f64>>>,
    pub complex_nested: Option<BTreeMap<String, Vec<Point<f64>>>>,
    pub timestamp: OffsetDateTime,
    #[fp(rename = "optional_timestamp")]
    pub renamed: Option<OffsetDateTime>,
    /// Raw identifiers are supported too.
    pub r#type: String,
    pub id: Uuid,
}

pub type ComplexAlias = ComplexGuestToHost;

#[derive(Serializable)]
pub struct ComplexGuestToHost {
    pub simple: Simple,
    pub map: BTreeMap<String, Simple>,
    pub timestamp: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "example_bindings")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestMethod {
    Delete,
    Get,
    Options,
    Post,
    Put,
}

#[derive(Clone, Debug, Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "example_bindings")]
#[serde(rename_all = "camelCase")]
pub struct RequestOptions {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<ByteBuf>,
}

/// A response to a request.
#[derive(Clone, Debug, Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "example_bindings")]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Response headers, by name.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Body,
}

/// Represents an error with the request.
#[derive(Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    /// Used when we know we don't have an active network connection.
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    ServerError {
        /// HTTP status code.
        status_code: u16,
        /// Response body.
        response: Body,
    },
    /// Misc.
    #[fp(rename = "other/misc")]
    Other {
        reason: String,
    },
}

#[derive(Serializable)]
pub struct ExplicitedlyImportedType {
    pub you_will_see_this: bool,
}

mod foobar {
    use fp_bindgen::prelude::*;
    pub mod baz {
        use fp_bindgen::prelude::*;
        #[derive(Serializable)]
        pub struct GroupImportedType1 {
            pub you_will_see_this: bool,
        }
    }
    #[derive(Serializable)]
    pub struct GroupImportedType2 {
        pub you_will_see_this: bool,
    }
}

fp_import! {
    use ExplicitedlyImportedType;

    use foobar::{baz::GroupImportedType1, GroupImportedType2};

    /// Logs a message to the (development) console.
    fn log(message: String);

    /// This is a very simple function that only uses primitives. Our bindgen should have little
    /// trouble with this.
    fn my_plain_imported_function(a: u32, b: u32) -> u32;

    /// This one passes complex data types. Things are getting interesting.
    fn my_complex_imported_function(a: ComplexAlias) -> ComplexHostToGuest;

    fn count_words(string: String) -> Result<u16, String>;

    async fn my_async_imported_function() -> ComplexHostToGuest;

    async fn make_request(opts: RequestOptions) -> Result<Response, RequestError>;
}

fp_export! {
    use ExplicitedlyImportedType;

    fn my_plain_exported_function(a: u32, b: u32) -> u32;

    fn my_complex_exported_function(a: ComplexHostToGuest) -> ComplexAlias;

    async fn my_async_exported_function() -> ComplexGuestToHost;

    async fn fetch_data(url: String) -> String;
}

const VERSION: &str = "1.0.0";
const AUTHORS: &str = r#"["Fiberplane <info@fiberplane.com>"]"#;
const NAME: &str = "example-bindings";

fn main() {
    for bindings_type in [
        BindingsType::RustPlugin(RustPluginConfig {
            name: NAME,
            authors: AUTHORS,
            version: VERSION,
            dependencies: BTreeMap::from([(
                "fp-bindgen-support".to_owned(),
                r#"{ path = "../../fp-bindgen-support", features = ["guest", "async"] }"#
                    .to_owned(),
            )]),
        }),
        BindingsType::RustWasmerRuntime,
        BindingsType::TsRuntime(TsRuntimeConfig {
            generate_raw_export_wrappers: true,
        }),
    ] {
        let output_path = format!("bindings/{}", bindings_type);

        fp_bindgen!(BindingConfig {
            bindings_type,
            path: &output_path,
        });
        println!("Generated bindings written to `{}/`.", output_path);
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
            dependencies: BTreeMap::from([(
                "fp-bindgen-support".to_owned(),
                r#"{ path = "../../fp-bindgen-support", features = ["async"] }"#.to_owned()
            )])
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
            "bindings/rust-wasmer-runtime/spec/bindings.rs",
            include_bytes!("assets/rust_wasmer_runtime_test/expected_bindings.rs"),
        ),
        (
            "bindings/rust-wasmer-runtime/spec/types.rs",
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
        bindings_type: BindingsType::TsRuntime(TsRuntimeConfig {
            generate_raw_export_wrappers: true
        }),
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

        let actual_lines = actual.split('\n').collect::<Vec<_>>();
        let expected_lines = expected_code.split('\n').collect::<Vec<_>>();
        pretty_assertions::assert_eq!(actual_lines, expected_lines);
    }
}
