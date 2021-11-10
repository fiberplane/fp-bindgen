# fp-bindgen

Macros for generating WebAssembly bindings.

## Comparison to `wasm-bindgen`

`fp-bindgen` is an alternative to `wasm-bindgen`, but diverges primarily in two ways:

- It doesn't assume the Wasm host is a browser or other JS-based runtime. Instead, it can generate
  bindings for both JS and Rust runtimes and our communication primitives are
  [documented](#specification) so that bindings for other languages can be contributed.
- It uses [MessagePack](https://msgpack.org/index.html) for serializing complex data structures, to
  improve performance over `wasm-bindgen`'s JSON serialization.

## Usage

Using `fp-bindgen` is a three-step process:

- First you [define a protocol](#defining-a-protocol) that specifies the functions and data
  structures available for communication across the Wasm bridge.
- Then you [generate the bindings](#generating-bindings) for the hosts and plugin language that are
  relevant to you.
- Finally, you can start implementing plugins and runtimes using the generated bindings.

## Defining a protocol

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

Note that `Serializable` is implemented by default for some common standard types, such as
`Option`, `Vec`, and other container types.

### Async functions

Functions can also be `async`, which can be achieved by nothing more than putting the `async`
keyword in front of the function declaration.

**Example:**

```rust
fp_import! {
    async fn my_async_function(data: MyStruct) -> Result<MyStruct, MyError>;
}
```

### Using existing Rust types

Sometimes you may wish to use Rust types for your protocol that you want to use directly in the
generated runtime or plugin implementation. In such a case, generation of the data types would be
a hindrance, rather than an aid, so we allow explicit annotations to achieve this:

**Example:**

```rust
#[derive(Deserialize, Serialize, Serializable)]
#[fp(rust_wasmer_runtime_module = "my_crate::prelude")]
#[serde(rename_all = "camelCase")]
pub struct MyStruct {
    pub foo: i32,
    pub bar_qux: String,
}
```

In this example, `MyStruct` has a double duty: it acts both as a type definition for the protocol
(through `fp-bindgen`'s `Serializable` trait), which can still be used for generating a TypeScript
type definition, for instance. _And_ it acts as a type that can be directly used by the Rust Wasmer
runtime, under the assumption the runtime can import it from `my_crate::prelude`.

Please note that in this case, you do have a bigger responsibility to make sure the definition
fulfills the requirements of the code generator, hence why Serde's trait derives and annotations
have to be added manually here, in accordance with how the generator would otherwise generate them.

For now, this feature is limited to the Rust generators through either the
`rust_wasmer_runtime_module` or `rust_plugin_module` annotations. For us, this makes sense given the
protocol itself is specified using Rust syntax as well. If desired, we could extend this to the
TypeScript generator as well, though that would imply an even bigger responsibility for the user to
keep their TypeScript types in sync with the protocol.

### Cargo features

The `fp-bindgen` crate supports optional Cargo features for compatibility with some common types
from the crate ecosystem:

- `chrono-compat`: Enables compatibility with Chrono's `DateTime<Utc>` type (other timezones are
  currently not supported).
- `http-compat`: Enables compatibility with types from the `http` crate.
- `serde-bytes-compat`: Enables compatibility with `serde_bytes`' `ByteBuf` type (the `Bytes` type
  is a reference type, which `fp-bindgen` doesn't support in general).

## Generating bindings

To generate bindings based on your protocol, you first need to create a function that will generate
them for you. Creating this function is easy, because its implementation can be created for you
using the `fp_bindgen` macro:

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

For examples, please look at the `example-protocol/` folder. This contains various examples on how
to use the macros. You can generate the bindings from the protocol using `cargo run`. Running
`cargo test` in the same folder will verify the generated bindings match the output we expect.

There is also an `example-plugin/` directory, which is a plugin that is compiled against the
bindings generated by the `example-protocol`. This way, you can see how the bindings are expected
to be used, and we verify that the generated bindings actually compile (at least for the
`rust-plugin` bindings).

## Specification

We have written down a specification that describes the primitives used by our bindings. This is
aimed primarily at those that want to understand how the bindings work under the hood, and may be
valuable if you want to implement bindings for your own favorite language.

If that is you, please have a look at [`docs/SPEC.md`](docs/SPEC.md).

## FAQ

### I added a `Serializable` derive to my type, why don't I see it included in the bindings?

Are you using the type in one of the `fp_import!` or `fp_export!` functions? Deriving `Serializable`
makes it possible to use the type as part of your protocol, but it won't become part of the
generated bindings until it is actually used. Note that usage can be either direct (referenced
directly by one of the `fp_import!` or `fp_export!` functions), or indirect when it is referenced
by another type that is already in use.

Are you using the type and you believe the omission is in error? Please
[file an issue](https://github.com/fiberplane/fp-bindgen/issues).

### Can I use aliases?

Yes, but with a few caveats. Type aliases such as these are supported:

```rs
type MyType = SomeOtherType;
```

And they will also appear as aliases in the generated bindings.

But do note that because aliases don't have a `Serializable` derive attached to themselves (only to
the type they alias), they can be tricky for us to detect. One area where their usage is currently
problematic is if you pass them as a generic argument to another type, such as `Option<MyType>`.

If this (or other issues with aliases) affect you, please
[file an issue](https://github.com/fiberplane/fp-bindgen/issues).

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

### What about versioning?

Generally, versioning is considered out-of-scope for this project. This means it is your own
responsibility to verify a plugin you execute was compiled against a compatible version of the
protocol your runtime provides.

If your protocol ever needs to introduce breaking changes, we advise to include a `version() -> u32`
export function in the protocol itself that you can call before invoking any other functions.

As for what constitutes a breaking change, we offer the following guidelines:

- All plugin exports are always optional. Because of this, new exports can always be added without
  breaking existing plugins, unless your runtime performs an explicit check that mandates an
  export's existence.
- Adding new imports is always safe, as they will simply be ignored by existing plugins.
- Adding fields to `struct`s is always safe, unless your runtime mandates the existence of such
  fields in arguments or return values coming from the plugin.
- Adding new types is always safe.
- **Anything else should be considered a breaking change.**

Note that, because of the above guidelines, you should never need to define a versioning function in
your first iteration. Because plugin exports are optional, the absense of a versioning function can
simply be interpreted as meaning the plugin is at version 1.

## License

This project is licensed under Apache License, version 2.0
([LICENSE.txt](https://github.com/fiberplane/fp-bindgen/blob/main/LICENSE.txt)).
