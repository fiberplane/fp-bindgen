#[doc(hidden)]
pub type FatPtr = u64;

#[doc(hidden)]
pub fn to_fat_ptr(ptr: *const u8, len: u32) -> FatPtr {
    (ptr as FatPtr) << 32 | (len as FatPtr)
}

#[doc(hidden)]
pub fn from_fat_ptr(ptr: FatPtr) -> (*const u8, u32) {
    ((ptr >> 32) as *const u8, (ptr & 0xffffffff) as u32)
}
