use crate::common::mem::FatPtr;
use wasmer::WasmPtr;

/// Get a regular pointer and the length from a fat pointer
pub(crate) fn from_fat_ptr(ptr: FatPtr) -> (u32, u32) {
    ((ptr >> 32) as u32, (ptr & 0xffffffff) as u32)
}

/// Take a regular FatPtr and convert it to a WasmPtr (which makes it easier to
/// interact with the wasmer memory).
pub fn to_wasm_ptr<T>(ptr: FatPtr) -> (WasmPtr<T>, u32)
where
    T: Copy,
{
    let (ptr, len) = from_fat_ptr(ptr);
    (WasmPtr::new(ptr), len)
}

/// Create a fat pointer from a ptr and length
pub(crate) fn to_fat_ptr(ptr: u32, len: u32) -> FatPtr {
    (ptr as FatPtr) << 32 | (len as FatPtr)
}
