# the patch to support multi-thread tokio

In order to fix the panic in multi-thread tokio env (https://github.com/fiberplane/fp-bindgen/issues/194), I followed @arendjr's advice, and created a TaskSpawner, which will:

1. Run tokio runtime in single thread
2. handle guest async function in this spawner
3. Handle (at least) the second half of the host async function in this spawner

## The TaskSpawner

The goal of this TaskSpawneris to run all async tasks in a dedicated single thread runtime. By referring to Tokio book and some help from ChatGPT, I got this TaskSpawner, which has:

1. Use `tokio::sync::mpsc` channel to receive tasks
2. Support bidirectional communications: users could get results back, by creating a channel for each request
3. Support multiple result types: use `std::any::Any` to represent result in a generic way
4. Support spawn task from both async and sync context

Here is the full source code

```rust
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use std::thread;
use std::pin::Pin;
use std::future::Future;
use std::any::Any;

type BoxedFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;
type Task = (BoxedFuture, mpsc::Sender<Box<dyn Any + Send>>);

#[derive(Clone)]
struct Spawner {
    sender: mpsc::Sender<Task>,
}

impl Spawner {
    fn new() -> Self {
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

        Spawner { sender }
    }

    // spawn and return immediatelly, used in async context
    fn spawn<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
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
    fn spawn_blocking<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
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
    async fn spawn_async<F, T>(&self, task: F) -> mpsc::Receiver<Box<dyn Any + Send>>
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

fn main() {
    let spawner = Spawner::new();

    let spawner_clone = spawner.clone();
    let sync_code = thread::spawn(move || {
        let recv = spawner_clone.spawn_blocking(async {
            println!("hello from sync world");
            3.14
        });
        recv
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().unwrap();

    rt.block_on(async move {
        let mut result_receiver = spawner.spawn(async {
            println!("{:?} Hello from task", std::thread::current().id());
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            42
        });
        //
        let mut result_receiver2 = spawner.spawn_async(async {
            println!("{:?} Hello from task again", std::thread::current().id());
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            "foobar".to_owned()
        }).await;

        // Get the task result
        if let Some(result) = result_receiver2.recv().await {
            let result = result.downcast::<String>().unwrap();
            println!("Task result: {}", result);
        } else {
            println!("Task get error");
        }

        // Get the task result
        if let Some(result) = result_receiver.recv().await {
            let result = result.downcast::<i32>().unwrap();
            println!("Task result: {}", result);
        } else {
            println!("Task get error");
        }
        // Keep the main thread running
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    });

    let res = sync_code.join().unwrap().blocking_recv();
    let res = res.unwrap().downcast::<f64>().unwrap();
    println!("recv from sync block: {}", res);
}
```

With this TaskSpawner, I forward both host and guest async functions to this TaskSpawner, and it runs without any panic 

## Guest async function

### Previous version


```rust
pub async fn guest_func(&self, arg1: String) -> Result<String, InvocationError> {
    let arg1 = serialize_to_vec(&arg1);
    let result = self.guest_func_raw(arg1);
    let result = result.await;
    let result = result.map(|ref data| deserialize_from_slice(data));
    result
}
```

Without specifying runtime, `result.await` in line 4 will run in system default tokio runtime, and we will panic with multithread runtime, and that's why we have to stick with single thread runtime

### Current version

```rust
pub async fn guest_func(&self, arg1: String) -> Result<String, InvocationError> {
    let this = self.clone();
    let task = async move {
        let arg1 = serialize_to_vec(&arg1);
        let result = this.guest_func_raw(arg1);
        let result = result.await;
        let result = result.map(|ref data| deserialize_from_slice::<String>(data));
        result.unwrap()
    };
    let mut recv = SPAWNER.spawn_async(task).await;
    match recv.recv().await {
        Some(result) => Ok(*result.downcast::<String>().unwrap()),
        None => Err(InvocationError::UnexpectedReturnType),
    }
}
```

In line 10, we forward this task to the dedicated single-thread runtime (The SPAWNER)

## Host async function

### Previous version

```rust
pub fn _host_func(env: &RuntimeInstanceData, arg1: FatPtr) -> FatPtr {
    let arg1 = import_from_guest::<String>(env, arg1);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = super::host_func(arg1).await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    });
    async_ptr
}
```

### Current version

```rust
pub fn _host_func(env: &RuntimeInstanceData, arg1: FatPtr) -> FatPtr {
    let arg1 = import_from_guest::<String>(&env_clone, arg1);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let task = async move {
        let result = super::host_func(arg1).await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    };
    SPAWNER.spawn(task);
    async_ptr
}
```

The key difference is line 10. We forward this async task to runtime's dedicated single thread runtime
And with this change, at least we get expected values without panic

### A minor glitch

Now we still have a small problem: ALL host functions run in the same single thread runtime. In theory, we should make them run in the global multiple thread runtime, and only pass result in this dedicated single thread runtime. By adding another dedicated multiple-thread runtime, or passing the outmost tokio runtime created from main, we can guarantee that host functions run in multi-thread runtime


```rust
pub fn _host_func(env: &RuntimeInstanceData, arg1: FatPtr) -> FatPtr {
    let arg1 = import_from_guest::<String>(env, arg1);

    let env_clone = env.clone();
    let async_ptr = create_future_value(&env_clone);
    let host_task = MT_SPAWNER.spawn_async(async move {
        let result = super::host_func(arg1).await;
        result
    });

    let env_clone = env.clone();
    let guest_task = async move {
        let mut result = host_task.await;
        let res = match result.recv().await {
            Some(result) => *result.downcast::<String>().unwrap(),
            None => "xxx".to_string(), //TODO: fix this later
        };

        let result_ptr = export_to_guest(&env_clone, &res);
        env_clone.guest_resolve_async_value(async_ptr, result_ptr);
    };
    SG_SPAWNER.spawn(guest_task);
    async_ptr
}
```

### the extra call in main

We have to create the __global__ handler in main before calling `fp-bindgen`

```rust
use fp_bindgen_support::wasmer2_host::task_spawner::GLOBAL_HANDLER;
use fp_bindgen_support::wasmer2_host::task_spawner::GlobalSpawner;


#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let global_spawner = GlobalSpawner::new(tokio::runtime::Handle::current());
    let _ = GLOBAL_HANDLER.set(global_spawner);
    // ...
}

```

### the performance issue

This is just a rough implementation, mpsc channels and dynamic `std::any::Any` are definitely not the optimal solution, and it indeed shows poor performance in our production environment compared to the single threaded version.

So if this is the right direction, we need to spend more efforts to make it performant.
