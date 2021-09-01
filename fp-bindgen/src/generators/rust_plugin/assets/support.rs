use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[doc(hidden)]
pub type FatPtr = u64;

#[doc(hidden)]
pub fn export_value_to_host<T: Serialize>(value: &T) -> FatPtr {
    let mut buffer = Vec::new();
    value
        .serialize(&mut Serializer::new(&mut buffer).with_struct_map())
        .expect("Serialization error");

    let len = buffer.len();

    if buffer.capacity() > len {
        buffer.shrink_to_fit();

        // If there is still no exact fit, we will perform a copy to guarantee
        // the capacity does not exceed the length. This is to make sure we
        // don't have to lie to `Vec::from_raw_parts()` in `__fp_free()` below:
        if buffer.capacity() > len {
            buffer = {
                let mut exact_buffer = Vec::with_capacity(len);
                exact_buffer.append(&mut buffer);
                exact_buffer
            }
        }
    }

    // Make sure the length marker does not run into our extension bits:
    if len & 0xff000000 != 0 {
        panic!("Buffer too large ({} bytes)", len);
    }

    let ptr = buffer.as_ptr();
    std::mem::forget(buffer);
    to_fat_ptr(ptr, len as u32)
}

/// # Safety
///
/// This function is only safe if passed a valid pointer given to us by the
/// host. After this call, the pointer is no longer valid.
#[doc(hidden)]
pub unsafe fn import_value_from_host<'de, T: Deserialize<'de>>(fat_ptr: FatPtr) -> T {
    let (ptr, len) = from_fat_ptr(fat_ptr);
    if len & 0xff000000 != 0 {
        panic!("Unknown extension bits");
    }

    let slice = std::slice::from_raw_parts(ptr, len as usize);
    let mut deserializer = Deserializer::new(slice);
    let value = T::deserialize(&mut deserializer).unwrap();

    __fp_free(fat_ptr);

    value
}

#[doc(hidden)]
pub fn to_fat_ptr(ptr: *const u8, len: u32) -> FatPtr {
    (ptr as FatPtr) << 32 | (len as FatPtr)
}

#[doc(hidden)]
pub fn from_fat_ptr(ptr: FatPtr) -> (*const u8, u32) {
    ((ptr >> 32) as *const u8, (ptr & 0xffffffff) as u32)
}

#[doc(hidden)]
pub fn malloc(len: u32) -> *const u8 {
    let vec = Vec::with_capacity(len as usize);
    let ptr = vec.as_ptr();
    std::mem::forget(vec);
    ptr
}

#[doc(hidden)]
#[no_mangle]
pub fn __fp_malloc(len: u32) -> FatPtr {
    to_fat_ptr(malloc(len), len)
}

/// # Safety
///
/// This function is only safe if passed a valid pointer from `__fp_malloc()`.
/// Any pointer returned by `__fp_malloc()` may only be passed exactly once,
/// after which it becomes invalid. Because this function can be called from
/// both the guest (us) and the host, we need to keep ownership rules in
/// account:
/// - When we allocate and pass to the host, the host frees the object.
/// - When the host allocates and passes to us, we free the object.
#[doc(hidden)]
#[no_mangle]
pub unsafe fn __fp_free(ptr: FatPtr) {
    let (ptr, len) = from_fat_ptr(ptr);

    if len & 0xff000000 != 0 {
        panic!("__fp_free() failed: unknown extension bits");
    }

    let vec = Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize);
    std::mem::drop(vec);
}
