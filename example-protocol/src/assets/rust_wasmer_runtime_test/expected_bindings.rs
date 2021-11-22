use super::types::*;
use crate::errors::InvocationError;
use crate::{
    support::{
        create_future_value, export_to_guest, export_to_guest_raw, import_from_guest,
        resolve_async_value, FatPtr, ModuleRawFuture,
    },
    Runtime, RuntimeInstanceData,
};
use wasmer::{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv};

impl Runtime {
    pub async fn fetch_data(&self, url: String) -> Result<String, InvocationError> {
        let url = serialize_to_vec(url);
        let res = self.fetch_data_raw(url);
        let res = res.await?;
        let mut deserializer = rmp_serde::Deserializer::new(&res).with_human_readable();
        String::deserialize(&mut deserializer).unwrap()
    }
    pub async fn fetch_data_raw(&self, url: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        let url = export_to_guest_raw(&env, url);
        let function = instance
            .exports
            .get_native_function::<(FatPtr), FatPtr>("__fp_gen_fetch_data")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call((url))?;
        let result = ModuleRawFuture::new(env.clone(), result).await;
        Ok(result)
    }

    pub async fn my_async_exported_function(&self) -> Result<ComplexGuestToHost, InvocationError> {
        let res = self.my_async_exported_function_raw();
        let res = res.await?;
        let mut deserializer = rmp_serde::Deserializer::new(&res).with_human_readable();
        ComplexGuestToHost::deserialize(&mut deserializer).unwrap()
    }
    pub async fn my_async_exported_function_raw(&self) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        let function = instance
            .exports
            .get_native_function::<(), FatPtr>("__fp_gen_my_async_exported_function")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(())?;
        let result = ModuleRawFuture::new(env.clone(), result).await;
        Ok(result)
    }

    pub fn my_complex_exported_function(
        &self,
        a: ComplexHostToGuest,
    ) -> Result<ComplexAlias, InvocationError> {
        let a = serialize_to_vec(a);
        let res = self.my_complex_exported_function_raw(a);
        let mut deserializer = rmp_serde::Deserializer::new(&res).with_human_readable();
        ComplexAlias::deserialize(&mut deserializer).unwrap()
    }
    pub fn my_complex_exported_function_raw(&self, a: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        let a = export_to_guest_raw(&env, a);
        let function = instance
            .exports
            .get_native_function::<(FatPtr), FatPtr>("__fp_gen_my_complex_exported_function")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call((a))?;
        let result = import_from_guest_raw(&env, result);
        Ok(result)
    }

    pub fn my_plain_exported_function(&self, a: u32, b: u32) -> Result<u32, InvocationError> {
        let res = self.my_plain_exported_function_raw(a, b);
        let mut deserializer = rmp_serde::Deserializer::new(&res).with_human_readable();
        u32::deserialize(&mut deserializer).unwrap()
    }
    pub fn my_plain_exported_function_raw(
        &self,
        a: u32,
        b: u32,
    ) -> Result<Vec<u8>, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        let function = instance
            .exports
            .get_native_function::<(u32, u32), u32>("__fp_gen_my_plain_exported_function")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call((a, b))?;
        let result = import_from_guest_raw(&env, result);
        Ok(result)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
       "fp" => {
           "__fp_host_resolve_async_value" => Function :: new_native_with_env (store , env . clone () , resolve_async_value) ,
           "__fp_gen_count_words" => Function :: new_native_with_env (store , env . clone () , _count_words) ,
           "__fp_gen_log" => Function :: new_native_with_env (store , env . clone () , _log) ,
           "__fp_gen_make_request" => Function :: new_native_with_env (store , env . clone () , _make_request) ,
           "__fp_gen_my_async_imported_function" => Function :: new_native_with_env (store , env . clone () , _my_async_imported_function) ,
           "__fp_gen_my_complex_imported_function" => Function :: new_native_with_env (store , env . clone () , _my_complex_imported_function) ,
           "__fp_gen_my_plain_imported_function" => Function :: new_native_with_env (store , env . clone () , _my_plain_imported_function) ,
        }
    }
}

pub fn _count_words(env: &RuntimeInstanceData, string: FatPtr) -> FatPtr {
    let string = import_from_guest::<String>(env, string);
    let result = super::count_words(string);
    export_to_guest(env, &result)
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);
    let result = super::log(message);
    ()
}

pub fn _make_request(env: &RuntimeInstanceData, opts: FatPtr) -> FatPtr {
    let opts = import_from_guest::<RequestOptions>(env, opts);
    let result = super::make_request(opts);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = result.await;
        let result_ptr = export_to_guest(&env, &result);
        unsafe {
            env.__fp_guest_resolve_async_value
                .get_unchecked()
                .call(async_ptr, result_ptr)
                .expect("Runtime error: Cannot resolve async value");
        }
    });
    async_ptr
}

pub fn _my_async_imported_function(env: &RuntimeInstanceData) -> FatPtr {
    let result = super::my_async_imported_function();
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = result.await;
        let result_ptr = export_to_guest(&env, &result);
        unsafe {
            env.__fp_guest_resolve_async_value
                .get_unchecked()
                .call(async_ptr, result_ptr)
                .expect("Runtime error: Cannot resolve async value");
        }
    });
    async_ptr
}

pub fn _my_complex_imported_function(env: &RuntimeInstanceData, a: FatPtr) -> FatPtr {
    let a = import_from_guest::<ComplexAlias>(env, a);
    let result = super::my_complex_imported_function(a);
    export_to_guest(env, &result)
}

pub fn _my_plain_imported_function(env: &RuntimeInstanceData, a: u32, b: u32) -> u32 {
    let result = super::my_plain_imported_function(a, b);
    result
}
