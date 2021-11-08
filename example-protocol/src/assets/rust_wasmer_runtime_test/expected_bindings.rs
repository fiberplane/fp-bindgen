use super::types::*;
use crate::errors::InvocationError;
use crate::{
    support::{
        create_future_value, export_to_guest, import_from_guest, resolve_async_value,
        FatPtr, ModuleRawFuture,
    },
    Runtime, RuntimeInstanceData,
};
use wasmer::{imports, Function, ImportObject, Instance, Store, Value, WasmerEnv};

impl Runtime {
    pub async fn fetch_data(&self, url: String) -> Result<String, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let url = export_to_guest(&env, &url);

        let function = instance
            .exports
            .get_function("__fp_gen_fetch_data")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[url.into()])?;

        let async_ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        let raw_result = ModuleRawFuture::new(env.clone(), async_ptr).await;
        Ok(rmp_serde::from_slice(&raw_result).unwrap())
    }

    pub async fn my_async_exported_function(&self) -> Result<ComplexGuestToHost, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let function = instance
            .exports
            .get_function("__fp_gen_my_async_exported_function")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[])?;

        let async_ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        let raw_result = ModuleRawFuture::new(env.clone(), async_ptr).await;
        Ok(rmp_serde::from_slice(&raw_result).unwrap())
    }

    pub fn my_complex_exported_function(&self, a: ComplexHostToGuest) -> Result<ComplexAlias, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let a = export_to_guest(&env, &a);

        let function = instance
            .exports
            .get_function("__fp_gen_my_complex_exported_function")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[a.into()])?;

        let ptr: FatPtr = match result[0] {
            Value::I64(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        };

        Ok(import_from_guest(&env, ptr))
    }

    pub fn my_plain_exported_function(&self, a: u32, b: u32) -> Result<u32, InvocationError> {
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(self.module.store(), &env);
        let instance = Instance::new(&self.module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();

        let function = instance
            .exports
            .get_function("__fp_gen_my_plain_exported_function")
            .map_err(|_| InvocationError::FunctionNotExported)?;
        let result = function.call(&[a.into(), b.into()])?;

        match result[0] {
            Value::I32(v) => unsafe { std::mem::transmute(v) },
            _ => return Err(InvocationError::UnexpectedReturnType),
        }
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
        "fp" => {
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
            "__fp_gen_count_words" => Function::new_native_with_env(store, env.clone(), _count_words),
            "__fp_gen_log" => Function::new_native_with_env(store, env.clone(), _log),
            "__fp_gen_make_request" => Function::new_native_with_env(store, env.clone(), _make_request),
            "__fp_gen_my_async_imported_function" => Function::new_native_with_env(store, env.clone(), _my_async_imported_function),
            "__fp_gen_my_complex_imported_function" => Function::new_native_with_env(store, env.clone(), _my_complex_imported_function),
            "__fp_gen_my_plain_imported_function" => Function::new_native_with_env(store, env.clone(), _my_plain_imported_function),
        }
    }
}

pub fn _count_words(env: &RuntimeInstanceData, string: FatPtr) -> FatPtr {
    let string = import_from_guest::<String>(env, string);

    export_to_guest(env, &super::count_words(string))
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);

    super::log(message);
}

pub fn _make_request(env: &RuntimeInstanceData, opts: FatPtr) -> FatPtr {
    let opts = import_from_guest::<RequestOptions>(env, opts);

    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result_ptr = export_to_guest(&env, &super::make_request(opts).await);

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
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result_ptr = export_to_guest(&env, &super::my_async_imported_function().await);

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

    export_to_guest(env, &super::my_complex_imported_function(a))
}

pub fn _my_plain_imported_function(env: &RuntimeInstanceData, a: u32, b: u32) -> u32 {
    super::my_plain_imported_function(a, b)
}
