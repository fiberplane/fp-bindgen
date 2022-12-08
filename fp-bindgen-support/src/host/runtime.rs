use std::cell::UnsafeCell;
use crate::common::mem::FatPtr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use wasmer::{AsStoreMut, Instance, Memory, Store, TypedFunction};

#[derive(Default)]
pub struct RuntimeInstanceData {
    pub(crate) memory: Option<Memory>,
    pub(crate) wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>,
    __fp_free: Option<TypedFunction<FatPtr, ()>>,
    __fp_guest_resolve_async_value: Option<TypedFunction<(FatPtr, FatPtr), ()>>,
    __fp_malloc: Option<TypedFunction<u32, FatPtr>>,
}

impl RuntimeInstanceData {
    pub fn from_instance(store: &Store, instance: &Instance) -> Self {
        Self {
            memory: Some(instance.exports.get_memory("memory").unwrap().clone()),
            __fp_free: Some(instance.exports.get_typed_function(store, "__fp_free").unwrap()),
            __fp_guest_resolve_async_value: Some(instance.exports.get_typed_function(store, "__fp_guest_resolve_async_value").unwrap()),
            __fp_malloc: Some(instance.exports.get_typed_function(store, "__fp_malloc").unwrap()),
            ..Default::default()
        }
    }

    pub fn copy_from(&mut self, other: Self) {
        self.memory = other.memory;
        self.wakers = other.wakers;
        self.__fp_free = other.__fp_free;
        self.__fp_guest_resolve_async_value = other.__fp_guest_resolve_async_value;
        self.__fp_malloc = other.__fp_malloc;
    }

    pub fn guest_resolve_async_value(&self, async_ptr: FatPtr, result_ptr: FatPtr) {
        /*unsafe {
            self.__fp_guest_resolve_async_value
                .unwrap_unchecked()
                .call(async_ptr, result_ptr)
                .expect("Runtime error: Cannot resolve async value");
        }*/
        todo!()
    }

    pub fn malloc(&self, store: &mut impl AsStoreMut, len: u32) -> FatPtr {
        self.__fp_malloc
            .as_ref()
            .unwrap()
            .call(store, len)
            .expect("unable to call malloc")
    }

    pub fn free(&self, ptr: FatPtr) {
        /*unsafe {
            self.__fp_free
                .get_unchecked()
                .call(ptr)
                .expect("unable to call free")
        };*/
        todo!()
    }
}
