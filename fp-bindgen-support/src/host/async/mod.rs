#![allow(unused)]
use super::{
    io::{from_fat_ptr},
    runtime::RuntimeInstanceData,
};
use crate::common::{
    mem::FatPtr,
    r#async::{AsyncValue, FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
};
use std::{
    sync::Arc,
    task::Waker
};
use wasmer::FunctionEnvMut;
use crate::host::mem::{export_to_guest_raw, update_in_guest_raw};

pub mod future;

/// Create an empty FutureValue in the linear memory and return a FatPtr to it.
pub fn create_future_value(env: &mut FunctionEnvMut<Arc<RuntimeInstanceData>>) -> FatPtr {
    export_to_guest_raw(env, bytemuck::bytes_of(&AsyncValue {
        status: FUTURE_STATUS_PENDING,
        ptr: 0,
        len: 0,
    }))
}

/// Note: In this case we are only interested in the pointer itself, we do not
/// want to deserialize it (which would actually free it as well).
/// This function also doesn't call another function since everything is
/// contained in the env object.
pub fn resolve_async_value(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, async_value_ptr: FatPtr, result_ptr: FatPtr) {
    update_in_guest_raw(&mut env, async_value_ptr, |async_value: &mut AsyncValue| {
        let (result_ptr, result_len) = from_fat_ptr(result_ptr);

        async_value.status = FUTURE_STATUS_READY;
        async_value.ptr = result_ptr;
        async_value.len = result_len;
    });

    /*env.wakers
        .lock()
        .unwrap()
        .remove(&async_value_ptr)
        .as_ref()
        .map(Waker::wake_by_ref);*/
}
