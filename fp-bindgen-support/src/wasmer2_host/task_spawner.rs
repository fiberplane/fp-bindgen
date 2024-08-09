use crate::common::mem::FatPtr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::pin::Pin;
use std::future::Future;
use std::task::Waker;
use tokio::task::LocalSet;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::runtime::Builder;
use wasmer::{LazyInit, Memory, NativeFunc};
use std::any::Any;
use once_cell::sync::OnceCell;

type BoxedFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;
type Task = (BoxedFuture, mpsc::Sender<Box<dyn Any + Send>>);

#[derive(Clone, Debug)]
pub struct CurrentThreadSpawner {
    sender: mpsc::Sender<Task>,
}

#[derive(Clone, Default)]
pub struct GlobalSpawner {
    sender: Arc<Option<mpsc::Sender<Task>>>,
}

pub static GLOBAL_HANDLER: OnceCell<GlobalSpawner> = OnceCell::new();

impl GlobalSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        let (sender, mut receiver) = mpsc::channel::<Task>(100);
        handle.spawn(async move {
            loop {
                if let Some((task, result_sender)) = receiver.recv().await {
                    let task = async move {
                        let result = task.await;
                        let _ = result_sender.send(result).await;
                    };
                    tokio::spawn(task);
                }
            }
        });

        GlobalSpawner { sender: Arc::new(Some(sender)) }
    }

    // spawn and return immediatelly, used in async context
    pub fn spawn<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Any + Send + 'static
    {
        let (result_sender, result_receiver) = mpsc::channel(1);
        let task = Box::pin(async move {Box::new(task.await) as Box<dyn Any + Send> });
        let _ = self.sender.clone().as_ref().clone().unwrap().try_send((task, result_sender));
        result_receiver
    }

    // spawn and wait, used in sync context
    pub fn spawn_blocking<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Any + Send + 'static
    {
        let (result_sender, result_receiver) = mpsc::channel(1);
        let task = Box::pin(async move {Box::new(task.await) as Box<dyn Any + Send> });
        let _ = self.sender.clone().as_ref().clone().unwrap().blocking_send((task, result_sender));
        result_receiver
    }
    //
    // // spawn and wait(in async) until there is capacity, used in sync context
    pub async fn spawn_async<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Any + Send + 'static
    {
        let (result_sender, result_receiver) = mpsc::channel(1);
        let task = Box::pin(async move {Box::new(task.await) as Box<dyn Any + Send> });
        let _ = self.sender.clone().as_ref().clone().unwrap().send((task, result_sender)).await;
        result_receiver
    }
}

impl CurrentThreadSpawner {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::channel::<Task>(100);

        let rt = Builder::new_current_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        std::thread::spawn(move || {
            rt.block_on(async move {
                loop {
                    if let Some((task, result_sender)) = receiver.recv().await {
                        let task = async move {
                            let result = task.await;
                            let _ = result_sender.send(result).await;
                        };
                        tokio::spawn(task);
                    }
                }
            });
        });

        CurrentThreadSpawner { sender }
    }

    // spawn and return immediatelly, used in async context
    pub fn spawn<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Any + Send + 'static
    {
        let (result_sender, result_receiver) = mpsc::channel(1);
        let task = Box::pin(async move {Box::new(task.await) as Box<dyn Any + Send> });
        let _ = self.sender.try_send((task, result_sender));
        result_receiver
    }

    // spawn and wait, used in sync context
    pub fn spawn_blocking<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Any + Send + 'static
    {
        let (result_sender, result_receiver) = mpsc::channel(1);
        let task = Box::pin(async move {Box::new(task.await) as Box<dyn Any + Send> });
        let _ = self.sender.blocking_send((task, result_sender));
        result_receiver
    }
    //
    // // spawn and wait(in async) until there is capacity, used in sync context
    pub async fn spawn_async<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
    where
        F: Future<Output = T> + Send + 'static,
        T: Any + Send + 'static
    {
        let (result_sender, result_receiver) = mpsc::channel(1);
        let task = Box::pin(async move {Box::new(task.await) as Box<dyn Any + Send> });
        let _ = self.sender.send((task, result_sender)).await;
        result_receiver
    }
}
