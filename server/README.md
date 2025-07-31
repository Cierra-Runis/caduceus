```bash
# ./server/
go test -cover ./... -coverprofile="coverage.out" && go tool cover -html="coverage.out" -o coverage.html
```
