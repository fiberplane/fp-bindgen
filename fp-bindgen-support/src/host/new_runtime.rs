use super::io::{from_fat_ptr, to_wasm_ptr};
use crate::common::mem::FatPtr;
use rmp_serde::Serializer;
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::TryFrom, marker::PhantomData};
use wasmer::{Array, ExportError, Instance, NativeFunc, WasmPtr};

trait Runtime {
    fn malloc(size: u32) -> FatPtr;
    fn free(ptr: FatPtr);
}

trait AsyncRuntime: Runtime {
    fn resolve_async(async_value_ptr: FatPtr, result_ptr: FatPtr);
}

#[derive(Clone)]
struct RequiredGuestFunctions {
    malloc: NativeFunc<u32, FatPtr>,
    free: NativeFunc<FatPtr, ()>,
    async_resolve: NativeFunc<(FatPtr, FatPtr), ()>,
}

impl TryFrom<&Instance> for RequiredGuestFunctions {
    type Error = ExportError;

    fn try_from(ins: &Instance) -> Result<Self, Self::Error> {
        Ok(Self {
            malloc: ins.exports.get_native_function("__fp_gen_malloc")?,
            free: ins.exports.get_native_function("__fp_gen_free")?,
            async_resolve: ins
                .exports
                .get_native_function("__fp_guest_resolve_async_value")?,
        })
    }
}

#[derive(Clone)]
pub struct FPRuntime {
    funcs: RequiredGuestFunctions,
}

//TODO: investigate adding error handling for runtime failures
impl FPRuntime {
    pub fn new(instance: &Instance) -> Result<Self, ExportError> {
        Ok(Self {
            funcs: RequiredGuestFunctions::try_from(instance)?,
        })
    }

    pub fn malloc(&self, size: u32) -> FatPtr {
        self.funcs.malloc.call(size).unwrap()
    }
    pub fn free(&self, ptr: FatPtr) {
        self.funcs.free.call(ptr).unwrap()
    }
    pub fn async_resolve(&self, async_value_fat_ptr: FatPtr, result_ptr: FatPtr) {
        self.funcs
            .async_resolve
            .call(async_value_fat_ptr, result_ptr)
            .unwrap()
    }
}

pub struct GuestPtr<T: DeserializeOwned> {
    runtime: FPRuntime,
    ptr: FatPtr,
    _p: PhantomData<T>,
}

impl<T: DeserializeOwned + Serialize> GuestPtr<T> {
    pub fn fetch(self) -> T {
        todo!()
    }
    pub fn insert(runtime: &FPRuntime, value: T) -> GuestPtr<T> {
        let mut data = Vec::with_capacity(std::mem::size_of::<T>()); //we at least need sizeof(T) bytes
        value.serialize(&mut Serializer::new(&mut data)).unwrap();

        let ptr = runtime.malloc(data.len() as u32);

        let (ptr, len) = from_fat_ptr(ptr);
        let wptr = WasmPtr::<u8, Array>::new(ptr);

        todo!()
    }
}
