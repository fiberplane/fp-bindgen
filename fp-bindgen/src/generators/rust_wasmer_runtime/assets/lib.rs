use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use support::FatPtr;
use wasmer::{LazyInit, Memory, Module, NativeFunc, WasmerEnv};

mod errors;
pub mod spec;
mod support;

pub struct Runtime {
    module: Module,
}

impl Runtime {
    pub fn new(
        store: wasmer::Store,
        wasm_module: impl AsRef<[u8]>,
    ) -> Result<Self, errors::RuntimeError> {
        let module = Module::new(&store, wasm_module)?;

        Ok(Self { module })
    }
}

#[derive(WasmerEnv, Clone)]
pub struct RuntimeInstanceData {
    #[wasmer(export)]
    memory: LazyInit<Memory>,

    pub(crate) wakers: Arc<Mutex<HashMap<FatPtr, Waker>>>,

    #[wasmer(export)]
    pub(crate) __fp_free: LazyInit<NativeFunc<FatPtr>>,

    #[wasmer(export)]
    pub(crate) __fp_guest_resolve_async_value: LazyInit<NativeFunc<FatPtr>>,

    #[wasmer(export)]
    pub(crate) __fp_malloc: LazyInit<NativeFunc<u32, FatPtr>>,
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
