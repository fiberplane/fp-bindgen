use super::{
    io::{from_fat_ptr, to_wasm_ptr},
    runtime::RuntimeInstanceData,
};
use crate::common::{
    mem::FatPtr,
    r#async::{AsyncValue, FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
};
use std::{mem::size_of, task::Waker};
use wasmer::{AsStoreRef, AsStoreMut, FunctionEnvMut, Store};

pub mod future;

/// Create an empty FutureValue in the linear memory and return a FatPtr to it.
pub fn create_future_value(env: &mut FunctionEnvMut<RuntimeInstanceData>) -> FatPtr {
    let size = size_of::<AsyncValue>(); //TODO: Is this *actually* safe? Might be a different size in wasm land...
    let ptr = env.data().malloc().call(&mut env.as_store_mut(), size as u32).unwrap();

    let (async_ptr, async_len) = to_wasm_ptr(ptr);

    let memory = env.data().memory();
    let memory_view = memory.view(&env.as_store_ref());
    let values = async_ptr.slice(&memory_view, async_len).unwrap();

    values.write(0, FUTURE_STATUS_PENDING).unwrap();
    values.write(1, 0).unwrap();
    values.write(2, 0).unwrap();

    ptr
}

/// Note: In this case we are only interested in the pointer itself, we do not
/// want to deserialize it (which would actually free it as well).
/// This function also doesn't call another function since everything is
/// contained in the env object.
pub fn resolve_async_value(env: FunctionEnvMut<RuntimeInstanceData>, async_value_ptr: FatPtr, result_ptr: FatPtr) {
    // First assign the result ptr and mark the async value as ready:
    let memory = env.data().memory();
    let memory_view = memory.view(&env.as_store_ref());

    let (async_ptr, async_len) = to_wasm_ptr(async_value_ptr);
    let (result_ptr, result_len) = from_fat_ptr(result_ptr);
    let values = async_ptr.slice(&memory_view, async_len).unwrap();

    values.write(0, FUTURE_STATUS_READY).unwrap();
    values.write(1, result_ptr).unwrap();
    values.write(2, result_len).unwrap();

    env.data().wakers
        .lock()
        .unwrap()
        .remove(&async_value_ptr)
        .as_ref()
        .map(Waker::wake_by_ref);
}
