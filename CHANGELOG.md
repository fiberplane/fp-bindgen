# CHANGELOG

## main

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
