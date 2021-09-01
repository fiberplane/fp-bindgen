// This file is a modified version from wasm-bindgen-futures.
// See: https://github.com/rustwasm/wasm-bindgen/blob/master/crates/futures/src/queue.rs
// Licensed under Apache/MIT

use once_cell::unsync::Lazy;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

struct QueueState {
    // The queue of Tasks which will be run in order. In practice this is all the
    // synchronous work of futures, and each `Task` represents calling `poll` on
    // a future "at the right time"
    tasks: RefCell<VecDeque<Rc<super::task::Task>>>,

    // This flag indicates whether we're currently executing inside of
    // `run_all` or have scheduled `run_all` to run in the future. This is
    // used to ensure that it's only scheduled once.
    is_spinning: Cell<bool>,
}

impl QueueState {
    fn run_all(&self) {
        debug_assert!(self.is_spinning.get());

        // Runs all Tasks until empty. This blocks the event loop if a Future is
        // stuck in an infinite loop, so we may want to yield back to the main
        // event loop occasionally. For now though greedy execution should get
        // the job done.
        while let Some(task) = self.tasks.borrow_mut().pop_front() {
            task.run();
        }

        // All of the Tasks have been run, so it's now possible to schedule the
        // next tick again
        self.is_spinning.set(false);
    }
}

struct Queue {
    state: Rc<QueueState>,
}

impl Queue {
    fn push_task(&self, task: Rc<super::task::Task>) {
        self.state.tasks.borrow_mut().push_back(task);

        // If we're already inside the `run_all` loop then that'll pick up the
        // task we just enqueued. If we're not in `run_all`, though, we
        // synchronously start running.
        if !self.state.is_spinning.replace(true) {
            self.state.run_all();
        }
    }
}

impl Queue {
    fn new() -> Self {
        let state = Rc::new(QueueState {
            is_spinning: Cell::new(false),
            tasks: RefCell::new(VecDeque::new()),
        });

        Self { state }
    }
}

static mut QUEUE: Lazy<Queue> = Lazy::new(Queue::new);

pub(crate) fn push_task(task: Rc<super::task::Task>) {
    unsafe { QUEUE.push_task(task) }
}
