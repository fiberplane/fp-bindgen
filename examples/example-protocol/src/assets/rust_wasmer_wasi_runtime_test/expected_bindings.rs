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
use std::sync::Arc;
use wasmer::{
    imports, AsStoreMut, Function, FunctionEnv, FunctionEnvMut, Imports, Instance, Module, Store,
};

pub struct Runtime {
    store: Store,
    instance: Instance,
    env: FunctionEnv<Arc<RuntimeInstanceData>>,
}

impl Runtime {
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let mut store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        let env = FunctionEnv::new(&mut store, Arc::new(RuntimeInstanceData::default()));
        let mut wasi_env = wasmer_wasi::WasiState::new("fp")
            .finalize(&mut store)
            .unwrap();
        let mut import_object = wasi_env.import_object(&mut store, &module).unwrap();
        import_object.register_namespace("fp", create_imports(&mut store, &env));
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();
        wasi_env.initialize(&mut store, &instance).unwrap();
        let env_from_instance = RuntimeInstanceData::from_instance(&mut store, &instance);
        Arc::get_mut(env.as_mut(&mut store))
            .unwrap()
            .copy_from(env_from_instance);
        Ok(Self {
            store,
            instance,
            env,
        })
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn default_store() -> wasmer::Store {
        Store::new(wasmer_compiler_cranelift::Cranelift::default())
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn default_store() -> wasmer::Store {
        Store::new(wasmer_compiler_singlepass::Singlepass::default())
    }

    fn function_env_mut(&mut self) -> FunctionEnvMut<Arc<RuntimeInstanceData>> {
        self.env.clone().into_mut(&mut self.store)
    }

    pub fn export_array_f32(&mut self, arg: [f32; 3]) -> Result<[f32; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_f32_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_f32_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_f32")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_f32".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_f64(&mut self, arg: [f64; 3]) -> Result<[f64; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_f64_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_f64_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_f64")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_f64".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_i16(&mut self, arg: [i16; 3]) -> Result<[i16; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_i16_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_i16_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_i16")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_i16".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_i32(&mut self, arg: [i32; 3]) -> Result<[i32; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_i32_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_i32_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_i32")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_i32".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_i8(&mut self, arg: [i8; 3]) -> Result<[i8; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_i8_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_i8_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_i8")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_i8".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_u16(&mut self, arg: [u16; 3]) -> Result<[u16; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_u16_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_u16_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_u16")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_u16".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_u32(&mut self, arg: [u32; 3]) -> Result<[u32; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_u32_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_u32_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_u32")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_u32".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_array_u8(&mut self, arg: [u8; 3]) -> Result<[u8; 3], InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_array_u8_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_array_u8_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_array_u8")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_array_u8".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub async fn export_async_struct(
        &mut self,
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
        &mut self,
        arg1: Vec<u8>,
        arg2: u64,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg1 = export_to_guest_raw(&mut self.function_env_mut(), arg1);
        let function = self
            .instance
            .exports
            .get_typed_function::<(FatPtr, <u64 as WasmAbi>::AbiType), FatPtr>(
                &mut self.store,
                "__fp_gen_export_async_struct",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_async_struct".to_owned())
            })?;
        let result = function.call(&mut self.store, arg1.to_abi(), arg2.to_abi())?;
        let result = ModuleRawFuture::new(self.function_env_mut(), result).await;
        Ok(result)
    }

    pub fn export_fp_adjacently_tagged(
        &mut self,
        arg: FpAdjacentlyTagged,
    ) -> Result<FpAdjacentlyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_adjacently_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_adjacently_tagged_raw(
        &mut self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(
                &mut self.store,
                "__fp_gen_export_fp_adjacently_tagged",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_fp_adjacently_tagged".to_owned(),
                )
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_fp_enum(
        &mut self,
        arg: FpVariantRenaming,
    ) -> Result<FpVariantRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_enum_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_enum_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_fp_enum")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_enum".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_fp_flatten(&mut self, arg: FpFlatten) -> Result<FpFlatten, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_flatten_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_flatten_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_fp_flatten")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_flatten".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_fp_internally_tagged(
        &mut self,
        arg: FpInternallyTagged,
    ) -> Result<FpInternallyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_internally_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_internally_tagged_raw(
        &mut self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(
                &mut self.store,
                "__fp_gen_export_fp_internally_tagged",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_fp_internally_tagged".to_owned(),
                )
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_fp_struct(
        &mut self,
        arg: FpPropertyRenaming,
    ) -> Result<FpPropertyRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_struct_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_struct_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_fp_struct")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_struct".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_fp_untagged(&mut self, arg: FpUntagged) -> Result<FpUntagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_fp_untagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_fp_untagged_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_fp_untagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_fp_untagged".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_generics(
        &mut self,
        arg: StructWithGenerics<u64>,
    ) -> Result<StructWithGenerics<u64>, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_generics_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_generics_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_generics")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_generics".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_get_bytes(&mut self) -> Result<Result<bytes::Bytes, String>, InvocationError> {
        let result = self.export_get_bytes_raw();
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_get_bytes_raw(&mut self) -> Result<Vec<u8>, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<(), FatPtr>(&mut self.store, "__fp_gen_export_get_bytes")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_get_bytes".to_owned())
            })?;
        let result = function.call(&mut self.store)?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_get_serde_bytes(
        &mut self,
    ) -> Result<Result<serde_bytes::ByteBuf, String>, InvocationError> {
        let result = self.export_get_serde_bytes_raw();
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_get_serde_bytes_raw(&mut self) -> Result<Vec<u8>, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<(), FatPtr>(&mut self.store, "__fp_gen_export_get_serde_bytes")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_get_serde_bytes".to_owned())
            })?;
        let result = function.call(&mut self.store)?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_multiple_primitives(
        &mut self,
        arg1: i8,
        arg2: String,
    ) -> Result<i64, InvocationError> {
        let arg2 = serialize_to_vec(&arg2);
        let result = self.export_multiple_primitives_raw(arg1, arg2);
        result
    }
    pub fn export_multiple_primitives_raw(
        &mut self,
        arg1: i8,
        arg2: Vec<u8>,
    ) -> Result<i64, InvocationError> {
        let arg2 = export_to_guest_raw(&mut self.function_env_mut(), arg2);
        let function = self
            .instance
            .exports
            .get_typed_function::<(<i8 as WasmAbi>::AbiType, FatPtr), <i64 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_multiple_primitives",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_multiple_primitives".to_owned(),
                )
            })?;
        let result = function.call(&mut self.store, arg1.to_abi(), arg2.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_bool(&mut self, arg: bool) -> Result<bool, InvocationError> {
        let result = self.export_primitive_bool_raw(arg);
        result
    }
    pub fn export_primitive_bool_raw(&mut self, arg: bool) -> Result<bool, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<bool as WasmAbi>::AbiType, <bool as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_bool",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_bool".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_f32(&mut self, arg: f32) -> Result<f32, InvocationError> {
        let result = self.export_primitive_f32_raw(arg);
        result
    }
    pub fn export_primitive_f32_raw(&mut self, arg: f32) -> Result<f32, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<f32 as WasmAbi>::AbiType, <f32 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_f32",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_f32".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_f64(&mut self, arg: f64) -> Result<f64, InvocationError> {
        let result = self.export_primitive_f64_raw(arg);
        result
    }
    pub fn export_primitive_f64_raw(&mut self, arg: f64) -> Result<f64, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<f64 as WasmAbi>::AbiType, <f64 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_f64",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_f64".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i16(&mut self, arg: i16) -> Result<i16, InvocationError> {
        let result = self.export_primitive_i16_raw(arg);
        result
    }
    pub fn export_primitive_i16_raw(&mut self, arg: i16) -> Result<i16, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<i16 as WasmAbi>::AbiType, <i16 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_i16",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i16".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i32(&mut self, arg: i32) -> Result<i32, InvocationError> {
        let result = self.export_primitive_i32_raw(arg);
        result
    }
    pub fn export_primitive_i32_raw(&mut self, arg: i32) -> Result<i32, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<i32 as WasmAbi>::AbiType, <i32 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_i32",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i32".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i64(&mut self, arg: i64) -> Result<i64, InvocationError> {
        let result = self.export_primitive_i64_raw(arg);
        result
    }
    pub fn export_primitive_i64_raw(&mut self, arg: i64) -> Result<i64, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<i64 as WasmAbi>::AbiType, <i64 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_i64",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i64".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_i8(&mut self, arg: i8) -> Result<i8, InvocationError> {
        let result = self.export_primitive_i8_raw(arg);
        result
    }
    pub fn export_primitive_i8_raw(&mut self, arg: i8) -> Result<i8, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<i8 as WasmAbi>::AbiType, <i8 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_i8",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_i8".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u16(&mut self, arg: u16) -> Result<u16, InvocationError> {
        let result = self.export_primitive_u16_raw(arg);
        result
    }
    pub fn export_primitive_u16_raw(&mut self, arg: u16) -> Result<u16, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<u16 as WasmAbi>::AbiType, <u16 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_u16",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u16".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u32(&mut self, arg: u32) -> Result<u32, InvocationError> {
        let result = self.export_primitive_u32_raw(arg);
        result
    }
    pub fn export_primitive_u32_raw(&mut self, arg: u32) -> Result<u32, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<u32 as WasmAbi>::AbiType, <u32 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_u32",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u32".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u64(&mut self, arg: u64) -> Result<u64, InvocationError> {
        let result = self.export_primitive_u64_raw(arg);
        result
    }
    pub fn export_primitive_u64_raw(&mut self, arg: u64) -> Result<u64, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<u64 as WasmAbi>::AbiType, <u64 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_u64",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u64".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_primitive_u8(&mut self, arg: u8) -> Result<u8, InvocationError> {
        let result = self.export_primitive_u8_raw(arg);
        result
    }
    pub fn export_primitive_u8_raw(&mut self, arg: u8) -> Result<u8, InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<<u8 as WasmAbi>::AbiType, <u8 as WasmAbi>::AbiType>(
                &mut self.store,
                "__fp_gen_export_primitive_u8",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_primitive_u8".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    pub fn export_serde_adjacently_tagged(
        &mut self,
        arg: SerdeAdjacentlyTagged,
    ) -> Result<SerdeAdjacentlyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_adjacently_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_adjacently_tagged_raw(
        &mut self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(
                &mut self.store,
                "__fp_gen_export_serde_adjacently_tagged",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_serde_adjacently_tagged".to_owned(),
                )
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_serde_enum(
        &mut self,
        arg: SerdeVariantRenaming,
    ) -> Result<SerdeVariantRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_enum_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_enum_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_serde_enum")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_enum".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_serde_flatten(
        &mut self,
        arg: SerdeFlatten,
    ) -> Result<SerdeFlatten, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_flatten_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_flatten_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_serde_flatten")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_flatten".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_serde_internally_tagged(
        &mut self,
        arg: SerdeInternallyTagged,
    ) -> Result<SerdeInternallyTagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_internally_tagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_internally_tagged_raw(
        &mut self,
        arg: Vec<u8>,
    ) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(
                &mut self.store,
                "__fp_gen_export_serde_internally_tagged",
            )
            .map_err(|_| {
                InvocationError::FunctionNotExported(
                    "__fp_gen_export_serde_internally_tagged".to_owned(),
                )
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_serde_struct(
        &mut self,
        arg: SerdePropertyRenaming,
    ) -> Result<SerdePropertyRenaming, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_struct_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_struct_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_serde_struct")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_struct".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_serde_untagged(
        &mut self,
        arg: SerdeUntagged,
    ) -> Result<SerdeUntagged, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_serde_untagged_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_serde_untagged_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_serde_untagged")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_serde_untagged".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_string(&mut self, arg: String) -> Result<String, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_string_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_string_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_string")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_string".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_timestamp(&mut self, arg: MyDateTime) -> Result<MyDateTime, InvocationError> {
        let arg = serialize_to_vec(&arg);
        let result = self.export_timestamp_raw(arg);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn export_timestamp_raw(&mut self, arg: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let arg = export_to_guest_raw(&mut self.function_env_mut(), arg);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_export_timestamp")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_timestamp".to_owned())
            })?;
        let result = function.call(&mut self.store, arg.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }

    pub fn export_void_function(&mut self) -> Result<(), InvocationError> {
        let result = self.export_void_function_raw();
        result
    }
    pub fn export_void_function_raw(&mut self) -> Result<(), InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<(), ()>(&mut self.store, "__fp_gen_export_void_function")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_export_void_function".to_owned())
            })?;
        let result = function.call(&mut self.store)?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    /// Example how plugin could expose async data-fetching capabilities.
    pub async fn fetch_data(
        &mut self,
        r#type: String,
    ) -> Result<Result<String, String>, InvocationError> {
        let r#type = serialize_to_vec(&r#type);
        let result = self.fetch_data_raw(r#type);
        let result = result.await;
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub async fn fetch_data_raw(&mut self, r#type: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let r#type = export_to_guest_raw(&mut self.function_env_mut(), r#type);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_fetch_data")
            .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_fetch_data".to_owned()))?;
        let result = function.call(&mut self.store, r#type.to_abi())?;
        let result = ModuleRawFuture::new(self.function_env_mut(), result).await;
        Ok(result)
    }

    /// Called on the plugin to give it a chance to initialize.
    pub fn init(&mut self) -> Result<(), InvocationError> {
        let result = self.init_raw();
        result
    }
    pub fn init_raw(&mut self) -> Result<(), InvocationError> {
        let function = self
            .instance
            .exports
            .get_typed_function::<(), ()>(&mut self.store, "__fp_gen_init")
            .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_init".to_owned()))?;
        let result = function.call(&mut self.store)?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }

    /// Example how plugin could expose a reducer.
    pub fn reducer_bridge(&mut self, action: ReduxAction) -> Result<StateUpdate, InvocationError> {
        let action = serialize_to_vec(&action);
        let result = self.reducer_bridge_raw(action);
        let result = result.map(|ref data| deserialize_from_slice(data));
        result
    }
    pub fn reducer_bridge_raw(&mut self, action: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let action = export_to_guest_raw(&mut self.function_env_mut(), action);
        let function = self
            .instance
            .exports
            .get_typed_function::<FatPtr, FatPtr>(&mut self.store, "__fp_gen_reducer_bridge")
            .map_err(|_| {
                InvocationError::FunctionNotExported("__fp_gen_reducer_bridge".to_owned())
            })?;
        let result = function.call(&mut self.store, action.to_abi())?;
        let result = import_from_guest_raw(&mut self.function_env_mut(), result);
        Ok(result)
    }
}

fn create_imports(
    store: &mut Store,
    env: &FunctionEnv<Arc<RuntimeInstanceData>>,
) -> wasmer::Exports {
    let mut namespace = wasmer::Exports::new();
    namespace.insert(
        "__fp_host_resolve_async_value",
        Function::new_typed_with_env(store, env, resolve_async_value),
    );
    namespace.insert(
        "__fp_gen_import_array_f32",
        Function::new_typed_with_env(store, env, _import_array_f32),
    );
    namespace.insert(
        "__fp_gen_import_array_f64",
        Function::new_typed_with_env(store, env, _import_array_f64),
    );
    namespace.insert(
        "__fp_gen_import_array_i16",
        Function::new_typed_with_env(store, env, _import_array_i16),
    );
    namespace.insert(
        "__fp_gen_import_array_i32",
        Function::new_typed_with_env(store, env, _import_array_i32),
    );
    namespace.insert(
        "__fp_gen_import_array_i8",
        Function::new_typed_with_env(store, env, _import_array_i8),
    );
    namespace.insert(
        "__fp_gen_import_array_u16",
        Function::new_typed_with_env(store, env, _import_array_u16),
    );
    namespace.insert(
        "__fp_gen_import_array_u32",
        Function::new_typed_with_env(store, env, _import_array_u32),
    );
    namespace.insert(
        "__fp_gen_import_array_u8",
        Function::new_typed_with_env(store, env, _import_array_u8),
    );
    namespace.insert(
        "__fp_gen_import_explicit_bound_point",
        Function::new_typed_with_env(store, env, _import_explicit_bound_point),
    );
    namespace.insert(
        "__fp_gen_import_fp_adjacently_tagged",
        Function::new_typed_with_env(store, env, _import_fp_adjacently_tagged),
    );
    namespace.insert(
        "__fp_gen_import_fp_enum",
        Function::new_typed_with_env(store, env, _import_fp_enum),
    );
    namespace.insert(
        "__fp_gen_import_fp_flatten",
        Function::new_typed_with_env(store, env, _import_fp_flatten),
    );
    namespace.insert(
        "__fp_gen_import_fp_internally_tagged",
        Function::new_typed_with_env(store, env, _import_fp_internally_tagged),
    );
    namespace.insert(
        "__fp_gen_import_fp_struct",
        Function::new_typed_with_env(store, env, _import_fp_struct),
    );
    namespace.insert(
        "__fp_gen_import_fp_untagged",
        Function::new_typed_with_env(store, env, _import_fp_untagged),
    );
    namespace.insert(
        "__fp_gen_import_generics",
        Function::new_typed_with_env(store, env, _import_generics),
    );
    namespace.insert(
        "__fp_gen_import_get_bytes",
        Function::new_typed_with_env(store, env, _import_get_bytes),
    );
    namespace.insert(
        "__fp_gen_import_get_serde_bytes",
        Function::new_typed_with_env(store, env, _import_get_serde_bytes),
    );
    namespace.insert(
        "__fp_gen_import_multiple_primitives",
        Function::new_typed_with_env(store, env, _import_multiple_primitives),
    );
    namespace.insert(
        "__fp_gen_import_primitive_bool",
        Function::new_typed_with_env(store, env, _import_primitive_bool),
    );
    namespace.insert(
        "__fp_gen_import_primitive_f32",
        Function::new_typed_with_env(store, env, _import_primitive_f32),
    );
    namespace.insert(
        "__fp_gen_import_primitive_f64",
        Function::new_typed_with_env(store, env, _import_primitive_f64),
    );
    namespace.insert(
        "__fp_gen_import_primitive_i16",
        Function::new_typed_with_env(store, env, _import_primitive_i16),
    );
    namespace.insert(
        "__fp_gen_import_primitive_i32",
        Function::new_typed_with_env(store, env, _import_primitive_i32),
    );
    namespace.insert(
        "__fp_gen_import_primitive_i64",
        Function::new_typed_with_env(store, env, _import_primitive_i64),
    );
    namespace.insert(
        "__fp_gen_import_primitive_i8",
        Function::new_typed_with_env(store, env, _import_primitive_i8),
    );
    namespace.insert(
        "__fp_gen_import_primitive_u16",
        Function::new_typed_with_env(store, env, _import_primitive_u16),
    );
    namespace.insert(
        "__fp_gen_import_primitive_u32",
        Function::new_typed_with_env(store, env, _import_primitive_u32),
    );
    namespace.insert(
        "__fp_gen_import_primitive_u64",
        Function::new_typed_with_env(store, env, _import_primitive_u64),
    );
    namespace.insert(
        "__fp_gen_import_primitive_u8",
        Function::new_typed_with_env(store, env, _import_primitive_u8),
    );
    namespace.insert(
        "__fp_gen_import_serde_adjacently_tagged",
        Function::new_typed_with_env(store, env, _import_serde_adjacently_tagged),
    );
    namespace.insert(
        "__fp_gen_import_serde_enum",
        Function::new_typed_with_env(store, env, _import_serde_enum),
    );
    namespace.insert(
        "__fp_gen_import_serde_flatten",
        Function::new_typed_with_env(store, env, _import_serde_flatten),
    );
    namespace.insert(
        "__fp_gen_import_serde_internally_tagged",
        Function::new_typed_with_env(store, env, _import_serde_internally_tagged),
    );
    namespace.insert(
        "__fp_gen_import_serde_struct",
        Function::new_typed_with_env(store, env, _import_serde_struct),
    );
    namespace.insert(
        "__fp_gen_import_serde_untagged",
        Function::new_typed_with_env(store, env, _import_serde_untagged),
    );
    namespace.insert(
        "__fp_gen_import_string",
        Function::new_typed_with_env(store, env, _import_string),
    );
    namespace.insert(
        "__fp_gen_import_timestamp",
        Function::new_typed_with_env(store, env, _import_timestamp),
    );
    namespace.insert(
        "__fp_gen_import_void_function",
        Function::new_typed_with_env(store, env, _import_void_function),
    );
    namespace.insert(
        "__fp_gen_import_void_function_empty_result",
        Function::new_typed_with_env(store, env, _import_void_function_empty_result),
    );
    namespace.insert(
        "__fp_gen_import_void_function_empty_return",
        Function::new_typed_with_env(store, env, _import_void_function_empty_return),
    );
    namespace.insert(
        "__fp_gen_log",
        Function::new_typed_with_env(store, env, _log),
    );
    namespace.insert(
        "__fp_gen_make_http_request",
        Function::new_typed_with_env(store, env, _make_http_request),
    );
    namespace
}

pub fn _import_array_f32(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[f32; 3]>(&mut env, arg);
    let result = super::import_array_f32(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_f64(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[f64; 3]>(&mut env, arg);
    let result = super::import_array_f64(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_i16(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[i16; 3]>(&mut env, arg);
    let result = super::import_array_i16(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_i32(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[i32; 3]>(&mut env, arg);
    let result = super::import_array_i32(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_i8(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[i8; 3]>(&mut env, arg);
    let result = super::import_array_i8(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_u16(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[u16; 3]>(&mut env, arg);
    let result = super::import_array_u16(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_u32(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[u32; 3]>(&mut env, arg);
    let result = super::import_array_u32(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_array_u8(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<[u8; 3]>(&mut env, arg);
    let result = super::import_array_u8(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_explicit_bound_point(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) {
    let arg = import_from_guest::<ExplicitBoundPoint<u64>>(&mut env, arg);
    let result = super::import_explicit_bound_point(arg);
}

pub fn _import_fp_adjacently_tagged(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<FpAdjacentlyTagged>(&mut env, arg);
    let result = super::import_fp_adjacently_tagged(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_fp_enum(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpVariantRenaming>(&mut env, arg);
    let result = super::import_fp_enum(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_fp_flatten(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<FpFlatten>(&mut env, arg);
    let result = super::import_fp_flatten(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_fp_internally_tagged(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<FpInternallyTagged>(&mut env, arg);
    let result = super::import_fp_internally_tagged(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_fp_struct(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<FpPropertyRenaming>(&mut env, arg);
    let result = super::import_fp_struct(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_fp_untagged(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<FpUntagged>(&mut env, arg);
    let result = super::import_fp_untagged(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_generics(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<StructWithGenerics<u64>>(&mut env, arg);
    let result = super::import_generics(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_get_bytes(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>) -> FatPtr {
    let result = super::import_get_bytes();
    export_to_guest(&mut env, &result)
}

pub fn _import_get_serde_bytes(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>) -> FatPtr {
    let result = super::import_get_serde_bytes();
    export_to_guest(&mut env, &result)
}

pub fn _import_multiple_primitives(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg1: <i8 as WasmAbi>::AbiType,
    arg2: FatPtr,
) -> <i64 as WasmAbi>::AbiType {
    let arg1 = WasmAbi::from_abi(arg1);
    let arg2 = import_from_guest::<String>(&mut env, arg2);
    let result = super::import_multiple_primitives(arg1, arg2);
    result.to_abi()
}

pub fn _import_primitive_bool(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <bool as WasmAbi>::AbiType,
) -> <bool as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_bool(arg);
    result.to_abi()
}

pub fn _import_primitive_f32(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <f32 as WasmAbi>::AbiType,
) -> <f32 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_f32(arg);
    result.to_abi()
}

pub fn _import_primitive_f64(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <f64 as WasmAbi>::AbiType,
) -> <f64 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_f64(arg);
    result.to_abi()
}

pub fn _import_primitive_i16(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <i16 as WasmAbi>::AbiType,
) -> <i16 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i16(arg);
    result.to_abi()
}

pub fn _import_primitive_i32(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <i32 as WasmAbi>::AbiType,
) -> <i32 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i32(arg);
    result.to_abi()
}

pub fn _import_primitive_i64(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <i64 as WasmAbi>::AbiType,
) -> <i64 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i64(arg);
    result.to_abi()
}

pub fn _import_primitive_i8(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <i8 as WasmAbi>::AbiType,
) -> <i8 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_i8(arg);
    result.to_abi()
}

pub fn _import_primitive_u16(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <u16 as WasmAbi>::AbiType,
) -> <u16 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u16(arg);
    result.to_abi()
}

pub fn _import_primitive_u32(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <u32 as WasmAbi>::AbiType,
) -> <u32 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u32(arg);
    result.to_abi()
}

pub fn _import_primitive_u64(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <u64 as WasmAbi>::AbiType,
) -> <u64 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u64(arg);
    result.to_abi()
}

pub fn _import_primitive_u8(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: <u8 as WasmAbi>::AbiType,
) -> <u8 as WasmAbi>::AbiType {
    let arg = WasmAbi::from_abi(arg);
    let result = super::import_primitive_u8(arg);
    result.to_abi()
}

pub fn _import_serde_adjacently_tagged(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<SerdeAdjacentlyTagged>(&mut env, arg);
    let result = super::import_serde_adjacently_tagged(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_serde_enum(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<SerdeVariantRenaming>(&mut env, arg);
    let result = super::import_serde_enum(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_serde_flatten(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<SerdeFlatten>(&mut env, arg);
    let result = super::import_serde_flatten(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_serde_internally_tagged(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<SerdeInternallyTagged>(&mut env, arg);
    let result = super::import_serde_internally_tagged(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_serde_struct(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<SerdePropertyRenaming>(&mut env, arg);
    let result = super::import_serde_struct(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_serde_untagged(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    arg: FatPtr,
) -> FatPtr {
    let arg = import_from_guest::<SerdeUntagged>(&mut env, arg);
    let result = super::import_serde_untagged(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_string(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<String>(&mut env, arg);
    let result = super::import_string(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_timestamp(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, arg: FatPtr) -> FatPtr {
    let arg = import_from_guest::<MyDateTime>(&mut env, arg);
    let result = super::import_timestamp(arg);
    export_to_guest(&mut env, &result)
}

pub fn _import_void_function(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>) {
    let result = super::import_void_function();
}

pub fn _import_void_function_empty_result(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
) -> FatPtr {
    let result = super::import_void_function_empty_result();
    export_to_guest(&mut env, &result)
}

pub fn _import_void_function_empty_return(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>) {
    let result = super::import_void_function_empty_return();
}

pub fn _log(mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>, message: FatPtr) {
    let message = import_from_guest::<String>(&mut env, message);
    let result = super::log(message);
}

pub fn _make_http_request(
    mut env: FunctionEnvMut<Arc<RuntimeInstanceData>>,
    request: FatPtr,
) -> FatPtr {
    let request = import_from_guest::<Request>(&mut env, request);
    let result = super::make_http_request(request);
    let async_ptr = create_future_value(&mut env);
    let result: Vec<u8> = std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async move { rmp_serde::to_vec(&result.await).unwrap() })
    })
    .join()
    .unwrap();

    let result_ptr = export_to_guest_raw(&mut env, &result);
    env.data()
        .clone()
        .guest_resolve_async_value(&mut env.as_store_mut(), async_ptr, result_ptr);

    async_ptr
}
