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
- Various smaller bugfixes.
- **Deprecation**: `BindingsType::TsRuntime` is now deprecated in favor of
  `BindingsType::TsRuntimeWithExtendedConfig`.
