# Development

This document outlines the process for compiling this crate's source code on your local machine.

## Prerequisites

Ensure you have the following installed:

- The latest stable version of **Rust**
- [`cargo-nextest`](https://crates.io/crates/cargo-nextest) for running tests
- [`Clippy`](https://crates.io/crates/clippy) for linting

## Code Style

We follow the default Rust formatting style enforced by `rustfmt`. To format your code, run:

```sh
cargo fmt
```

Additionally, we use **Clippy** for linting Rust code. You can check for linting issues by running:

```sh
cargo clippy
```

Please ensure your code is formatted and free of Clippy warnings before submitting any changes.

## Testing

We use [`cargo-nextest`](https://crates.io/crates/cargo-nextest) to run our test suite.

### Running Tests

To test the latest version of the schema, use:

```sh
cargo nextest run
```

Alternatively, you can use the alias command:

```sh
cargo run_test
```

### Running Tests for All Schema Versions

To test against all schema versions, execute the shell script:

```sh
./scripts/run_test.sh
```

This ensures all tests pass across different schema versions.
