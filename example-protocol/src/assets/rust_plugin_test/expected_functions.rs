use super::r#async::*;
use super::support::*;
use super::types::*;

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __fp_gen_count_words(string: FatPtr) -> FatPtr;

    fn __fp_gen_log(message: FatPtr);

    fn __fp_gen_make_request(opts: FatPtr) -> FatPtr;

    fn __fp_gen_my_async_imported_function() -> FatPtr;

    fn __fp_gen_my_complex_imported_function(a: FatPtr) -> FatPtr;

    fn __fp_gen_my_plain_imported_function(a: u32, b: u32) -> u32;

    pub fn __fp_host_resolve_async_value(async_value_ptr: FatPtr);
}

pub fn count_words(string: String) -> Result<u16, String> {
    let string = export_value_to_host(&string);
    unsafe {
        let ret = __fp_gen_count_words(string);
        import_value_from_host(ret)
    }
}

/// Logs a message to the (development) console.
pub fn log(message: String) {
    let message = export_value_to_host(&message);
    unsafe { __fp_gen_log(message); }
}

pub async fn make_request(opts: RequestOptions) -> Result<Response, RequestError> {
    let opts = export_value_to_host(&opts);
    unsafe {
        let ret = __fp_gen_make_request(opts);
        let result_ptr = HostFuture::new(ret).await;
        import_value_from_host(result_ptr)
    }
}

pub async fn my_async_imported_function() -> ComplexHostToGuest {
    unsafe {
        let ret = __fp_gen_my_async_imported_function();
        let result_ptr = HostFuture::new(ret).await;
        import_value_from_host(result_ptr)
    }
}

/// This one passes complex data types. Things are getting interesting.
pub fn my_complex_imported_function(a: ComplexAlias) -> ComplexHostToGuest {
    let a = export_value_to_host(&a);
    unsafe {
        let ret = __fp_gen_my_complex_imported_function(a);
        import_value_from_host(ret)
    }
}

/// This is a very simple function that only uses primitives. Our bindgen should have little
/// trouble with this.
pub fn my_plain_imported_function(a: u32, b: u32) -> u32 {
    unsafe { __fp_gen_my_plain_imported_function(a, b) }
}

#[macro_export]
macro_rules! fp_export {
    (async fn fetch_data$args:tt -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_fetch_data(url: __fp_macro::FatPtr) -> __fp_macro::FatPtr {
            use __fp_macro::*;
            let len = std::mem::size_of::<AsyncValue>() as u32;
            let ptr = malloc(len);
            let fat_ptr = to_fat_ptr(ptr, len);
            let ptr = ptr as *mut AsyncValue;

            Task::spawn(Box::pin(async move {
                let url = unsafe { import_value_from_host::<String>(url) };
                let ret = fetch_data(url).await;
                unsafe {
                    let (result_ptr, result_len) =
                       from_fat_ptr(export_value_to_host::<String>(&ret));
                    (*ptr).ptr = result_ptr as u32;
                    (*ptr).len = result_len;
                    (*ptr).status = 1;
                    __fp_host_resolve_async_value(fat_ptr);
                }
            }));

            fat_ptr
        }

        async fn fetch_data$args -> $ret $body
    };

    (async fn my_async_exported_function$args:tt -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_my_async_exported_function() -> __fp_macro::FatPtr {
            use __fp_macro::*;
            let len = std::mem::size_of::<AsyncValue>() as u32;
            let ptr = malloc(len);
            let fat_ptr = to_fat_ptr(ptr, len);
            let ptr = ptr as *mut AsyncValue;

            Task::spawn(Box::pin(async move {
                let ret = my_async_exported_function().await;
                unsafe {
                    let (result_ptr, result_len) =
                       from_fat_ptr(export_value_to_host::<ComplexGuestToHost>(&ret));
                    (*ptr).ptr = result_ptr as u32;
                    (*ptr).len = result_len;
                    (*ptr).status = 1;
                    __fp_host_resolve_async_value(fat_ptr);
                }
            }));

            fat_ptr
        }

        async fn my_async_exported_function$args -> $ret $body
    };

    (fn my_complex_exported_function$args:tt -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_my_complex_exported_function(a: __fp_macro::FatPtr) -> __fp_macro::FatPtr {
            use __fp_macro::*;
            let a = unsafe { import_value_from_host::<ComplexHostToGuest>(a) };
            let ret = my_complex_exported_function(a);
            export_value_to_host::<ComplexAlias>(&ret)
        }

        fn my_complex_exported_function$args -> $ret $body
    };

    (fn my_plain_exported_function$args:tt -> $ret:ty $body:block) => {
        #[no_mangle]
        pub fn __fp_gen_my_plain_exported_function(a: u32, b: u32) -> $ret {
            use __fp_macro::*;
            my_plain_exported_function(a, b)
        }

        fn my_plain_exported_function$args -> $ret $body
    };
}
