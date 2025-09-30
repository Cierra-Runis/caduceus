# Caduceus Server

## Overview

Caduceus server application is built with Rust. It leverages the [Actix-web](https://actix.rs) framework for handling HTTP requests and integrates with MongoDB for data storage.

## Development

### Running the Server

First, copy the `./config/test.yaml` file to `./config/dev.yaml`:

```bash
cp ./config/test.yaml ./config/dev.yaml
```

Then, fill in the necessary configuration values in `./config/dev.yaml`.

To start the development server, run the following command:

```bash
cargo run
```

For hot-reloading during development, download and install `cargo-watch`:

```bash
cargo install cargo-watch
```

Then, start the server with hot-reloading enabled:

```bash
cargo watch -x 'run'
```

## Testing

To run the tests, use the following command:

```bash
cargo test
```

### Coverage Report

First, install `cargo-llvm-cov` following the instructions at [cargo-llvm-cov GitHub repository](https://github.com/taiki-e/cargo-llvm-cov?tab=readme-ov-file#from-source).

Switch to the nightly Rust toolchain to exclude test code from the coverage report:

```bash
rustup override set nightly
```

To generate a coverage report, use the following command:

```bash
cargo llvm-cov --all-features --workspace --html
```

After running the above command, open the generated `target/llvm-cov/html/index.html` file in your web browser to view the coverage report.

You could install VSCode extension [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) to visualize the coverage report directly in your code editor.

Then, run the following command to generate a coverage report in lcov format:

```bash
cargo llvm-cov --lcov --output-path ./target/lcov.info
```

Finally, use the Coverage Gutters extension to display the coverage information in your source files.
