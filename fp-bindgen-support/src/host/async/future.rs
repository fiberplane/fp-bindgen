use crate::{
    common::{
        mem::FatPtr,
        r#async::{FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
    },
    host::{
        io::{to_fat_ptr, to_wasm_ptr},
        mem::import_from_guest_raw,
        runtime::RuntimeInstanceData,
    },
};
use std::{future::Future, task::Poll};
use wasmer::{AsStoreRef, FunctionEnvMut, Store};

// The ModuleRawFuture implements the Future Trait to handle async Futures as
// returned from the module.
pub struct ModuleRawFuture<'a> {
    env: FunctionEnvMut<'a, RuntimeInstanceData>,
    ptr: FatPtr,
}

impl<'a> ModuleRawFuture<'a> {
    pub fn new(env: FunctionEnvMut<'a, RuntimeInstanceData>, ptr: FatPtr) -> Self {
        Self { env, ptr }
    }
}

impl<'a, 'de> Future for ModuleRawFuture<'a> {
    type Output = Vec<u8>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let ptr = self.ptr;

        let (async_ptr, async_len) = to_wasm_ptr(ptr);
        let memory = self.env.data().memory.as_ref().unwrap();
        let memory_view = memory.view(&self.env.as_store_ref());
        let values = async_ptr.slice(&memory_view, async_len).unwrap();

        match values.read(0).unwrap() {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.env.data().wakers.lock().unwrap();
                wakers.insert(ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result_ptr = values.read(1).unwrap();
                let result_len = values.read(2).unwrap();
                let result = import_from_guest_raw(self.env.as_mut(), to_fat_ptr(result_ptr, result_len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
}
