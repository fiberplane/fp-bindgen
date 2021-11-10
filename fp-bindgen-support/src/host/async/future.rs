// The ModuleFuture implements the Future Trait to handle async Futures as
// returned from the module.
pub(crate) struct ModuleFuture<T> {
    pub ptr: FatPtr,
    pub env: RuntimeInstanceData,

    _p: PhantomData<T>,
}

impl<T> ModuleFuture<T> {
    pub fn new(env: RuntimeInstanceData, ptr: FatPtr) -> Self {
        Self {
            ptr,
            env,
            _p: PhantomData,
        }
    }
}

impl<'de, T> Future for ModuleFuture<T>
where
    T: Deserialize<'de>,
{
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let memory = unsafe { self.env.memory.get_unchecked() };

        let ptr = self.ptr;

        let (async_ptr, async_len) = to_wasm_ptr(ptr);
        let values = async_ptr.deref(memory, 0, async_len).unwrap();

        match values[0].get() {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.env.wakers.lock().unwrap();
                wakers.insert(ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result_ptr = values[1].get();
                let result_len = values[2].get();
                let result = import_from_guest(&self.env, to_fat_ptr(result_ptr, result_len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
}

// The ModuleRawFuture implements the Future Trait to handle async Futures as
// returned from the module.
pub(crate) struct ModuleRawFuture {
    pub ptr: FatPtr,
    pub env: RuntimeInstanceData,
}

impl ModuleRawFuture {
    pub fn new(env: RuntimeInstanceData, ptr: FatPtr) -> Self {
        Self { ptr, env }
    }
}

impl<'de> Future for ModuleRawFuture {
    type Output = Vec<u8>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let memory = unsafe { self.env.memory.get_unchecked() };

        let ptr = self.ptr;

        let (async_ptr, async_len) = to_wasm_ptr(ptr);
        let values = async_ptr.deref(memory, 0, async_len).unwrap();

        match values[0].get() {
            FUTURE_STATUS_PENDING => {
                let mut wakers = self.env.wakers.lock().unwrap();
                wakers.insert(ptr, cx.waker().clone());
                Poll::Pending
            }
            FUTURE_STATUS_READY => {
                let result_ptr = values[1].get();
                let result_len = values[2].get();
                let result = import_from_guest_raw(&self.env, to_fat_ptr(result_ptr, result_len));
                Poll::Ready(result)
            }
            value => panic!(
                "expected async value FUTURE_STATUS_PENDING ({}) or FUTURE_STATUS_READY ({}) but got: {}",
                FUTURE_STATUS_PENDING, FUTURE_STATUS_READY, value
            ),
        }
    }
}
