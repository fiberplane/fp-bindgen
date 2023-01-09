# CHANGELOG

## main

- Make sure `cargo check` doesn't complain about unused imports in the generated
  plugin bindings.
- Add support for `serde_json::Map` (https://github.com/fiberplane/fp-bindgen/pull/163)
- Migrate to Wasmer 3 (https://github.com/fiberplane/fp-bindgen/pull/157)

## 2.4.0

- Allow the `Runtime` to be cloned (to reuse compiled WASM modules)
- Added support for generating cargo dependencies that [inherit from
  the workspace dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace)

## 2.3.0

- Added support for configuring `default_features` in Cargo dependencies.

## 2.2.0

- Added the `serde-json-compat` feature for compatibility with the
  `serde_json::Value` type.
- Use the Wasmer Singlepass compiler on all architectures (Cranelift was used
  previously because Singlepass did not yet support arm64 chips)

## 2.1.0

- Add support for arrays of primitives (#94).
- Added the `bytes-compat` feature for compatibility with the `bytes` crate.
- Fixed an issue where embedded `Uint8Array`s that were returned to the
  TypeScript runtime from `Bytes` or `ByteBuf` types in the Rust plugin could
  end up being corrupted.

## 2.0.1

- Fix custom types in generic positions in export function signatures (#130).

## 2.0.0

### Breaking changes

- The `TypeIdent` struct has been altered. If you have custom `Serializable`
  implementations you likely need to update those.

### Other changes

- Added Deno support for the TypeScript runtime.
- Reorganized examples and improved documentation.
- Added end-to-end tests.
- Fix #105: Correctly handle passing negative integers as primitives (outside
  MessagePack) between Rust and TypeScript.
- Fix type of MessagePack-encoded 64-bit integers in TypeScript bindings.
- Fix handling synchronous responses from async plugin functions in TypeScript
  runtime.
- Fix #108: Serialization of types from the `time` crate now works between Rust
  and TypeScript.
- Implemented warnings when types that rely on custom Serde (de)serializers are
  used in contexts where their annotations cannot be used.
- Various smaller bugfixes.
- **Deprecation**: `BindingsType::TsRuntime` is now deprecated in favor of
  `BindingsType::TsRuntimeWithExtendedConfig`.
- Fix #88: Bounds are propagated correctly to generated types (with the
  exception of the compile-time only `Serializable` bound).
- Fix #88: Deal with the Unit (`()`) type.
- Use `any` type in TypeScript to represent `rmpv::Value` (#127).
- Fix issue when TypeScript types conflicted with built-in JavaScript globals
  (#128).
- Fix custom types in generic positions (#126).
