use bytemuck::{Pod, Zeroable};
use super::mem::{to_fat_ptr, FatPtr};

pub const FUTURE_STATUS_PENDING: u32 = 0;
pub const FUTURE_STATUS_READY: u32 = 1;

#[doc(hidden)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AsyncValue {
    pub status: u32,
    pub ptr: u32,
    pub len: u32,
}

impl AsyncValue {
    pub fn new() -> Self {
        Self {
            status: FUTURE_STATUS_PENDING,
            ptr: 0,
            len: 0,
        }
    }

    pub fn buffer_ptr(&self) -> FatPtr {
        to_fat_ptr(self.ptr as *const u8, self.len)
    }
}

impl Default for AsyncValue {
    fn default() -> Self {
        Self::new()
    }
}
