use fp_bindgen::prelude::*;
use std::collections::BTreeMap;

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

fp_import! {
    fn my_plain_imported_function(a: u32, b: u32) -> u32;

    fn my_complex_imported_function(a: ComplexGuestToHost) -> ComplexHostToGuest;

    async fn my_async_imported_function() -> ComplexHostToGuest;
}

fp_export! {
    fn my_plain_exported_function(a: u32, b: u32) -> u32;

    fn my_complex_exported_function(a: ComplexHostToGuest) -> ComplexGuestToHost;

    async fn my_async_exported_function() -> ComplexGuestToHost;
}

fn main() {
    let cmd = std::env::args().nth(1).expect("no command given");
    if cmd != "generate" {
        println!("Usage: cargo run generate <bindings-type>");
        return;
    }

    let bindings_type = std::env::args().nth(2).expect("no bindings type given");
    let output_path = format!("bindings/{}", bindings_type);

    fp_bindgen!(&bindings_type, &output_path);
}
