use super::support::{from_fat_ptr, to_fat_ptr};
use std::future::Future;

const STATUS_PENDING: u32 = 0;
const STATUS_READY: u32 = 1;

#[repr(C)]
struct AsyncValue {
    status: u32,
    ptr: u32,
    len: u32,
}

impl AsyncValue {
    fn buffer_ptr(&self) -> FatPtr {
        to_fat_ptr(self.ptr as *const u8, self.len)
    }
}

/// Represents a future value that will be resolved by the host runtime.
struct HostFuture {
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
        let async_value_ptr = ptr as *const AsyncValue;
        let status = unsafe { (*async_value_ptr).status };
        match status {
            STATUS_PENDING => {
                unsafe {
                    WAKERS.insert(self.ptr, cx.waker().clone());
                }
                Poll::Pending
            }
            STATUS_READY => Poll::Ready(unsafe { (*async_value_ptr).buffer_ptr() }),
            status => panic!("Unexpected status: {}", status),
        }
    }
}

#[doc(hidden)]
#[no_mangle]
pub fn __fp_guest_resolve_async_value(async_value_ptr: FatPtr) {
    unsafe {
        if let Some(waker) = WAKERS.remove(&async_value_ptr) {
            waker.wake();
        }
    }
}
