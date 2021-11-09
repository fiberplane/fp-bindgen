use crate::common::mem::FatPtr;
use wasmer::{Array, WasmPtr};

/// Get a regular pointer and the length from a fat pointer
pub(crate) fn from_fat_ptr(ptr: FatPtr) -> (u32, u32) {
    ((ptr >> 32) as u32, (ptr & 0xffffffff) as u32)
}

/// Take a regular FatPtr and convert it to a WasmPtr (which makes it easier to
/// interact with the wasmer memory).
pub fn to_wasm_ptr<T>(ptr: FatPtr) -> (WasmPtr<T, Array>, u32)
where
    T: Copy,
{
    let (ptr, len) = from_fat_ptr(ptr);
    (WasmPtr::new(ptr), len)
}
