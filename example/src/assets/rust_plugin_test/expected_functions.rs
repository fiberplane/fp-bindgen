use super::support::*;
use super::types::*;

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __gen_my_async_imported_function() -> FatPtr;

    fn __gen_my_complex_imported_function(a: FatPtr) -> FatPtr;

    fn __gen_my_plain_imported_function(a: u32, b: u32) -> u32;
}

pub async fn my_async_imported_function() -> ComplexHostToGuest {
    unsafe {
        let ret = __gen_my_async_imported_function();
        let result_ptr = HostFuture::new(ret).await;
        import_value_from_host(result_ptr)
    }
}

/// This one passes complex data types. Things are getting interesting.
pub fn my_complex_imported_function(a: ComplexGuestToHost) -> ComplexHostToGuest {
    let a = export_value_to_host(a);
    unsafe {
        let ret = __gen_my_complex_imported_function(a);
        import_value_from_host(ret)
    }
}

/// This is a very simple function that only uses primitives. Our bindgen should have little
/// trouble with this.
pub fn my_plain_imported_function(a: u32, b: u32) -> u32 {
    unsafe {
        __gen_my_plain_imported_function(a, b)
    }
}

macro_rules! fp_export {
    (async fn my_async_exported_function($($param:ident: $ty:ty),+) -> $ret:ty $body:block) => {
        #[no_mangle]
        pub __fp_gen_my_async_exported_function() -> FatPtr {
            let len = size_of::<r#async::AsyncValue>() as u32;
            let ptr = malloc(len);
            let fat_ptr = to_fat_ptr(ptr, len);
            let ptr = ptr as *mut r#async::AsyncValue;
            
            task::Task::spawn(Box::pin(async move {
                let ret = my_async_exported_function().await;
                unsafe {
                    let (result_ptr, result_len) =
                        from_fat_ptr(export_value_to_host::<ComplexGuestToHost>(&ret));
                    (*ptr).ptr = result_ptr as u32;
                    (*ptr).len = result_len;
                    (*ptr).status = STATUS_READY;
                    __fp_host_resolve_async_value(fat_ptr);
                }
            }));
            
            fat_ptr
        }

        async fn my_async_exported_function($($param: $ty),+) -> $ret $body
    };

    (fn my_complex_exported_function($($param:ident: $ty:ty),+) -> $ret:ty $body:block) => {
        #[no_mangle]
        pub __fp_gen_my_complex_exported_function(a: FatPtr) -> FatPtr {
            let a = unsafe { import_value_from_host::<ComplexHostToGuest>(a) };
            let ret = my_complex_exported_function(a);
            export_value_to_host::<ComplexGuestToHost>(ret)
        }

        fn my_complex_exported_function($($param: $ty),+) -> $ret $body
    };

    (fn my_plain_exported_function($($param:ident: $ty:ty),+) -> $ret:ty $body:block) => {
        #[no_mangle]
        pub __fp_gen_my_plain_exported_function(a: u32, b: u32) -> $ret {
            
            
            my_plain_exported_function(a, b)
        }

        fn my_plain_exported_function($($param: $ty),+) -> $ret $body
    };
}
