use chrono::{DateTime, Utc};
use fp_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub type Body = Vec<u8>;

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
    pub timestamp: DateTime<Utc>,
}

pub type ComplexAlias = ComplexGuestToHost;

#[derive(Serializable)]
pub struct ComplexGuestToHost {
    pub simple: Simple,
    pub map: BTreeMap<String, Simple>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "my_crate::other")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestMethod {
    Delete,
    Get,
    Options,
    Post,
    Put,
}

#[derive(Clone, Debug, Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "my_crate::prelude")]
#[serde(rename_all = "camelCase")]
pub struct RequestOptions {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none", with = "serde_bytes")]
    pub body: Option<Vec<u8>>,
}

/// A response to a request.
#[derive(Clone, Debug, Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "my_crate::prelude")]
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

fn main() {
    for bindings_type in ["rust-plugin", "rust-wasmer-runtime", "ts-runtime"] {
        let output_path = format!("bindings/{}", bindings_type);
        fp_bindgen!(bindings_type, &output_path);
        println!("Generated bindings written to `{}/`.", output_path);
    }
}

#[test]
fn test_generate_rust_plugin() {
    fp_bindgen!("rust-plugin", "bindings/rust-plugin");

    let generated_functions = std::fs::read_to_string("bindings/rust-plugin/functions.rs")
        .expect("Cannot read generated functions");
    let expected_functions = String::from_utf8_lossy(include_bytes!(
        "assets/rust_plugin_test/expected_functions.rs"
    ));
    tests::assert_lines_eq(&generated_functions, &expected_functions);

    let generated_types = std::fs::read_to_string("bindings/rust-plugin/types.rs")
        .expect("Cannot read generated types");
    let expected_types =
        String::from_utf8_lossy(include_bytes!("assets/rust_plugin_test/expected_types.rs"));
    tests::assert_lines_eq(&generated_types, &expected_types);

    let generated_mod = std::fs::read_to_string("bindings/rust-plugin/mod.rs")
        .expect("Cannot read generated mod.rs");
    let expected_mod =
        String::from_utf8_lossy(include_bytes!("assets/rust_plugin_test/expected_mod.rs"));
    tests::assert_lines_eq(&generated_mod, &expected_mod);
}

#[test]
fn test_generate_rust_wasmer_runtime() {
    fp_bindgen!("rust-wasmer-runtime", "bindings/rust-wasmer-runtime");

    let generated_functions =
        std::fs::read_to_string("bindings/rust-wasmer-runtime/spec/bindings.rs")
            .expect("Cannot read generated bindings");
    let expected_functions = String::from_utf8_lossy(include_bytes!(
        "assets/rust_wasmer_runtime_test/expected_bindings.rs"
    ));
    tests::assert_lines_eq(&generated_functions, &expected_functions);

    let generated_types = std::fs::read_to_string("bindings/rust-wasmer-runtime/spec/types.rs")
        .expect("Cannot read generated types");
    let expected_types = String::from_utf8_lossy(include_bytes!(
        "assets/rust_wasmer_runtime_test/expected_types.rs"
    ));
    tests::assert_lines_eq(&generated_types, &expected_types);
}

#[test]
fn test_generate_ts_runtime() {
    fp_bindgen!("ts-runtime", "bindings/ts-runtime");

    let generated_types = std::fs::read_to_string("bindings/ts-runtime/types.ts")
        .expect("Cannot read generated types");
    let expected_types =
        String::from_utf8_lossy(include_bytes!("assets/ts_runtime_test/expected_types.ts"));
    tests::assert_lines_eq(&generated_types, &expected_types);

    let generated_index = std::fs::read_to_string("bindings/ts-runtime/index.ts")
        .expect("Cannot read generated index.ts");
    let expected_index =
        String::from_utf8_lossy(include_bytes!("assets/ts_runtime_test/expected_index.ts"));
    tests::assert_lines_eq(&generated_index, &expected_index);
}

#[cfg(test)]
mod tests {
    pub fn assert_lines_eq(generated_code: &str, expected_code: &str) {
        let generated_lines = generated_code.split('\n').collect::<Vec<_>>();
        let expected_lines = expected_code.split('\n').collect::<Vec<_>>();
        pretty_assertions::assert_eq!(generated_lines, expected_lines);
    }
}
