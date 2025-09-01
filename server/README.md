# Caduceus Server

## Development

### Running the Server

1. First, copy the `./config/test.yaml` file to `./config/dev.yaml`:

   ```bash
   cp ./config/test.yaml ./config/dev.yaml
   ```

   Then, fill in the necessary configuration values in `./config/dev.yaml`.

2. To start the development server, run the following command:

   ```bash
   cargo run
   ```

3. For hot-reloading during development, download and install `cargo-watch`:

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

1. First, install `cargo-llvm-cov` following the instructions at [cargo-llvm-cov GitHub repository](https://github.com/taiki-e/cargo-llvm-cov?tab=readme-ov-file#from-source).

2. Switch to the nightly Rust toolchain to exclude test code from the coverage report:

   ```bash
   rustup override set nightly
   ```

3. To generate a coverage report, use the following command:

   ```bash
   cargo llvm-cov --all-features --workspace --html
   ```

4. After running the above command, open the generated `target/llvm-cov/html/index.html` file in your web browser to view the coverage report.
