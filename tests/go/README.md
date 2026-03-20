# Solana E2E Integration Tests

Go integration tests that validate the Protochain Solana gRPC API using the generated Go SDK against a live Solana validator.

## Running Tests

```bash
# Auto-detect running services (skips if not available)
cd tests/go && go test -v

# Force integration tests
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v

# Run specific test suite/test
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v -run "TestComposableE2ESuite/Test_05"

# Explicitly skip
cd tests/go && RUN_INTEGRATION_TESTS=0 go test -v
```

## Prerequisites

1. **Start Solana validator:**
   ```bash
   ./scripts/tests/start-validator-docker.sh
   # or: ./scripts/tests/start-validator.sh
   ```

2. **Start gRPC backend:**
   ```bash
   cargo run -p protochain-solana-api
   # or: ./scripts/tests/start-backend.sh
   ```

3. **Run tests:**
   ```bash
   cd tests/go
   RUN_INTEGRATION_TESTS=1 go test -v -timeout 10m
   ```

## Configuration

Tests use `local-config.json`:
```json
{
    "solana_rpc_url": "http://localhost:8899",
    "backend_grpc_endpoint": "localhost",
    "backend_grpc_port": 50051
}
```

## Test Files

| File | Coverage |
|------|----------|
| `streaming_e2e_test.go` | Transaction streaming, composable transaction lifecycle |
| `token_program_e2e_test.go` | SPL Token & Token-2022 operations, mint creation |
| `rpc_client_e2e_test.go` | RPC client service, rent calculations |

## Test Patterns

Tests use the **testify suite** pattern. Individual tests must be run with the `SuiteName/TestName` format:

```bash
# Correct
RUN_INTEGRATION_TESTS=1 go test -v -run "TestComposableE2ESuite/Test_05"

# Wrong (won't match testify suite tests)
RUN_INTEGRATION_TESTS=1 go test -v -run "Test_05"
```

### Dependencies
Tests use the generated Go client libraries:
```go
import (
    account_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/account/v1"
    system_program_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/program/system/v1"
    transaction_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/transaction/v1"
)
```

## Troubleshooting

### Backend connection errors
```bash
# Verify backend is running
lsof -i :50051

# Restart backend
./scripts/tests/start-backend.sh
```

### Validator issues
```bash
# Check validator health
curl -s http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# Restart validator
./scripts/tests/stop-validator-docker.sh
./scripts/tests/start-validator-docker.sh
```
