# Caduceus Server

## Testing

### Unit Tests

To run the unit tests, use the following command:

```bash
go test ./...
```

### Coverage Report

Run the following command to execute tests and generate a coverage report:

```bash
go test -coverprofile 'coverage.out' ./...
go tool cover -html 'coverage.out' -o 'coverage.html'
```

### Integration Tests

To run the integration tests, use the following command:

```bash
go test -tags=integration ./...
```
