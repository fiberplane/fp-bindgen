use super::{io::to_wasm_ptr, runtime::RuntimeInstanceData};
use crate::common::mem::FatPtr;
use rmp_serde::{decode::ReadReader, Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use wasmer::{AsStoreRef, AsStoreMut, FunctionEnvMut, Store};

/// Serialize the given value to MessagePack
pub fn serialize_to_vec<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut serializer = Serializer::new(&mut buffer)
        .with_struct_map()
        .with_human_readable();
    value.serialize(&mut serializer).unwrap();
    buffer
}

/// Deserialize the given MessagePack-encoded slice
pub fn deserialize_from_slice<'a, T: Deserialize<'a>>(slice: &'a [u8]) -> T {
    let mut deserializer = rmp_serde::Deserializer::new(slice).with_human_readable();
    T::deserialize(&mut deserializer).unwrap()
}

/// Serialize an object from the linear memory and after that free up the memory
pub fn import_from_guest<'de, T: Deserialize<'de>>(
    env: &mut FunctionEnvMut<RuntimeInstanceData>,
    fat_ptr: FatPtr,
) -> T {
    let value = import_from_guest_raw(env, fat_ptr);

    let mut deserializer =
        Deserializer::<ReadReader<&[u8]>>::new(value.as_ref()).with_human_readable();
    T::deserialize(&mut deserializer).unwrap()
}

/// Retrieve a serialized object from the linear memory as a Vec<u8> and free up
/// the memory it was using.
///
/// Useful when the consumer wants to pass the result, without having the
/// deserialize and serialize it.
pub fn import_from_guest_raw(env: &mut FunctionEnvMut<RuntimeInstanceData>, fat_ptr: FatPtr) -> Vec<u8> {
    if fat_ptr == 0 {
        // This may happen with async calls that don't return a result:
        return Vec::new();
    }

    let (ptr, len) = to_wasm_ptr::<u8>(fat_ptr);
    if len & 0xff000000 != 0 {
        panic!("Unknown extension bits");
    }

    let memory = env.data().memory();
    let memory_view = memory.view(&env.as_store_ref());
    let value = ptr.slice(&memory_view, len).unwrap().read_to_vec().unwrap();

    env.data().free().call(&mut env.as_store_mut(), fat_ptr).unwrap();

    value
}

/// Serialize a value and put it in linear memory.
pub fn export_to_guest<T: Serialize>(env: &mut FunctionEnvMut<RuntimeInstanceData>, value: &T) -> FatPtr {
    export_to_guest_raw( env, rmp_serde::to_vec(value).unwrap())
}

/// Copy the buffer into linear memory.
pub fn export_to_guest_raw(env: &mut FunctionEnvMut<RuntimeInstanceData>, buffer: Vec<u8>) -> FatPtr {
    let len = buffer.len() as u32;

    // Make sure the length marker does not run into our extension bits:
    if len & 0xff000000 != 0 {
        panic!("Buffer too large ({} bytes)", len);
    }

    let fat_ptr = env.data().malloc().call(&mut env.as_store_mut(), len).unwrap();

    let (ptr, len) = to_wasm_ptr(fat_ptr);

    let memory = env.data().memory();
    let memory_view = memory.view(&env.as_store_ref());
    let values = ptr.slice(&memory_view, len).unwrap();
    for (i, val) in buffer.iter().enumerate() {
        values.write(i as u64, *val).unwrap();
    }

    fat_ptr
}
