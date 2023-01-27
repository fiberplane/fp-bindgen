use super::types::*;
use fp_bindgen_support::{
    common::{abi::WasmAbi, mem::FatPtr},
    host::{
        errors::{InvocationError, RuntimeError},
        mem::{
            deserialize_from_slice, export_to_guest, export_to_guest_raw, import_from_guest,
            import_from_guest_raw, serialize_to_vec,
        },
        r#async::{create_future_value, future::ModuleRawFuture, resolve_async_value},
        runtime::RuntimeInstanceData,
    },
};
use std::cell::RefCell;
use wasmer::{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv};

#[derive(Clone)]
pub struct Runtime {
    instance: Instance,
    env: RuntimeInstanceData,
}

impl Runtime {
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(module.store(), &env);
        let instance = Instance::new(&module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        Ok(Self { instance, env })
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn default_store() -> wasmer::Store {
        let compiler = wasmer::Cranelift::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn default_store() -> wasmer::Store {
        let compiler = wasmer::Singlepass::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    pub fn export_array_f32(&self, arg: [f32; 3]) -> Result<[f32; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_f32_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_f32_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_f32")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_f32".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_f64(&self, arg: [f64; 3]) -> Result<[f64; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_f64_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_f64_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_f64")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_f64".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_i16(&self, arg: [i16; 3]) -> Result<[i16; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_i16_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_i16_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_i16")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_i16".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_i32(&self, arg: [i32; 3]) -> Result<[i32; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_i32_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_i32_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_i32")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_i32".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_i8(&self, arg: [i8; 3]) -> Result<[i8; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_i8_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_i8_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_i8")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_i8".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_u16(&self, arg: [u16; 3]) -> Result<[u16; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_u16_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_u16_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_u16")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_u16".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_u32(&self, arg: [u32; 3]) -> Result<[u32; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_u32_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_u32_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_u32")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_u32".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_array_u8(&self, arg: [u8; 3]) -> Result<[u8; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_u8_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_u8_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_array_u8")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_u8".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub async fn export_async_struct(
        &self,
        arg1: FpPropertyRenaming,
        arg2: u64,
    ) -> Result<FpPropertyRenaming, InvocationError> {
        let arg1 = serialize_to_vec(&arg1);
        let result = self.export_async_struct_raw(arg1, arg2);
        let result = result.await;
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub async fn export_async_struct_raw(
        &self,
        arg1: Vec<u8>,
        arg2: u64,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg1 = export_to_guest_raw(&self.env, arg1);
        let function = self
            .instance
            .exports
            .get_native_function::<(FatPtr, <u64 as WasmAbi>::AbiType), FatPtr>(
                "__fp_gen_export_async_struct",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_async_struct".to_owned())
            })?;
        let result = function.call(arg1.to_abi(), arg2.to_abi())?;
        let result = ModuleRawFuture::new(self.env.clone(), result).await;
        Ok(result)
    }

    pub fn export_fp_adjacently_tagged(
        &self,
        arg: FpAdjacentlyTagged,
    ) -> Result<FpAdjacentlyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_adjacently_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_adjacently_tagged_raw(
        &self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_fp_adjacently_tagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_fp_adjacently_tagged".to_owned(),
                )
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_fp_enum(
        &self,
        arg: FpVariantRenaming,
    ) -> Result<FpVariantRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_enum_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_enum_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_fp_enum")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_enum".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_fp_flatten(&self, arg: FpFlatten) -> Result<FpFlatten, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_flatten_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_flatten_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_fp_flatten")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_flatten".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_fp_internally_tagged(
        &self,
        arg: FpInternallyTagged,
    ) -> Result<FpInternallyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_internally_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_internally_tagged_raw(
        &self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_fp_internally_tagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_fp_internally_tagged".to_owned(),
                )
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_fp_struct(
        &self,
        arg: FpPropertyRenaming,
    ) -> Result<FpPropertyRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_struct_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_struct_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_fp_struct")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_struct".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_fp_untagged(&self, arg: FpUntagged) -> Result<FpUntagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_untagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_untagged_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_fp_untagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_untagged".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_generics(
        &self,
        arg: StructWithGenerics<u64>,
    ) -> Result<StructWithGenerics<u64>, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_generics_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_generics_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_generics")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_generics".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_get_bytes(&self) -> Result<Result<bytes::Bytes, String>, InvocationError> {
        let result = self.export_get_bytes_raw();
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_get_bytes_raw(&self) -> Result<Vec<u8>, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<(), FatPtr>("__fp_gen_export_get_bytes")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_get_bytes".to_owned())
            })?;
        let result = function.call()?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_get_serde_bytes(
        &self,
    ) -> Result<Result<serde_bytes::ByteBuf, String>, InvocationError> {
        let result = self.export_get_serde_bytes_raw();
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_get_serde_bytes_raw(&self) -> Result<Vec<u8>, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<(), FatPtr>("__fp_gen_export_get_serde_bytes")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_get_serde_bytes".to_owned())
            })?;
        let result = function.call()?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_multiple_primitives(
        &self,
        arg1: i8,
        arg2: String,
    ) -> Result<i64, InvocationError> {
        let arg2 = serialize_to_vec(&arg2);
        let result = self.export_multiple_primitives_raw(arg1, arg2);
        result
    }
    pub fn export_multiple_primitives_raw(
        &self,
        arg1: i8,
        arg2: Vec<u8>,
    ) -> Result<i64, InvocationError> {
        let arg2 = export_to_guest_raw(&self.env, arg2);
        let function = self
            .instance
            .exports
            .get_native_function::<(<i8 as WasmAbi>::AbiType, FatPtr), <i64 as WasmAbi>::AbiType>(
                "__fp_gen_export_multiple_primitives",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_multiple_primitives".to_owned(),
                )
            })?;
        let result = function.call(arg1.to_abi(), arg2.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_bool(&self, arg: bool) -> Result<bool, InvocationError> {
        let result = self.export_primitive_bool_raw(arg);
        result
    }
    pub fn export_primitive_bool_raw(&self, arg: bool) -> Result<bool, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<bool as WasmAbi>::AbiType, <bool as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_bool",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_bool".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_f32(&self, arg: f32) -> Result<f32, InvocationError> {
        let result = self.export_primitive_f32_raw(arg);
        result
    }
    pub fn export_primitive_f32_raw(&self, arg: f32) -> Result<f32, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<f32 as WasmAbi>::AbiType, <f32 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_f32",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_f32".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_f64(&self, arg: f64) -> Result<f64, InvocationError> {
        let result = self.export_primitive_f64_raw(arg);
        result
    }
    pub fn export_primitive_f64_raw(&self, arg: f64) -> Result<f64, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<f64 as WasmAbi>::AbiType, <f64 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_f64",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_f64".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i16(&self, arg: i16) -> Result<i16, InvocationError> {
        let result = self.export_primitive_i16_raw(arg);
        result
    }
    pub fn export_primitive_i16_raw(&self, arg: i16) -> Result<i16, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<i16 as WasmAbi>::AbiType, <i16 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_i16",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i16".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i32(&self, arg: i32) -> Result<i32, InvocationError> {
        let result = self.export_primitive_i32_raw(arg);
        result
    }
    pub fn export_primitive_i32_raw(&self, arg: i32) -> Result<i32, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<i32 as WasmAbi>::AbiType, <i32 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_i32",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i32".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i64(&self, arg: i64) -> Result<i64, InvocationError> {
        let result = self.export_primitive_i64_raw(arg);
        result
    }
    pub fn export_primitive_i64_raw(&self, arg: i64) -> Result<i64, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<i64 as WasmAbi>::AbiType, <i64 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_i64",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i64".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i8(&self, arg: i8) -> Result<i8, InvocationError> {
        let result = self.export_primitive_i8_raw(arg);
        result
    }
    pub fn export_primitive_i8_raw(&self, arg: i8) -> Result<i8, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<i8 as WasmAbi>::AbiType, <i8 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_i8",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i8".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u16(&self, arg: u16) -> Result<u16, InvocationError> {
        let result = self.export_primitive_u16_raw(arg);
        result
    }
    pub fn export_primitive_u16_raw(&self, arg: u16) -> Result<u16, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<u16 as WasmAbi>::AbiType, <u16 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_u16",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u16".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u32(&self, arg: u32) -> Result<u32, InvocationError> {
        let result = self.export_primitive_u32_raw(arg);
        result
    }
    pub fn export_primitive_u32_raw(&self, arg: u32) -> Result<u32, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<u32 as WasmAbi>::AbiType, <u32 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_u32",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u32".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u64(&self, arg: u64) -> Result<u64, InvocationError> {
        let result = self.export_primitive_u64_raw(arg);
        result
    }
    pub fn export_primitive_u64_raw(&self, arg: u64) -> Result<u64, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<u64 as WasmAbi>::AbiType, <u64 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_u64",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u64".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u8(&self, arg: u8) -> Result<u8, InvocationError> {
        let result = self.export_primitive_u8_raw(arg);
        result
    }
    pub fn export_primitive_u8_raw(&self, arg: u8) -> Result<u8, InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<<u8 as WasmAbi>::AbiType, <u8 as WasmAbi>::AbiType>(
                "__fp_gen_export_primitive_u8",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u8".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_serde_adjacently_tagged(
        &self,
        arg: SerdeAdjacentlyTagged,
    ) -> Result<SerdeAdjacentlyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_adjacently_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_adjacently_tagged_raw(
        &self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_serde_adjacently_tagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_serde_adjacently_tagged".to_owned(),
                )
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_serde_enum(
        &self,
        arg: SerdeVariantRenaming,
    ) -> Result<SerdeVariantRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_enum_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_enum_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_serde_enum")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_enum".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_serde_flatten(&self, arg: SerdeFlatten) -> Result<SerdeFlatten, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_flatten_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_flatten_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_serde_flatten")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_flatten".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_serde_internally_tagged(
        &self,
        arg: SerdeInternallyTagged,
    ) -> Result<SerdeInternallyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_internally_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_internally_tagged_raw(
        &self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_serde_internally_tagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_serde_internally_tagged".to_owned(),
                )
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_serde_struct(
        &self,
        arg: SerdePropertyRenaming,
    ) -> Result<SerdePropertyRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_struct_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_struct_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_serde_struct")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_struct".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_serde_untagged(
        &self,
        arg: SerdeUntagged,
    ) -> Result<SerdeUntagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_untagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_untagged_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_serde_untagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_untagged".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_string(&self, arg: String) -> Result<String, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_string_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_string_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_string")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_string".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_struct_with_options(
        &self,
        arg: StructWithOptions,
    ) -> Result<StructWithOptions, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_struct_with_options_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_struct_with_options_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_struct_with_options")
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_struct_with_options".to_owned(),
                )
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_timestamp(&self, arg: MyDateTime) -> Result<MyDateTime, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_timestamp_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_timestamp_raw(&self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&self.env, arg);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_export_timestamp")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_timestamp".to_owned())
            })?;
        let result = function.call(arg.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }

    pub fn export_void_function(&self) -> Result<(), InvocationError> {
        let result = self.export_void_function_raw();
        result
    }
    pub fn export_void_function_raw(&self) -> Result<(), InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<(), ()>("__fp_gen_export_void_function")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_void_function".to_owned())
            })?;
        let result = function.call()?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    /// Example how plugin could expose async data-fetching capabilities.
    pub async fn fetch_data(
        &self,
        r#type: String,
    ) -> Result<Result<String, String>, InvocationError> {
        let r#type = serialize_to_vec(&r#type);
        let result = self.fetch_data_raw(r#type);
        let result = result.await;
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub async fn fetch_data_raw(&self, r#type: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let r#type = export_to_guest_raw(&self.env, r#type);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_fetch_data")
            .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_fetch_data".to_owned()))?;
        let result = function.call(r#type.to_abi())?;
        let result = ModuleRawFuture::new(self.env.clone(), result).await;
        Ok(result)
    }

    /// Called on the plugin to give it a chance to initialize.
    pub fn init(&self) -> Result<(), InvocationError> {
        let result = self.init_raw();
        result
    }
    pub fn init_raw(&self) -> Result<(), InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<(), ()>("__fp_gen_init")
            .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_init".to_owned()))?;
        let result = function.call()?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    /// Example how plugin could expose a reducer.
    pub fn reducer_bridge(&self, action: ReduxAction) -> Result<StateUpdate, InvocationError> {
        let action = serialize_to_vec(&action);
        let result = self.reducer_bridge_raw(action);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn reducer_bridge_raw(&self, action: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let action = export_to_guest_raw(&self.env, action);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_reducer_bridge")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_reducer_bridge".to_owned())
            })?;
        let result = function.call(action.to_abi())?;
        let result = import_from_guest_raw(&self.env, result);
        Ok(result)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
        "fp" => {
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
            "__fp_gen_import_array_f32" => Function::new_native_with_env(store, env.clone(), _import_array_f32),
            "__fp_gen_import_array_f64" => Function::new_native_with_env(store, env.clone(), _import_array_f64),
            "__fp_gen_import_array_i16" => Function::new_native_with_env(store, env.clone(), _import_array_i16),
            "__fp_gen_import_array_i32" => Function::new_native_with_env(store, env.clone(), _import_array_i32),
            "__fp_gen_import_array_i8" => Function::new_native_with_env(store, env.clone(), _import_array_i8),
            "__fp_gen_import_array_u16" => Function::new_native_with_env(store, env.clone(), _import_array_u16),
            "__fp_gen_import_array_u32" => Function::new_native_with_env(store, env.clone(), _import_array_u32),
            "__fp_gen_import_array_u8" => Function::new_native_with_env(store, env.clone(), _import_array_u8),
            "__fp_gen_import_explicit_bound_point" => Function::new_native_with_env(store, env.clone(), _import_explicit_bound_point),
            "__fp_gen_import_fp_adjacently_tagged" => Function::new_native_with_env(store, env.clone(), _import_fp_adjacently_tagged),
            "__fp_gen_import_fp_enum" => Function::new_native_with_env(store, env.clone(), _import_fp_enum),
            "__fp_gen_import_fp_flatten" => Function::new_native_with_env(store, env.clone(), _import_fp_flatten),
            "__fp_gen_import_fp_internally_tagged" => Function::new_native_with_env(store, env.clone(), _import_fp_internally_tagged),
            "__fp_gen_import_fp_struct" => Function::new_native_with_env(store, env.clone(), _import_fp_struct),
            "__fp_gen_import_fp_untagged" => Function::new_native_with_env(store, env.clone(), _import_fp_untagged),
            "__fp_gen_import_generics" => Function::new_native_with_env(store, env.clone(), _import_generics),
            "__fp_gen_import_get_bytes" => Function::new_native_with_env(store, env.clone(), _import_get_bytes),
            "__fp_gen_import_get_serde_bytes" => Function::new_native_with_env(store, env.clone(), _import_get_serde_bytes),
            "__fp_gen_import_multiple_primitives" => Function::new_native_with_env(store, env.clone(), _import_multiple_primitives),
            "__fp_gen_import_primitive_bool" => Function::new_native_with_env(store, env.clone(), _import_primitive_bool),
            "__fp_gen_import_primitive_f32" => Function::new_native_with_env(store, env.clone(), _import_primitive_f32),
            "__fp_gen_import_primitive_f64" => Function::new_native_with_env(store, env.clone(), _import_primitive_f64),
            "__fp_gen_import_primitive_i16" => Function::new_native_with_env(store, env.clone(), _import_primitive_i16),
            "__fp_gen_import_primitive_i32" => Function::new_native_with_env(store, env.clone(), _import_primitive_i32),
            "__fp_gen_import_primitive_i64" => Function::new_native_with_env(store, env.clone(), _import_primitive_i64),
            "__fp_gen_import_primitive_i8" => Function::new_native_with_env(store, env.clone(), _import_primitive_i8),
            "__fp_gen_import_primitive_u16" => Function::new_native_with_env(store, env.clone(), _import_primitive_u16),
            "__fp_gen_import_primitive_u32" => Function::new_native_with_env(store, env.clone(), _import_primitive_u32),
            "__fp_gen_import_primitive_u64" => Function::new_native_with_env(store, env.clone(), _import_primitive_u64),
            "__fp_gen_import_primitive_u8" => Function::new_native_with_env(store, env.clone(), _import_primitive_u8),
            "__fp_gen_import_serde_adjacently_tagged" => Function::new_native_with_env(store, env.clone(), _import_serde_adjacently_tagged),
            "__fp_gen_import_serde_enum" => Function::new_native_with_env(store, env.clone(), _import_serde_enum),
            "__fp_gen_import_serde_flatten" => Function::new_native_with_env(store, env.clone(), _import_serde_flatten),
            "__fp_gen_import_serde_internally_tagged" => Function::new_native_with_env(store, env.clone(), _import_serde_internally_tagged),
            "__fp_gen_import_serde_struct" => Function::new_native_with_env(store, env.clone(), _import_serde_struct),
            "__fp_gen_import_serde_untagged" => Function::new_native_with_env(store, env.clone(), _import_serde_untagged),
            "__fp_gen_import_string" => Function::new_native_with_env(store, env.clone(), _import_string),
            "__fp_gen_import_struct_with_options" => Function::new_native_with_env(store, env.clone(), _import_struct_with_options),
            "__fp_gen_import_timestamp" => Function::new_native_with_env(store, env.clone(), _import_timestamp),
            "__fp_gen_import_void_function" => Function::new_native_with_env(store, env.clone(), _import_void_function),
            "__fp_gen_import_void_function_empty_result" => Function::new_native_with_env(store, env.clone(), _import_void_function_empty_result),
            "__fp_gen_import_void_function_empty_return" => Function::new_native_with_env(store, env.clone(), _import_void_function_empty_return),
            "__fp_gen_log" => Function::new_native_with_env(store, env.clone(), _log),
            "__fp_gen_make_http_request" => Function::new_native_with_env(store, env.clone(), _make_http_request),
        }
    }
}

pub fn _import_array_f32(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[f32; 3]>(env, arg);
    let result = super::import_array_f32(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_f64(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[f64; 3]>(env, arg);
    let result = super::import_array_f64(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_i16(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[i16; 3]>(env, arg);
    let result = super::import_array_i16(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_i32(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[i32; 3]>(env, arg);
    let result = super::import_array_i32(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_i8(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[i8; 3]>(env, arg);
    let result = super::import_array_i8(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_u16(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[u16; 3]>(env, arg);
    let result = super::import_array_u16(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_u32(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[u32; 3]>(env, arg);
    let result = super::import_array_u32(arg);
    export_to_guest(env, &result)
}

pub fn _import_array_u8(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[u8; 3]>(env, arg);
    let result = super::import_array_u8(arg);
    export_to_guest(env, &result)
}

pub fn _import_explicit_bound_point(env: &RuntimeInstanceData, arg: FatPtr) {
    let arg = import_from_guest::<ExplicitBoundPoint<u64>>(env, arg);
    let result = super::import_explicit_bound_point(arg);
}

pub fn _import_fp_adjacently_tagged(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpAdjacentlyTagged>(env, arg);
    let result = super::import_fp_adjacently_tagged(arg);
    export_to_guest(env, &result)
}

pub fn _import_fp_enum(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpVariantRenaming>(env, arg);
    let result = super::import_fp_enum(arg);
    export_to_guest(env, &result)
}

pub fn _import_fp_flatten(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpFlatten>(env, arg);
    let result = super::import_fp_flatten(arg);
    export_to_guest(env, &result)
}

pub fn _import_fp_internally_tagged(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpInternallyTagged>(env, arg);
    let result = super::import_fp_internally_tagged(arg);
    export_to_guest(env, &result)
}

pub fn _import_fp_struct(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpPropertyRenaming>(env, arg);
    let result = super::import_fp_struct(arg);
    export_to_guest(env, &result)
}

pub fn _import_fp_untagged(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpUntagged>(env, arg);
    let result = super::import_fp_untagged(arg);
    export_to_guest(env, &result)
}

pub fn _import_generics(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<StructWithGenerics<u64>>(env, arg);
    let result = super::import_generics(arg);
    export_to_guest(env, &result)
}

pub fn _import_get_bytes(env: &RuntimeInstanceData) -> FatPtr {
    let result = super::import_get_bytes();
    export_to_guest(env, &result)
}

pub fn _import_get_serde_bytes(env: &RuntimeInstanceData) -> FatPtr {
    let result = super::import_get_serde_bytes();
    export_to_guest(env, &result)
}

pub fn _import_multiple_primitives(
    env: &RuntimeInstanceData,
    arg1: <i8 as WasmAbi>::AbiType,
    arg2: FatPtr,
) -> <i64 as WasmAbi>::AbiType {
    let arg1 = WasmAbi::from_abi(arg1);
    let arg2 = import_from_guest::<String>(env, arg2);
    let result = super::import_multiple_primitives(arg1, arg2);
    result.to_abi()
}

pub fn _import_primitive_bool(
    env: &RuntimeInstanceData,
    arg: <bool as WasmAbi>::AbiType,
) -> <bool as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_bool(arg);
    result.to_abi()
}

pub fn _import_primitive_f32(
    env: &RuntimeInstanceData,
    arg: <f32 as WasmAbi>::AbiType,
) -> <f32 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_f32(arg);
    result.to_abi()
}

pub fn _import_primitive_f64(
    env: &RuntimeInstanceData,
    arg: <f64 as WasmAbi>::AbiType,
) -> <f64 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_f64(arg);
    result.to_abi()
}

pub fn _import_primitive_i16(
    env: &RuntimeInstanceData,
    arg: <i16 as WasmAbi>::AbiType,
) -> <i16 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i16(arg);
    result.to_abi()
}

pub fn _import_primitive_i32(
    env: &RuntimeInstanceData,
    arg: <i32 as WasmAbi>::AbiType,
) -> <i32 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i32(arg);
    result.to_abi()
}

pub fn _import_primitive_i64(
    env: &RuntimeInstanceData,
    arg: <i64 as WasmAbi>::AbiType,
) -> <i64 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i64(arg);
    result.to_abi()
}

pub fn _import_primitive_i8(
    env: &RuntimeInstanceData,
    arg: <i8 as WasmAbi>::AbiType,
) -> <i8 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i8(arg);
    result.to_abi()
}

pub fn _import_primitive_u16(
    env: &RuntimeInstanceData,
    arg: <u16 as WasmAbi>::AbiType,
) -> <u16 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u16(arg);
    result.to_abi()
}

pub fn _import_primitive_u32(
    env: &RuntimeInstanceData,
    arg: <u32 as WasmAbi>::AbiType,
) -> <u32 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u32(arg);
    result.to_abi()
}

pub fn _import_primitive_u64(
    env: &RuntimeInstanceData,
    arg: <u64 as WasmAbi>::AbiType,
) -> <u64 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u64(arg);
    result.to_abi()
}

pub fn _import_primitive_u8(
    env: &RuntimeInstanceData,
    arg: <u8 as WasmAbi>::AbiType,
) -> <u8 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u8(arg);
    result.to_abi()
}

pub fn _import_serde_adjacently_tagged(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<SerdeAdjacentlyTagged>(env, arg);
    let result = super::import_serde_adjacently_tagged(arg);
    export_to_guest(env, &result)
}

pub fn _import_serde_enum(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<SerdeVariantRenaming>(env, arg);
    let result = super::import_serde_enum(arg);
    export_to_guest(env, &result)
}

pub fn _import_serde_flatten(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<SerdeFlatten>(env, arg);
    let result = super::import_serde_flatten(arg);
    export_to_guest(env, &result)
}

pub fn _import_serde_internally_tagged(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<SerdeInternallyTagged>(env, arg);
    let result = super::import_serde_internally_tagged(arg);
    export_to_guest(env, &result)
}

pub fn _import_serde_struct(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<SerdePropertyRenaming>(env, arg);
    let result = super::import_serde_struct(arg);
    export_to_guest(env, &result)
}

pub fn _import_serde_untagged(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<SerdeUntagged>(env, arg);
    let result = super::import_serde_untagged(arg);
    export_to_guest(env, &result)
}

pub fn _import_string(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<String>(env, arg);
    let result = super::import_string(arg);
    export_to_guest(env, &result)
}

pub fn _import_struct_with_options(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<StructWithOptions>(env, arg);
    let result = super::import_struct_with_options(arg);
    export_to_guest(env, &result)
}

pub fn _import_timestamp(env: &RuntimeInstanceData, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<MyDateTime>(env, arg);
    let result = super::import_timestamp(arg);
    export_to_guest(env, &result)
}

pub fn _import_void_function(env: &RuntimeInstanceData) {
    let result = super::import_void_function();
}

pub fn _import_void_function_empty_result(env: &RuntimeInstanceData) -> FatPtr {
    let result = super::import_void_function_empty_result();
    export_to_guest(env, &result)
}

pub fn _import_void_function_empty_return(env: &RuntimeInstanceData) {
    let result = super::import_void_function_empty_return();
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);
    let result = super::log(message);
}

pub fn _make_http_request(env: &RuntimeInstanceData, request: FatPtr) -> FatPtr {
    let request = import_from_guest::<Request>(env, request);
    let result = super::make_http_request(request);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = result.await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    });
    async_ptr
}
