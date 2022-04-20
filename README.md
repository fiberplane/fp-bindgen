# fp-bindgen

[![Crates.io](https://img.shields.io/crates/v/fp-bindgen.svg)](https://crates.io/crates/fp-bindgen)
[![Discord Shield](https://discordapp.com/api/guilds/950489382626951178/widget.png?style=shield)](https://discord.gg/59ZANEFm)

Bindings generator for full-stack WASM plugins.

## Comparison to other "bindgen" tools

`fp-bindgen` is not the only tool for generating Wasm bindings. The most well-known tool for this
is probably `wasm-bindgen`, though it is limited to Rust modules running inside browser
environments. A more generic alternative, based on the Wasm
[interface types proposal](https://github.com/WebAssembly/interface-types/blob/main/proposals/interface-types/Explainer.md),
is `wit-bindgen`. We do believe interface types to be the future of Wasm bindings, but for the
short-term, `fp-bindgen` provides bindings that work with a stable serialization format, which helps
us to avoid versioning issues and opens up compatibility with tools such as Serde.

It is worth mentioning that, though we have a [specification](#specification) for our communication
primitives that allows generators for other languages to be contributed, `fp-bindgen` is opinionated
towards Rust. It uses Rust data structures and function signatures as its "protocol format",
enabling tight integration with existing crates from the Rust ecosystem.

The following table is intended to highlight the major differences between the different tools:

| Feature                                                   |         `fp-bindgen`        | `wasm-bindgen` |         `wit-bindgen`           |
| --------------------------------------------------------- | :-------------------------: | :------------: | :-----------------------------: |
| Host environments                                         | Rust (Wasmer), TypeScript\* |     JS/TS      | Rust/Python (Wasmtime), JS/TS\* |
| Guest languages                                           |            Rust\*           |      Rust      |           Rust, C\*             |
| Protocol format                                           |     Rust (using macros)     |      N/A       |              .wit               |
| Serialization format                                      |         MessagePack         |      JSON      |             Custom              |
| [Can use existing Rust types](#using-existing-rust-types) |           &#9989;           |    &#10060;    |            &#10060;             |

\*) These are only the _currently supported_ options. More may be added in the future.

## Usage

Using `fp-bindgen` is a three-step process:

- First you [define a protocol](#defining-a-protocol) that specifies the functions and data
  structures available for communication across the Wasm bridge.
- Then you [generate the bindings](#generating-bindings) for the hosts and plugin language that are
  relevant to you.
- Finally, you can start [implementing plugins and runtimes](#using-the-bindings) using the
  generated bindings.

## Defining a protocol

Before you can generate bindings using this library, you first define a protocol of functions that
can be called by the _runtime_ (the Wasm host) and functions that can be called by the _plugin_ (the
Wasm guest module). The protocol specifies the function declarations, which are placed inside two
macros: `fp_import!` and `fp_export!`. These macros specify which functions can be imported and
which can be exported, _from the perspective of the plugin_. In other words, `fp_import!` functions
can be called by the plugin and must be implemented by the runtime, while `fp_export!` functions can
be called by the runtime and _may_ be implemented by the plugin.

**Example:**

```rust
fp_import! {
    fn my_imported_function(a: u32, b: u32) -> u32;
}

fp_export! {
    fn my_exported_function(a: u32, b: u32) -> u32;
}
```

**Important caveat:** There must be exactly one `fp_import!` block and one `fp_export!` block in the
same module as where you invoke `fp_bindgen!()`. If you only have imports, or only have exports, you
should create an empty block for the other.

### Data structures

Besides primitives, functions can pass Rust `struct`s and `enum`s as their arguments and return
value, but only by value (passing a reference across the Wasm bridge is currently not supported) and
only for types that implement `Serializable`.

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

Functions can also be `async`, which works as you would expect:

**Example:**

```rust
fp_import! {
    async fn my_async_function(data: MyStruct) -> Result<MyStruct, MyError>;
}
```

### Using existing Rust types

Sometimes you may wish to use Rust types for your protocol that you also want to use directly in the
generated runtime or plugin implementation. In such a case, generation of the data types might force
you to perform unnecessary copies, so we allow explicit annotations to import the existing
definition instead of generating a new one:

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

In this example, `MyStruct` has a double function: it acts both as a type definition for the
protocol (through `fp-bindgen`'s `Serializable` trait), which can still be used for generating a
TypeScript type definition, for instance. _And_ it acts as a type that can be directly used by the
Rust Wasmer runtime, under the assumption the runtime can import it from `my_crate::prelude`.

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

- `http-compat`: Enables compatibility with types from the `http` crate.
- `serde-bytes-compat`: Enables compatibility with `serde_bytes`'s `ByteBuf` type (the `Bytes` type
  is a reference type, which `fp-bindgen` doesn't support in general).
- `time-compat`: Enables compatibility with `time`'s `PrimitiveDateTime` and `OffsetDateTime` types.

## Generating bindings

To generate bindings based on your protocol, you first need to create a function that will generate
them for you. Creating this function is easy, because its implementation can be created for you
using the `fp_bindgen` macro:

```rust
let bindings_type = BindingsType::RustWasmerRuntime;
fp_bindgen!(BindingConfig {
    bindings_type,
    path: &format!("bindings/{}", bindings_type)
});
```

Currently, we support the following binding types:

- `BindingsType::RustPlugin`: Generates bindings for a Rust plugin.
- `BindingsType::RustWasmerRuntime`: Generates runtime bindings for use with Wasmer.
- `BindingsType::TsRuntime`: Generates bindings for a TypeScript runtime.

Note that some binding types take an additional config argument.

## Using the bindings

How to use the generated bindings differs between the various types.

### Using the Rust plugin bindings

The generator for our Rust plugin bindings generates a complete crate that allows to be linked
against by plugins. The plugin can import all the functions from the `fp_import!` block from it,
and call them like any other functions.

In order to export the functions that are defined in the `fp_export!` block, it can use the exported
`fp_export_impl` macro, like so:

```rust
#[fp_export_impl(bindings_crate_path)]
fn my_exported_function(a: u32, b: u32) -> u32 {
    /* ... */
}
```

`bindings_crate_path` is expected to match with the module path from which the bindings crate
itself is imported. The function signature must match exactly with one of the `fp_export!`
functions.

When compiling a plugin, don't forget to compile against the "wasm32-unknown-unknown" target, or you
will receive linker errors.

See the `example-plugin/` directory for an example of a plugin that uses bindings generated from
our `example-protocol/` (do note this plugin only builds after you've run `cargo run` inside the
`example-protocol/` directory).

### Using the Rust Wasmer runtime bindings

The generator for our Rust Wasmer runtime works a bit differently. Instead of generating a crate,
it generates two files: `bindings.rs` and `types.rs`. These can be placed in a module of your
choosing (we chose a module named `spec` in the `example-rust-runtime/`).

As the implementor of the runtime, it is then your responsibility to implement the `fp_import!`
functions within the same module as you've placed the generated files. You can see an example of
this in `example-rust-runtime/spec/mod.rs` (do note the example runtime only builds after you've run
`cargo run` inside the `example-protocol/` directory).

Finally, the `bindings.rs` file contains a constructor (`Runtime::new()`) that you can use to
instantiate Wasmer runtimes with the Wasm module provided as a blob. The `fp_export!` functions are
provided on the `Runtime` instance as methods. Please be aware that implementation of the
`fp_export!` functions is always at the discretion of the plugin, and an attempt to invoke a missing
implementation can fail with an `InvocationError::FunctionNotExported` error.

### Using the TypeScript runtime bindings

The TypeScript runtime generator can work with browsers, Node.js and Deno.

It works similarly to that for the Wasmer runtime, but it generates an `index.ts` and a `types.ts`.
`types.ts` contains the type definitions for all the data structures, while the `index.ts` exports a
`createRuntime()` function that you can use for instantiating the runtime. Upon instantiation, you
are expected to provide implementations for all the `fp_import!` functions, while the returned
`Promise` will give you an object with all the `fp_export!` functions the provided plugin has
implemented.

## Examples

Please have a look at [`examples/README.md`](examples/README.md) for various examples on how to use
`fp-bindgen`.

## Specification

We have written down a specification that describes the primitives used by our bindings. This is
aimed primarily at those that want to understand how the bindings work under the hood, and may be
valuable if you want to implement bindings for your own favorite language.

If that is you, please have a look at [`docs/SPEC.md`](docs/SPEC.md).

## Known Limitations

- Data types may only contain value types. References are currently unsupported.
- Referencing types using their full module path is prone to cause mismatches during type
  discovery. Please import types using a `use` statement and refer to them by their name only.
- TypeScript bindings handle 64-bit integers somewhat inconsistently. When passed as primitives (as
  plain function arguments or return values) they will be encoded using the `BigInt` type. But when
  they're part of a MessagePack-encoded data type, they will be encoded using `number`, which
  effectively limits them to a maximum size of `2^53 - 1`. For more information, see:
  https://github.com/msgpack/msgpack-javascript/issues/115

## FAQ

### I added a `Serializable` derive to my type, why don't I see it included in the bindings?

Are you using the type in one of the `fp_import!` or `fp_export!` functions? Deriving `Serializable`
makes it possible to use the type as part of your protocol, but it won't become part of the
generated bindings until it is actually referenced. Note that types can be either referenced
directly by one of the `fp_import!` or `fp_export!` functions, or indirectly by another type that is
already in use.

If a type is not referenced either directly or indirectly by any of the functions that are part of
your protocol, you can force inclusion by adding a `use` statement referencing the type to either
the `fp_import!` or `fp_export!` section:

```rs
fp_import! {
    use MyType;
}
```

Are you referencing the type and it is still not included in your bindings? Please
[file an issue](https://github.com/fiberplane/fp-bindgen/issues).

### Can I use aliases?

Yes, but because aliases cannot have a derive macro, please repeat the alias in either the
`fp_import!` or `fp_export!` section:

```rs
fp_import! {
  type MyType = SomeOtherType;
}
```

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

## Community

Do you want help using `fp-bindgen`? Want to discuss new features?

Please come visit us in the `#fp-bindgen` channel on our [Discord](https://discord.gg/59ZANEFm).

## Contributing

Please follow our [Contributing Guidelines](CONTRIBUTING.md) to learn how best to contribute to
this project.

## License

This project is licensed under Apache License, version 2.0
([LICENSE.txt](https://github.com/fiberplane/fp-bindgen/blob/main/LICENSE.txt)).
