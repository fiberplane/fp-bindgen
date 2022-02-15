use crate::common::{
    errors::FPGuestError,
    io::{deserialize_from_slice, serialize_to_vec},
    mem::*,
};
use serde::{de::DeserializeOwned, Serialize};
use std::alloc::Layout;

#[doc(hidden)]
pub fn export_value_to_host<T: Serialize>(value: &T) -> FatPtr {
    let mut buffer = serialize_to_vec(value);

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
pub unsafe fn import_value_from_host<T: DeserializeOwned>(
    fat_ptr: FatPtr,
) -> Result<T, FPGuestError> {
    let (ptr, len) = from_fat_ptr(fat_ptr);
    if len & 0xff000000 != 0 {
        return Err(FPGuestError::InvalidFatPtr);
    }

    let slice = std::slice::from_raw_parts(ptr, len as usize);
    let value = deserialize_from_slice(slice)?;

    __fp_free(fat_ptr);

    Ok(value)
}

const MALLOC_ALIGNMENT: usize = 16;

#[doc(hidden)]
#[no_mangle]
pub fn __fp_malloc(len: u32) -> FatPtr {
    let ptr = unsafe {
        std::alloc::alloc(
            Layout::from_size_align(len as usize, MALLOC_ALIGNMENT)
                .expect("Allocation failed unexpectedly, check requested allocation size"),
        )
    };
    to_fat_ptr(ptr, len)
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

    assert_eq!(
        len & 0xff000000,
        0,
        "__fp_free() failed: unknown extension bits"
    );

    std::alloc::dealloc(
        ptr as *mut u8,
        Layout::from_size_align(len as usize, MALLOC_ALIGNMENT)
            .expect("Deallocation failed unexpectedly, check the pointer is valid"),
    );
}
