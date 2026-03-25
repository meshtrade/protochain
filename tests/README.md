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

### Run a specific test

```bash
cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite/Test_02"
```

See [tests/go/README.md](go/README.md) for detailed documentation.
