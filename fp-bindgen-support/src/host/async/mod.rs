use super::{
    io::{from_fat_ptr, to_wasm_ptr},
    runtime::RuntimeInstanceData,
};
use crate::common::{
    mem::FatPtr,
    r#async::{AsyncValue, FUTURE_STATUS_PENDING, FUTURE_STATUS_READY},
};
use std::{mem::size_of, task::Waker};

pub mod future;

/// Create an empty FutureValue in the linear memory and return a FatPtr to it.
pub fn create_future_value(env: &RuntimeInstanceData) -> FatPtr {
    let memory = unsafe { env.memory.get_unchecked() };

    let size = size_of::<AsyncValue>(); //TODO: Is this *actually* safe? Might be a different size in wasm land...
    let ptr = env.malloc(size as u32);

    let (async_ptr, async_len) = to_wasm_ptr(ptr);
    let values = async_ptr.deref(memory, 0, async_len).unwrap();

    values[0].set(FUTURE_STATUS_PENDING);
    values[1].set(0);
    values[2].set(0);

    ptr
}

/// Note: In this case we are only interested in the pointer itself, we do not
/// want to deserialize it (which would actually free it as well).
/// This function also doesn't call another function since everything is
/// contained in the env object.
pub fn resolve_async_value(env: &RuntimeInstanceData, async_value_ptr: FatPtr, result_ptr: FatPtr) {
    // First assign the result ptr and mark the async value as ready:
    let memory = unsafe { env.memory.get_unchecked() };
    let (async_ptr, async_len) = to_wasm_ptr(async_value_ptr);
    let (result_ptr, result_len) = from_fat_ptr(result_ptr);
    let values = async_ptr.deref(memory, 0, async_len).unwrap();

    values[0].set(FUTURE_STATUS_READY);
    values[1].set(result_ptr);
    values[2].set(result_len);

    env.wakers
        .lock()
        .unwrap()
        .remove(&async_value_ptr)
        .as_ref()
        .map(Waker::wake_by_ref);
}
