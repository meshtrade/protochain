# Solana E2E Integration Tests

Go integration tests that validate the Protochain Solana gRPC API using the generated Go SDK against a live Solana validator.

## Running Tests

### Local Development (recommended)

```bash
# Start only the surfpool validator via Docker
docker compose up surfpool -d

# Run the Rust backend locally (fast rebuilds, instant restarts)
cargo run -p protochain-solana-api

# Run tests
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v

docker compose down
```

> **Tip:** Always develop with `cargo run` locally. Building the API in Docker is slow and unnecessary for day-to-day work.

### Full Stack Docker (testing the published image)

```bash
# Pulls the pre-built image from GHCR — does NOT build locally
docker compose up -d
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v
docker compose down
```

### Test controls

```bash
# Auto-detect running services (skips if not available)
cd tests/go && go test -v

# Force integration tests
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v

# Run specific test suite/test
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite/Test_02"

# Explicitly skip
cd tests/go && RUN_INTEGRATION_TESTS=0 go test -v
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
RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite/Test_05"

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

### Services not running
```bash
# Start the full stack
docker compose up -d

# Check container health
docker compose ps
```

### Backend connection errors
```bash
# Check surfpool is healthy
curl -s http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# Check API logs
docker compose logs protochain-solana-api
```
