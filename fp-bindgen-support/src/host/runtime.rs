use crate::common::mem::FatPtr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use wasmer::{FunctionEnv, Instance, Memory, Store, StoreMut, TypedFunction};

#[derive(Clone, Default)]
pub struct RuntimeInstanceData {
    pub(crate) memory: Option<Memory>,
    pub(crate) wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>,
    __fp_free: Option<TypedFunction<FatPtr, ()>>,
    __fp_guest_resolve_async_value: Option<TypedFunction<(FatPtr, FatPtr), ()>>,
    __fp_malloc: Option<TypedFunction<u32, FatPtr>>,
}

impl RuntimeInstanceData {
    pub fn init_with_instance(&mut self, store: &mut Store, instance: &Instance) {
        self.memory = Some(instance.exports.get_memory("memory").unwrap().clone());
        self.__fp_free = Some(instance.exports.get_typed_function(store, "__fp_free").unwrap());
        self.__fp_guest_resolve_async_value= Some(instance.exports.get_typed_function(store, "__fp_guest_resolve_async_value").unwrap());
        self.__fp_malloc= Some(instance.exports.get_typed_function(store, "__fp_malloc").unwrap());
    }

    pub fn guest_resolve_async_value(&self) -> TypedFunction<(FatPtr, FatPtr), ()> {
            self.__fp_guest_resolve_async_value
                .as_ref()
                .unwrap()
    .clone()
    }

    pub fn malloc(&self) -> TypedFunction<u32, FatPtr> {
            self.__fp_malloc.as_ref().unwrap().clone()
    }

    pub fn free(&self) -> TypedFunction<FatPtr, ()> {
            self.__fp_free
                .as_ref()
                .unwrap()
                .clone()
    }
}
