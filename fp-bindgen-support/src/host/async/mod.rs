pub mod future;

/// Create an empty FutureValue in the linear memory and return a FatPtr to it.
pub(crate) fn create_future_value(env: &RuntimeInstanceData) -> FatPtr {
    let memory = unsafe { env.memory.get_unchecked() };

    let size = size_of::<AsyncValue>();
    let ptr = unsafe {
        env.__fp_malloc
            .get_unchecked()
            .call(size as u32)
            .expect("runtime error")
    };

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
pub(crate) fn resolve_async_value(
    env: &RuntimeInstanceData,
    async_value_ptr: FatPtr,
    result_ptr: FatPtr,
) {
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
        .map(Waker::wake_by_ref);
}
