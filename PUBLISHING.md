# Publishing a new version

If you need to publish a new version of `fp-bindgen`, please follow the next
steps:

0. Make sure you are logged in with your `Crates.io` account to the Cargo CLI:
   https://doc.rust-lang.org/cargo/reference/publishing.html#before-your-first-publish
1. Update all references to the version number in the project. At the time of
   writing, these are the files containing the version number:
   * `fp-bindgen/Cargo.toml`
   * `fp-bindgen-support/Cargo.toml`
   * `macros/Cargo.toml`
   * `examples/example-protocol/src/asserts/rust_plugin_test/expected_Cargo.toml`
     (only the version of the `fp-bindgen-support` dependency needs to be
     bumped, the version of the `example-bindings` itself can stay the same.)
2. Add entries to the [CHANGELOG](./CHANGELOG.md) detailing the updates since the last version (run `git log` to look at all of the unpublished commits)
3. Commit all the version changes and create a PR for it. Please wait for an
   approval before continuing to publish the new version.
4. Run `cargo publish` in the three crates, in the following order:
   1. `macros`
   2. `fp-bindgen-support`
   3. `fp-bindgen`
