# Contributing to fp-bindgen

We love your input! We want to make contributing to this project as easy and
transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features

## We Develop with GitHub

We use GitHub to host code, to track issues and feature requests, as well as to
accept pull requests.

## We Use [Github Flow](https://docs.github.com/en/get-started/quickstart/github-flow), So All Code Changes Happen Through Pull Requests

Pull requests are the best way to propose changes to the codebase (we use
[Github Flow](https://docs.github.com/en/get-started/quickstart/github-flow)).
We actively welcome your pull requests:

0. Before creating a pull request, please make sure it is in line with the
   projects' goals. If you have doubts about this, feel free to open an issue
   first before spending time on a pull request that may get rejected.
1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've updated APIs, update the documentation. Be mindful of breaking
   changes, please discuss these first with the maintainers.
4. Ensure the test suite passes.
5. Make sure your code is correctly formatted (see below).
6. Add an entry to the [CHANGELOG.md](CHANGELOG.md).
7. Issue that pull request!

## Contributions you make will be under the Apache 2.0 License and the MIT License

In short, when you submit code changes, your submissions are understood to be
under the same [Apache License](LICENSE-APACHE) and [MIT License](LICENSE-MIT)
that cover the project. Feel free to contact the maintainers if that's a
concern.

## Report bugs using GitHub's [issues](https://github.com/fiberplane/fp-bindgen/issues)

We use GitHub issues to track public bugs. Report a bug by
[opening a new issue](https://github.com/fiberplane/fp-bindgen/issues/new)!

## Write bug reports with detail, background, and sample code

Great bug reports tend to have:

- A quick summary and/or background
- Steps to reproduce
  - Be specific!
  - Give sample code if you can.
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you
  tried that didn't work)

The better you help us understand your problem, the better we can help you!

## Use a Consistent Coding Style

Please follow the
[Rust Coding Conventions](https://rustc-dev-guide.rust-lang.org/conventions.html).

We run `cargo fmt -- --check` to verify pull requests confirm to the expected
style and you can use this command locally to verify your changes before
pushing. In addition, we use `cargo clippy` to detect common issues with the
code itself.

## Documentation

- Do not edit `README.md` directly
- Make modifications to `fp-bindgen/src/lib.rs`
- Make sure `cargo-rdme` is installed: `cargo install cargo-rdme`
- Run `cargo-rdme` in the `fp-bindgen` folder to recompile `README.md`

## References

This document was adapted from the contribution guidelines for
[Transcriptase](https://gist.github.com/briandk/3d2e8b3ec8daf5a27a62).
