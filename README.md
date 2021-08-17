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
Wasm guest module). The protocol specifies the function declarations, put inside `fp-bindgen`
macros: `fp_import!` and `fp_export!`. These macros specify which functions can be imported and
which can be exported, _from the perspective of the plugin_. In other words, `fp_import!` functions
can be called by the plugin and must be implemented by the runtime, while `fp_export!` functions can
be called by the runtime and may be implemented by the plugin.

**Example:**

```rust
fp_import! {
    fn my_imported_function(a: u32, b: u32) -> u32;
}

fp_export! {
    fn my_exported_function(a: u32, b: u32) -> u32;
}
```

Functions can pass Rust `struct`s and `enum`s as their arguments and return value, but only by value
(passing a reference across the Wasm bridge is currently not supported) and only for types that
implement `Serializable`.

**Example:**

```rust
#[derive(Serializable)]
pub struct MyStruct {
    pub foo: i32,
    pub bar: String,
}

fp_import! {
    fn my_function(data: MyStruct) -> MyStruct;
}
```

Note that `Serialize` and `Deserialize` are implemented by default for some common standard types,
such as `Option`, `Box`, and `Result`.

Functions can also be `async`, which can be achieved by nothing more than putting the `async`
keyword in front of the function declaration.

**Example:**

```rust
fp_import! {
    async fn my_async_function(data: MyStruct) -> Result<MyStruct, MyError>;
}
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

**Important caveat:** There must be exactly one `fp_import!` block and one `fp_export!` block in the
same module as where you invoke `fp_bindgen!()`. If you only have imports, or only have exports, you
should create an empty block for the other.

## Examples

For examples, please look at the `example/` folder. This contains various examples on how to use
the macros. You can run the project using `cargo run generate <bindings-type>`, where
`<bindings-type>` should be replaced with one of the types mentioned above.

## FAQ

### Why not utilize [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs) for the TypeScript generator?

The `derive` macro exported by `ts-rs` parses Rust data structures into a `DerivedTS` struct. As the
name suggests, this struct is highly TypeScript-specific, so if we were to integrate this, it would
exist in parallel to code for generating the other Rust bindings. However, because we need to
perform type extraction at a very early stage in our pipeline, this would basically result in two
disjoint pipelines: one for Rust and one for TS.

So while utilizing `ts-rs` would help us get the TS bindings off the ground fast, it wouldn't help
us with the Rust side of things at all. If anything, it would likely complicate long-term
maintenance as developers now need to understand two completely separate pipelines, and the needs of
`ts-rs` and this crate would need to be tightly synchronized (there would be pretty tight coupling
between the two).

As a final nail in the coffin, the Rust pipeline would need to be generic anyway, because it will be
used for generating both the runtime and the plugin bindings. Extending it so the TypeScript
generator can also work on top of the same pipeline was only a small step at that point.

Even so, having previously contributed to the `ts-rs` project, we'd like to extend our thanks to
everyone who has contributed to it for the inspiration it gave us.
