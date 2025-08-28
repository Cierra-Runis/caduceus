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

To generate a coverage report, use the following command:

```bash
cargo tarpaulin --out Html
```
