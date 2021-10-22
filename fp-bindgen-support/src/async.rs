use super::support::{from_fat_ptr, to_fat_ptr, FatPtr};
use once_cell::unsync::Lazy;
use std::collections::BTreeMap;
use std::future::Future;
use std::ptr::{read_volatile, write_volatile};
use std::task::{Context, Poll, Waker};

static mut WAKERS: Lazy<BTreeMap<FatPtr, Waker>> = Lazy::new(BTreeMap::new);

const STATUS_PENDING: u32 = 0;
const STATUS_READY: u32 = 1;

#[doc(hidden)]
#[repr(C)]
pub struct AsyncValue {
    pub status: u32,
    pub ptr: u32,
    pub len: u32,
}

impl AsyncValue {
    fn buffer_ptr(&self) -> FatPtr {
        to_fat_ptr(self.ptr as *const u8, self.len)
    }
}

/// Represents a future value that will be resolved by the host runtime.
pub(crate) struct HostFuture {
    ptr: FatPtr,
}

impl HostFuture {
    /// # Safety
    ///
    /// This function is only safe if passed a valid pointer to an `AsyncValue`
    /// created by the host. Only a single `HostFuture` may be created from such
    /// a pointer.
    pub unsafe fn new(async_value_ptr: FatPtr) -> Self {
        Self {
            ptr: async_value_ptr,
        }
    }
}

impl Future for HostFuture {
    type Output = FatPtr;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (ptr, _) = from_fat_ptr(self.ptr);
        let async_value = unsafe { read_volatile(ptr as *const AsyncValue) };
        match async_value.status {
            STATUS_PENDING => {
                unsafe {
                    WAKERS.insert(self.ptr, cx.waker().clone());
                }
                Poll::Pending
            }
            STATUS_READY => Poll::Ready(async_value.buffer_ptr()),
            status => panic!("Unexpected status: {}", status),
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub fn __fp_guest_resolve_async_value(async_value_ptr: FatPtr, result_ptr: FatPtr) {
    unsafe {
        if let Some(waker) = WAKERS.remove(&async_value_ptr) {
            // First assign the result ptr and mark the async value as ready:
            let (ptr, len) = from_fat_ptr(result_ptr);
            let (async_value_ptr, _) = from_fat_ptr(async_value_ptr);
            write_volatile(
                async_value_ptr as *mut AsyncValue,
                AsyncValue {
                    status: STATUS_READY,
                    ptr: ptr as u32,
                    len,
                },
            );

            waker.wake();
        }
    }
}
