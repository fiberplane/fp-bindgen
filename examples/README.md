# `fp-bindgen` examples

This folder contains various examples for inspiration on how to set up `fp-bindgen`. Note that we
run these examples as part of our CI, to both verify the examples themselves work, and the generated
bindings are correct.

**Make sure you run `cargo run` inside the `example-protocol/` folder first.** This is a
prerequisite in order to get the example plugin and runtimes to work.

## `example-protocol/`

This is an example of how you define a protocol. Run `cargo run` inside the folder to (re)generate
the bindings the other examples depend on.

The folder `example-protocol/src/types/` contains various serializable types that can be
communicated across the WASM bridge. They're grouped by the feature they aim to demonstrate. For
instance, [`tagged_enums.rs`](example-protocol/src/types/tagged_enums.rs) shows how to use tagged
enums.

## `example-plugin/`

This is an example of a plugin built against the example protocol. It implements all of the
`fp_export!` functions defined in the protocol.

The file [`reducer.rs`](example-plugin/src/reducer.rs) is an example of how to implement a Redux
reducer in Rust.

Note that the plugin also depends on the [`redux-example`](#redux-example) crate for access to the
Redux types. This is also a demonstration of how to use the `#[fp(rust_plugin_module = "...")]` and
`#[fp(rust_wasmer_runtime_module = "...")]` annotations to share types between the protocol
definition and the dependent Rust crates.

## `example-deno-runtime/`

This is an example of a TypeScript runtime that can be run using Deno.

Note that in order to run this runtime, you first need to generate the bindings by running
`cargo run` inside the `example-protocol/` folder
**and then you need to build the plugin using `cargo build` inside the `example-plugin/` folder**.
After that you can run the runtime using `deno main.ts`. It will load the plugin and verify all its
functions can be called correctly.

## `example-rust-wasmer-runtime/`

This is an example of a Rust runtime that can load the example plugin.

Note that in order to run this runtime, you first need to generate the bindings by running
`cargo run` inside the `example-protocol/` folder
**and then you need to build the plugin using `cargo build` inside the `example-plugin/` folder**.
After that you can run the runtime using `cargo run`. It will load the plugin and verify all its
functions can be called correctly.
If you want to run the tests you can run `cargo test` for the wasm32-unknown-unknown architecture.
If you want to run the tests for wasm32-wasi you can run `cargo test -F wasi`.

## `redux-example/`

This is an example of how to set up Redux state management using `fp-bindgen`. Note that this crate
only contains the types; the actual reducer is implemented inside of the example plugin in the file
[`reducer.rs`](example-plugin/src/reducer.rs).

For more information about writing Redux reducers in Rust, please read the blog post:
https://fiberplane.dev/blog/writing-redux-reducers-in-rust/
