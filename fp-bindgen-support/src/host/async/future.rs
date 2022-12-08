use crate::common::r#async::AsyncValue;
use crate::{
    common::{
        mem::FatPtr,
        r#async::{FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
    },
    host::{io::to_fat_ptr, mem::import_from_guest_raw, runtime::RuntimeInstanceData},
};
use bytemuck::from_bytes;
use std::sync::Arc;
use std::{future::Future, task::Poll};
use wasmer::FunctionEnvMut;

// The ModuleRawFuture implements the Future Trait to handle async Futures as
// returned from the module.
pub struct ModuleRawFuture<'a> {
    ptr: FatPtr,
    env: FunctionEnvMut<'a, Arc<RuntimeInstanceData>>,
}

impl<'a> ModuleRawFuture<'a> {
    pub fn new(env: FunctionEnvMut<'a, Arc<RuntimeInstanceData>>, ptr: FatPtr) -> Self {
        Self { ptr, env }
    }
}

impl<'a> Future for ModuleRawFuture<'a> {
    type Output = Vec<u8>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let ptr = self.ptr;
        let bytes = import_from_guest_raw(&mut self.env, ptr);
        let async_value = *from_bytes::<AsyncValue>(&bytes);
        match async_value.status {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.env.data().wakers.lock().unwrap();
                wakers.insert(ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result = import_from_guest_raw(&mut self.env, to_fat_ptr(async_value.ptr, async_value.len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
}
