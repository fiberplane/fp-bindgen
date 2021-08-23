use super::support::*;
use super::types::*;

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __gen_my_async_imported_function() -> FatPtr;

    fn __gen_my_complex_imported_function(a: FatPtr) -> FatPtr;

    fn __gen_my_plain_imported_function(a: u32, b: u32) -> u32;
}

pub async fn my_async_imported_function() -> ComplexHostToGuest {
    let ret = __gen_my_async_imported_function();
    unsafe {
        import_value_from_host(ret)
    }
}

pub fn my_complex_imported_function(a: ComplexGuestToHost) -> ComplexHostToGuest {
    let a = export_value_to_host(a);
    let ret = __gen_my_complex_imported_function(a);
    unsafe {
        import_value_from_host(ret)
    }
}

pub fn my_plain_imported_function(a: u32, b: u32) -> u32 {
    __gen_my_plain_imported_function(a, b)
}
