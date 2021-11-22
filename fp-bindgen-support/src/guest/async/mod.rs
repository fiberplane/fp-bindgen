mod queue;
pub mod task;
use crate::common::{
    mem::{from_fat_ptr, FatPtr},
    r#async::{AsyncValue, FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
};
use once_cell::unsync::Lazy;
use std::collections::BTreeMap;
use std::future::Future;
use std::ptr::{read_volatile, write_volatile};
use std::task::{Context, Poll, Waker};

static mut WAKERS: Lazy<BTreeMap<FatPtr, Waker>> = Lazy::new(BTreeMap::new);

/// Represents a future value that will be resolved by the host runtime.
pub struct HostFuture {
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
            FUTURE_STATUS_PENDING => {
                unsafe {
                    WAKERS.insert(self.ptr, cx.waker().clone());
                }
                Poll::Pending
            }
            FUTURE_STATUS_READY => Poll::Ready(async_value.buffer_ptr()),
            status => panic!("Unexpected status: {}", status),
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe fn __fp_guest_resolve_async_value(async_value_fat_ptr: FatPtr, result_ptr: FatPtr) {
    // First assign the result ptr and mark the async value as ready:
    let (ptr, len) = from_fat_ptr(result_ptr);
    let (async_value_ptr, _) = from_fat_ptr(async_value_fat_ptr);
    write_volatile(
        async_value_ptr as *mut AsyncValue,
        AsyncValue {
            status: FUTURE_STATUS_READY,
            ptr: ptr as u32,
            len,
        },
    );

    if let Some(waker) = WAKERS.remove(&async_value_fat_ptr) {
        waker.wake();
    }
}

#[link(wasm_import_module = "fp")]
extern "C" {
    fn __fp_host_resolve_async_value(async_value_ptr: FatPtr, result_ptr: FatPtr);
}

pub fn host_resolve_async_value(async_value_ptr: FatPtr, result_ptr: FatPtr) {
    unsafe { __fp_host_resolve_async_value(async_value_ptr, result_ptr) }
}
