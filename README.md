# fp-bindgen

Macros for generating WebAssembly bindings.

## Comparison to `wasm-bindgen`

`fp-bindgen` is an alternative to `wasm-bindgen`, but diverges primarily in two ways:

- It doesn't assume the Wasm host is a browser or other JS-based runtime. Instead, it can generate
  bindings for both JS and Rust runtimes. We also intend to document our communication primitives so
  that bindings for other languages can be contributed.
- It uses [MessagePack](https://msgpack.org/index.html) for serializing complex data structures, to
  improve performance over `wasm-bindgen`'s JSON serialization.

## Usage

### Defining a protocol

Before you can generate bindings using this library, you first define a protocol of functions that
can be called by the _runtime_ (the Wasm host) and functions that can be called by the _plugin_ (the
Wasm guest module). The protocol specifies the function declarations, annotated using `fp-bindgen`
macros: `fp_import` and `fp_export`. These macros specify which functions can be imported and which
can be exported, _from the perspective of the plugin_. In other words, `fp_import` functions can be
called by the plugin and must be implemented by the runtime, while `fp_export` functions can be
called by the runtime and may be implemented by the plugin.

**Example:**

```rust
#[fp_import]
fn my_imported_function(a: u32, b: u32) -> u32;

#[fp_export]
fn my_exported_function(a: u32, b: u32) -> u32;
```

Functions can pass Rust `struct`s as their arguments and return value, but only by value (passing
a reference across the Wasm bridge is currently not supported) and only for structs that implement
either `Serialize` or `Deserialize`. Whether to use `Serialize` or `Deserialize` is again determined
from the perspective of the plugin. If the plugin needs to serialize a value (arguments to
`fp_import` functions and return values of `fp_export` functions), its type must implement
`Serialize`. If the plugin needs to deserialize a value (arguments to `fp_export` functions and
return values of `fp_import` functions), its type must implement `Deserialize`.

**Example:**

```rust
#[derive(Deserialize, Serialize)]
pub struct MyStruct {
    pub foo: i32,
    pub bar: String,
}

#[fp_import]
fn my_function(data: MyStruct) -> MyStruct;
```

Note that `Serialize` and `Deserialize` are implemented by default for some common standard types,
such as `Option`, `Box`, and `Result`.

Functions can also be `async`, which can be achieved by nothing more than putting the `async`
keyword in front of the function declaration.

**Example:**

```rust
#[fp_import]
async fn my_async_function(data: MyStruct) -> Result<MyStruct, MyError>;
```

### Generating bindings

Finally, to generate bindings based on your protocol, you first need to create a function that will
generate them for you. Creating this function is easy, because its implementation can be created for
you using the `fp_bindgen` macro:

```rust
fn generate_bindings(bindings_type: &str) {
    let output_path = format!("bindings/{}", bindings_type);
    fp_bindgen!(bindings_type, &output_path);
}
```

The macro accepts two arguments: the type of bindings to generate, and where to write them.
Currently, we support the following binding types:

- `"rust-plugin"`: Generates bindings for a Rust plugin.
- `"rust-wasmer-runtime"`: Generates runtime bindings for use with Wasmer.
- `"ts-runtime"`: Generates bindings for a TypeScript runtime.

**Important caveat:** The functions annotated with `fp_import` and `fp_export` must be in the same
source file as where you invoke `fp_bindgen!()`. Splitting declarations over multiple files is prone
to lead to unexpected results due to limitations surrounding macro invocations and incremental
compilation!

## Examples

For examples, please look at the `example/` folder. This contains various examples on how to use
the macros. You can run the project using `cargo run generate <bindings-type>`, where
`<bindings-type>` should be replaced with one of the types mentioned above.
