use crate::common::mem::FatPtr;

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
