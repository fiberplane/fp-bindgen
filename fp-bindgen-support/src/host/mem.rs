use super::RuntimeInstanceData;
use rmp_serde::{decode::ReadReader, Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::future::Future;
use std::marker::PhantomData;
use std::mem::size_of;
use std::task::Poll;
use wasmer::{Array, WasmPtr};

/// Serialize an object from the linear memory and after that free up the memory
pub(crate) fn import_from_guest<'de, T: Deserialize<'de>>(
    env: &RuntimeInstanceData,
    fat_ptr: FatPtr,
) -> T {
    let value = import_from_guest_raw(env, fat_ptr);

    let mut deserializer = Deserializer::<ReadReader<&[u8]>>::new(value.as_ref());
    T::deserialize(&mut deserializer).unwrap()
}

/// Retrieve a serialized object from the linear memory as a Vec<u8> and free up
/// the memory it was using.
///
/// Useful when the consumer wants to pass the result, without having the
/// deserialize and serialize it.
fn import_from_guest_raw(env: &RuntimeInstanceData, fat_ptr: FatPtr) -> Vec<u8> {
    if fat_ptr == 0 {
        // This may happen with async calls that don't return a result:
        return Vec::new();
    }

    let memory = unsafe { env.memory.get_unchecked() };

    let (ptr, len) = to_wasm_ptr::<u8>(fat_ptr);
    if len & 0xff000000 != 0 {
        panic!("Unknown extension bits");
    }

    let value: Vec<u8> = {
        let view = ptr.deref(memory, 0, len).unwrap();
        view.iter().map(Cell::get).collect()
    };

    unsafe {
        env.__fp_free
            .get_unchecked()
            .call(fat_ptr)
            .expect("free should be called")
    };

    value
}

/// Serialize a value and put it in linear memory.
pub(crate) fn export_to_guest<T: Serialize>(env: &RuntimeInstanceData, value: &T) -> FatPtr {
    let mut buffer = Vec::new();
    value.serialize(&mut Serializer::new(&mut buffer)).unwrap();

    export_to_guest_raw(env, buffer)
}

/// Copy the buffer into linear memory.
pub(crate) fn export_to_guest_raw(env: &RuntimeInstanceData, buffer: Vec<u8>) -> FatPtr {
    let memory = unsafe { env.memory.get_unchecked() };

    let len = buffer.len() as u32;

    // Make sure the length marker does not run into our extension bits:
    if len & 0xff000000 != 0 {
        panic!("Buffer too large ({} bytes)", len);
    }

    let fat_ptr = unsafe {
        env.__fp_malloc
            .get_unchecked()
            .call(len)
            .expect("unable to call malloc")
    };

    let (ptr, len) = to_wasm_ptr(fat_ptr);

    let values = ptr.deref(memory, 0, len).unwrap();
    for (i, val) in buffer.iter().enumerate() {
        values[i].set(*val);
    }

    fat_ptr
}

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

// The ModuleFuture implements the Future Trait to handle async Futures as
// returned from the module.
pub(crate) struct ModuleFuture<T> {
    pub ptr: FatPtr,
    pub env: RuntimeInstanceData,

    _p: PhantomData<T>,
}

impl<T> ModuleFuture<T> {
    pub fn new(env: RuntimeInstanceData, ptr: FatPtr) -> Self {
        Self {
            ptr,
            env,
            _p: PhantomData,
        }
    }
}

impl<'de, T> Future for ModuleFuture<T>
where
    T: Deserialize<'de>,
{
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let memory = unsafe { self.env.memory.get_unchecked() };

        let ptr = self.ptr;

        let (async_ptr, async_len) = to_wasm_ptr(ptr);
        let values = async_ptr.deref(memory, 0, async_len).unwrap();

        match values[0].get() {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.env.wakers.lock().unwrap();
                wakers.insert(ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result_ptr = values[1].get();
                let result_len = values[2].get();
                let result = import_from_guest(&self.env, to_fat_ptr(result_ptr, result_len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
}

// The ModuleRawFuture implements the Future Trait to handle async Futures as
// returned from the module.
pub(crate) struct ModuleRawFuture {
    pub ptr: FatPtr,
    pub env: RuntimeInstanceData,
}

impl ModuleRawFuture {
    pub fn new(env: RuntimeInstanceData, ptr: FatPtr) -> Self {
        Self { ptr, env }
    }
}

impl<'de> Future for ModuleRawFuture {
    type Output = Vec<u8>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let memory = unsafe { self.env.memory.get_unchecked() };

        let ptr = self.ptr;

        let (async_ptr, async_len) = to_wasm_ptr(ptr);
        let values = async_ptr.deref(memory, 0, async_len).unwrap();

        match values[0].get() {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.env.wakers.lock().unwrap();
                wakers.insert(ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result_ptr = values[1].get();
                let result_len = values[2].get();
                let result = import_from_guest_raw(&self.env, to_fat_ptr(result_ptr, result_len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
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
    let waker = {
        let mut wakers = env.wakers.lock().unwrap();
        wakers.remove(&async_value_ptr)
    };

    match waker {
        Some(waker) => {
            // First assign the result ptr and mark the async value as ready:
            let memory = unsafe { env.memory.get_unchecked() };
            let (async_ptr, async_len) = to_wasm_ptr(async_value_ptr);
            let (result_ptr, result_len) = from_fat_ptr(result_ptr);
            let values = async_ptr.deref(memory, 0, async_len).unwrap();

            values[1].set(result_ptr);
            values[2].set(result_len);
            values[0].set(FUTURE_STATUS_READY);

            waker.wake_by_ref()
        }
        None => panic!("unknown async value"),
    }
}
