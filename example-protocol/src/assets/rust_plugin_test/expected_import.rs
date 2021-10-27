use crate::types::*;
 
            
#[link(wasm_import_module = "fp")]
extern "C" {
    fn __fp_gen_count_words(string: fp_bindgen_support::FatPtr) -> fp_bindgen_support::FatPtr;

    fn __fp_gen_log(message: fp_bindgen_support::FatPtr);

    fn __fp_gen_make_request(opts: fp_bindgen_support::FatPtr) -> fp_bindgen_support::FatPtr;

    fn __fp_gen_my_async_imported_function() -> fp_bindgen_support::FatPtr;

    fn __fp_gen_my_complex_imported_function(a: fp_bindgen_support::FatPtr) -> fp_bindgen_support::FatPtr;

    fn __fp_gen_my_plain_imported_function(a: u32, b: u32) -> u32;
}

pub fn count_words(string: String) -> Result<u16, String> {
    let string = fp_bindgen_support::export_value_to_host(&string);
    unsafe {
        let ret = __fp_gen_count_words(string);
        fp_bindgen_support::import_value_from_host(ret)
    }
}

/// Logs a message to the (development) console.
pub fn log(message: String) {
    let message = fp_bindgen_support::export_value_to_host(&message);
    unsafe { __fp_gen_log(message); }
}

pub async fn make_request(opts: RequestOptions) -> Result<Response, RequestError> {
    let opts = fp_bindgen_support::export_value_to_host(&opts);
    unsafe {
        let ret = __fp_gen_make_request(opts);
        let result_ptr = fp_bindgen_support::HostFuture::new(ret).await;
        fp_bindgen_support::import_value_from_host(result_ptr)
    }
}

pub async fn my_async_imported_function() -> ComplexHostToGuest {
    unsafe {
        let ret = __fp_gen_my_async_imported_function();
        let result_ptr = fp_bindgen_support::HostFuture::new(ret).await;
        fp_bindgen_support::import_value_from_host(result_ptr)
    }
}

/// This one passes complex data types. Things are getting interesting.
pub fn my_complex_imported_function(a: ComplexAlias) -> ComplexHostToGuest {
    let a = fp_bindgen_support::export_value_to_host(&a);
    unsafe {
        let ret = __fp_gen_my_complex_imported_function(a);
        fp_bindgen_support::import_value_from_host(ret)
    }
}

/// This is a very simple function that only uses primitives. Our bindgen should have little
/// trouble with this.
pub fn my_plain_imported_function(a: u32, b: u32) -> u32 {
    unsafe { __fp_gen_my_plain_imported_function(a, b) }
}
