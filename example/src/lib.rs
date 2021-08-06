use fp_bindgen::prelude::*;
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
pub struct Simple {
    pub foo: i32,
    pub bar: String,
}

#[derive(Deserialize)]
pub struct ComplexImported {
    pub simple: Simple,
    pub list: Vec<f64>,
}

#[derive(Serialize)]
pub struct ComplexExported {
    pub simple: Simple,
    pub map: BTreeMap<String, Simple>,
}

#[fp_import]
fn my_plain_imported_function(a: u32, b: u32) -> u32;

#[fp_import]
fn my_complex_imported_function(a: ComplexExported) -> ComplexImported;

#[fp_import]
async fn my_async_imported_function() -> ComplexImported;

#[fp_export]
fn my_plain_exported_function(a: u32, b: u32) -> u32;

#[fp_export]
fn my_complex_imported_function(a: ComplexImported) -> ComplexExported;

#[fp_export]
async fn my_async_exported_function() -> ComplexExported;
