use wasmer::{Memory, ValueType, WasmPtr};

use crate::{
    common::{
        mem::{to_fat_ptr, FatPtr},
        r#async::{AsyncValue, FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
    },
    host::io::{from_fat_ptr, to_wasm_ptr},
};
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

// The GuestFuture implements the Future Trait to handle async Futures as
// returned from the guest.
pub struct GuestFuture {
    ptr: FatPtr,
    guest_mem: Memory,
    wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>,
}

impl GuestFuture {
    pub fn new(ptr: FatPtr, guest_mem: Memory, wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>) -> Self {
        Self {
            ptr,
            guest_mem,
            wakers,
        }
    }
}

unsafe impl ValueType for AsyncValue {}
/*
impl<'de> Future for GuestFuture {
    type Output = Vec<u8>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let (ptr, async_len) = from_fat_ptr(self.ptr);
        let async_ptr = WasmPtr::<AsyncValue>::new(ptr);

        let val = async_ptr.deref(&self.guest_mem).unwrap().get();

        match val.status {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.wakers.lock().unwrap();
                wakers.insert(self.ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result = import_from_guest_raw(&self.env, to_fat_ptr(val.ptr, val.len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
} */
