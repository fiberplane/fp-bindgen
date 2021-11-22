use crate::common::mem::FatPtr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use wasmer::{LazyInit, Memory, NativeFunc, WasmerEnv};

#[derive(WasmerEnv, Clone)]
pub struct RuntimeInstanceData {
    #[wasmer(export)]
    pub(crate) memory: LazyInit<Memory>,

    pub(crate) wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>,

    #[wasmer(export)]
    __fp_free: LazyInit<NativeFunc<FatPtr>>,

    #[wasmer(export)]
    __fp_guest_resolve_async_value: LazyInit<NativeFunc<(FatPtr, FatPtr)>>,

    #[wasmer(export)]
    __fp_malloc: LazyInit<NativeFunc<u32, FatPtr>>,
}

impl RuntimeInstanceData {
    pub fn guest_resolve_async_value(&self, async_ptr: FatPtr, result_ptr: FatPtr) {
        unsafe {
            self.__fp_guest_resolve_async_value
                .get_unchecked()
                .call(async_ptr, result_ptr)
                .expect("Runtime error: Cannot resolve async value");
        }
    }

    pub fn malloc(&self, len: u32) -> FatPtr {
        unsafe {
            self.__fp_malloc
                .get_unchecked()
                .call(len)
                .expect("unable to call malloc")
        }
    }

    pub fn free(&self, ptr: FatPtr) {
        unsafe {
            self.__fp_free
                .get_unchecked()
                .call(ptr)
                .expect("unable to call free")
        };
    }
}

impl Default for RuntimeInstanceData {
    fn default() -> Self {
        Self {
            memory: Default::default(),
            wakers: Default::default(),
            __fp_free: Default::default(),
            __fp_guest_resolve_async_value: Default::default(),
            __fp_malloc: Default::default(),
        }
    }
}
