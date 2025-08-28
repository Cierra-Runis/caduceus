# Caduceus Server

## Development

### Writing Code

Please install the [Golangci-lint](https://golangci-lint.run/docs/welcome/install/#local-installation) tool to ensure code quality. You can run the linter with:

```bash
golangci-lint run
```

And [integrate with your IDE](https://golangci-lint.run/docs/welcome/integrations) for real-time feedback.

> ![Note]
> If you're using VSCode, the recommended settings are already included in `.vscode/settings.json`.
> In IDE mode, some linters will be disable since `--fast-only` is enabled. To run all linters, use the command line.

### Running the Server

To start the development server, run the following command:

```bash
go run .
```

For hot-reloading during development, download and install `air` by

```bash
go install github.com/air-verse/air@latest
```

Then, start the server with hot-reloading using:

```bash
air
```

## Testing

### Unit Tests

To run the unit tests, use the following command:

```bash
go test ./src/...
```

### Coverage Report

Run the following command to execute tests and generate a coverage report:

```bash
go test -coverprofile 'coverage.out' ./src/...
go tool cover -html 'coverage.out' -o 'coverage.html'
```

### Integration Tests

To run the integration tests, use the following command:

```bash
go test -tags=integration ./src/...
```
