# Contributing
This document provides some guidance to contributing to this library.

## Submitting Bugs/Issues
Before submitting an issue, ensure that it has a simple, reproducible example.
* Refer here for [more tips](https://stackoverflow.com/help/minimal-reproducible-example).

## Features
Feature requests should try to solve some issue. They should also list possible alternatives and potential issues.

All added features should include docstrings and an accompanying doctest/unittest.

```rust
\\\ Returns hello world!
\\\ ```
\\\ assert_eq!(hello_world(), "Hello World!".to_string())
\\\ ```
fn hello_world() -> String {
    "Hello World!".to_string()
}
```

```rust
use crate::hello_world;

#[test]
fn test_hello_world() {
    assert_eq!(hello_world(), "Hello World!".to_string())
}
```

## Tests
Ensure that the entire test suite passes before submitting a PR.
```bash
# Test the saptest library.
cargo test --lib
```
To check code coverage, install [cargo tarpaulin](https://crates.io/crates/cargo-tarpaulin).
```bash
cargo install cargo-tarpaulin
cargo tarpaulin
```

### Formatting
Both `clippy` and `rustfmt` should pass.
```bash
# Format code and lint.
cargo fmt && cargo clippy
```
