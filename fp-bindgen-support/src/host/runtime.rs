use std::borrow::Borrow;
use crate::common::mem::FatPtr;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use wasmer::{FunctionEnv, Instance, Memory, Store, StoreMut, TypedFunction};

#[derive(Clone, Default)]
pub struct RuntimeInstanceData {
    memory: Option<Memory>,
    pub(crate) wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>,
    __fp_free: Option<TypedFunction<FatPtr, ()>>,
    __fp_guest_resolve_async_value: Option<TypedFunction<(FatPtr, FatPtr), ()>>,
    __fp_malloc: Option<TypedFunction<u32, FatPtr>>,
}

impl RuntimeInstanceData {
    pub fn init_with_instance(&self, store: &Store, instance: &Instance) {
        let s = unsafe {
            &mut *(self as *const Self as *mut Self)
        };
        s.memory = Some(instance.exports.get_memory("memory").unwrap().clone());
        s.__fp_free = Some(instance.exports.get_typed_function(store, "__fp_free").unwrap());
        s.__fp_guest_resolve_async_value= Some(instance.exports.get_typed_function(store, "__fp_guest_resolve_async_value").unwrap());
        s.__fp_malloc= Some(instance.exports.get_typed_function(store, "__fp_malloc").unwrap());
    }

    pub(crate) fn memory(&self) -> &Memory {
        self.memory.as_ref().unwrap()
    }

    pub fn guest_resolve_async_value<'a>(&self) -> &'a TypedFunction<(FatPtr, FatPtr), ()> {
            let ptr = self.__fp_guest_resolve_async_value
                .as_ref()
                .unwrap()
            as *const TypedFunction<(FatPtr, FatPtr), ()>;
        unsafe {
            ptr.as_ref().unwrap()
        }
    }

    pub fn malloc<'a>(&self) -> &'a TypedFunction<u32, FatPtr> {
        let ptr = self.__fp_malloc.as_ref().unwrap() as *const TypedFunction<u32, FatPtr>;
        unsafe {
            ptr.as_ref().unwrap()
        }
    }

    pub fn free<'a>(&self) -> &'a TypedFunction<FatPtr, ()> {
            let ptr = self.__fp_free
                .as_ref()
                .unwrap()
            as *const TypedFunction<FatPtr, ()>;
        unsafe {
            ptr.as_ref().unwrap()
        }
    }
}
