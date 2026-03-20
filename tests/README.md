# Integration Tests

End-to-end integration tests for the Protochain Solana gRPC API services.

## Directory Structure

```
tests/
├── README.md                      # This file
└── go/                            # Go integration test module
    ├── README.md                  # Detailed test documentation
    ├── streaming_e2e_test.go      # Transaction streaming tests
    ├── token_program_e2e_test.go  # Token program tests
    ├── rpc_client_e2e_test.go     # RPC client tests
    ├── go.mod
    ├── go.sum
    └── config/
        └── config.go              # Shared configuration utilities
```

## Quick Start

### Prerequisites
- Go 1.21+
- Running Solana validator
- Running Protochain gRPC backend

### Running Tests

```bash
# Option 1: Auto-detect running services
cd tests/go && go test -v

# Option 2: Force integration tests
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v

# Run a specific test suite
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v -run "TestComposableE2ESuite/Test_05"
```

### Starting Services

```bash
# Terminal 1: Start validator (Docker or native)
./scripts/tests/start-validator-docker.sh
# or: ./scripts/tests/start-validator.sh

# Terminal 2: Start backend
cargo run -p protochain-solana-api
# or: ./scripts/tests/start-backend.sh

# Terminal 3: Run tests
cd tests/go && go test -v
```

See [tests/go/README.md](go/README.md) for detailed documentation.
