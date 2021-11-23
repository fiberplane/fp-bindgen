// This file is a modified version from wasm-bindgen-futures.
// See: https://github.com/rustwasm/wasm-bindgen/blob/master/crates/futures/src/task/singlethread.rs
// Licensed under Apache/MIT

use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

use crate::common::mem::{to_fat_ptr, FatPtr};
use crate::common::r#async::AsyncValue;
use crate::guest::io::export_value_to_host;

use super::host_resolve_async_value;

struct Inner {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
    waker: Waker,
}

pub struct Task {
    // The actual Future that we're executing as part of this task.
    //
    // This is an Option so that the Future can be immediately dropped when it's
    // finished
    inner: RefCell<Option<Inner>>,

    // This is used to ensure that the Task will only be queued once
    is_queued: Cell<bool>,
}

impl Task {
    pub fn spawn(future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        let this = Rc::new(Self {
            inner: RefCell::new(None),
            is_queued: Cell::new(false),
        });

        let waker = unsafe { Waker::from_raw(Task::into_raw_waker(Rc::clone(&this))) };

        *this.inner.borrow_mut() = Some(Inner { future, waker });

        Task::wake_by_ref(&this);
    }

    pub fn alloc_and_spawn<FUT, RET>(future: FUT) -> FatPtr
    where
        FUT: Future<Output = RET> + 'static,
        RET: Serialize,
    {
        let layout = std::alloc::Layout::new::<AsyncValue>();
        let len = layout.size() as u32;
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        let fat_ptr = to_fat_ptr(ptr, len);

        Task::spawn(Box::pin(async move {
            let ret = future.await;
            let result_ptr = export_value_to_host(&ret);
            host_resolve_async_value(fat_ptr, result_ptr);
        }));

        fat_ptr
    }

    fn wake_by_ref(this: &Rc<Self>) {
        // If we've already been placed on the run queue then there's no need to
        // requeue ourselves since we're going to run at some point in the
        // future anyway.
        if this.is_queued.replace(true) {
            return;
        }

        super::queue::push_task(Rc::clone(this));
    }

    /// Creates a standard library `RawWaker` from an `Rc` of ourselves.
    ///
    /// Note that in general this is wildly unsafe because everything with
    /// Futures requires `Sync` + `Send` with regard to Wakers. For wasm,
    /// however, everything is guaranteed to be singlethreaded (since we're
    /// compiled without the `atomics` feature) so we "safely lie" and say our
    /// `Rc` pointer is good enough.
    unsafe fn into_raw_waker(this: Rc<Self>) -> RawWaker {
        unsafe fn raw_clone(ptr: *const ()) -> RawWaker {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const Task));
            Task::into_raw_waker((*ptr).clone())
        }

        unsafe fn raw_wake(ptr: *const ()) {
            let ptr = Rc::from_raw(ptr as *const Task);
            Task::wake_by_ref(&ptr);
        }

        unsafe fn raw_wake_by_ref(ptr: *const ()) {
            let ptr = ManuallyDrop::new(Rc::from_raw(ptr as *const Task));
            Task::wake_by_ref(&ptr);
        }

        unsafe fn raw_drop(ptr: *const ()) {
            drop(Rc::from_raw(ptr as *const Task));
        }

        const VTABLE: RawWakerVTable =
            RawWakerVTable::new(raw_clone, raw_wake, raw_wake_by_ref, raw_drop);

        RawWaker::new(Rc::into_raw(this) as *const (), &VTABLE)
    }

    pub(crate) fn run(&self) {
        let mut borrow = self.inner.borrow_mut();

        // Wakeups can come in after a Future has finished and been destroyed,
        // so handle this gracefully by just ignoring the request to run.
        let inner = match borrow.as_mut() {
            Some(inner) => inner,
            None => return,
        };

        // Ensure that if poll calls `waker.wake()` we can get enqueued back on
        // the run queue.
        self.is_queued.set(false);

        let poll = {
            let mut cx = Context::from_waker(&inner.waker);
            inner.future.as_mut().poll(&mut cx)
        };

        // If a future has finished (`Ready`) then clean up resources associated
        // with the future ASAP. This ensures that we don't keep anything extra
        // alive in-memory by accident. Our own struct, `Rc<Task>` won't
        // actually go away until all wakers referencing us go away, which may
        // take quite some time, so ensure that the heaviest of resources are
        // released early.
        if poll.is_ready() {
            *borrow = None;
        }
    }
}
