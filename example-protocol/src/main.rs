use fp_bindgen::prelude::*;
use std::collections::{BTreeMap, HashMap};

#[derive(Serializable)]
pub struct DeadCode {
    pub you_wont_see_this: bool,
}

#[derive(Serializable)]
pub struct Simple {
    pub foo: i32,
    pub bar: String,
}

#[derive(Serializable)]
pub struct ComplexHostToGuest {
    pub simple: Simple,
    pub list: Vec<f64>,
}

#[derive(Serializable)]
pub struct ComplexGuestToHost {
    pub simple: Simple,
    pub map: BTreeMap<String, Simple>,
}

#[derive(Serializable)]
#[fp(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestMethod {
    Delete,
    Get,
    Options,
    Post,
    Update,
}

#[derive(Serializable)]
pub struct RequestOptions {
    pub url: String,
    pub method: RequestMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Serializable)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Serializable)]
#[fp(tag = "type", rename_all = "snake_case")]
pub enum RequestError {
    Offline,
    NoRoute,
    ConnectionRefused,
    Timeout,
    ServerError { status_code: u16, response: Vec<u8> },
    Other { reason: String },
}

fp_import! {
    /// Logs a message to the (development) console.
    fn log(message: String);

    /// This is a very simple function that only uses primitives. Our bindgen should have little
    /// trouble with this.
    fn my_plain_imported_function(a: u32, b: u32) -> u32;

    /// This one passes complex data types. Things are getting interesting.
    fn my_complex_imported_function(a: ComplexGuestToHost) -> ComplexHostToGuest;

    async fn my_async_imported_function() -> ComplexHostToGuest;

    async fn make_request(opts: RequestOptions) -> Result<Response, RequestError>;
}

fp_export! {
    fn my_plain_exported_function(a: u32, b: u32) -> u32;

    fn my_complex_exported_function(a: ComplexHostToGuest) -> ComplexGuestToHost;

    async fn my_async_exported_function() -> ComplexGuestToHost;

    async fn fetch_data(url: String) -> String;
}

fn main() {
    for bindings_type in ["rust-plugin", "ts-runtime"] {
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
