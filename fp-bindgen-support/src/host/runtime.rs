use crate::common::mem::FatPtr;
use std::convert::TryFrom;
use wasmer::{ExportError, Instance, NativeFunc};

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
