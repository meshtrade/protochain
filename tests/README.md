# Solana API Integration Tests

## Overview

This directory contains consolidated end-to-end integration tests for the Solana gRPC API services. The tests are organized as a single comprehensive suite that validates the entire transaction pipeline from account management through transaction signing and submission.

## Current Status

✅ **Basic Test Infrastructure** - Working  
📌 **Full E2E Test Suite** - Available but requires protobuf generation  

## Quick Start

### Run Basic Tests
```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v
```

Or from the project root:
```bash
RUN_INTEGRATION_TESTS=1 go test ./tests/go -v
```

### Enable Full E2E Test Suite

1. **Generate protobuf code:**
   ```bash
   ./dev/generate/all.sh
   ```

2. **Uncomment the full test suite in `e2e_test.go`:**
   - Remove the opening `/*` comment marker at the beginning
   - Remove the closing `*/` comment marker at the end

3. **Update dependencies:**
   ```bash
   go mod tidy
   ```

4. **Run full test suite:**
   ```bash
   cd tests/go
   RUN_INTEGRATION_TESTS=1 go test -v
   ```

## Test Coverage

### Current (Basic)
- `TestSimpleConnection` - Basic setup and configuration validation

### Full E2E Suite (After protobuf generation)

**Account Service Tests:**
- Account retrieval with various scenarios
- Keypair generation

**System Program Service Tests:**
- Account creation transactions  
- Transfer transactions

**Transaction Service Tests:**
- Single signature signing
- Multi-signature signing

**Complete Flow Tests:**
- Full transaction pipeline (Create → Sign → Ready)
- Account creation with funding
- Error handling scenarios
- Performance baselines
- Future architecture documentation

## Prerequisites

### Development Environment
- Go 1.21+
- Solana CLI tools
- Local Solana validator
- Solana gRPC backend service

### Running Full Integration Tests

1. **Start local Solana validator:**
   ```bash
   ./project/solana/scripts/start-validator.sh
   ```

2. **Start Solana gRPC backend:**
   ```bash
   export SOLANA_RPC_URL="http://localhost:8899"
   ./project/solana/scripts/start-backend.sh
   ```

3. **Run tests:**
   ```bash
   cd tests/go
   RUN_INTEGRATION_TESTS=1 go test -v -timeout 10m
   ```

4. **Cleanup:**
   ```bash
   ./project/solana/scripts/stop-backend.sh
   pkill solana-test-validator
   ```

## Configuration

The tests use `local-config.json` for configuration:

```json
{
    "solana_rpc_url": "http://localhost:8899",
    "backend_grpc_endpoint": "localhost", 
    "backend_grpc_port": 50051,
    "test_account_address": "5MvYgrb6DDznpeqejPzkJSxj7cBCu4UjTRVb1saMsGPr"
}
```

## Architecture Notes

**Current State:** Tests use the transaction-based architecture where each service creates complete transactions.

**Future State:** These tests will be updated to use the new instruction composition architecture where services return `SolanaInstruction` objects that can be composed into atomic transactions.

The test suite includes placeholders and documentation for the future architecture, making the transition straightforward when the instruction-based system is implemented.

## Directory Structure

```
tests/
├── README.md                   # This file
├── local-config.json           # Test configuration
└── go/                         # Go test module
    ├── README.md               # Detailed documentation
    ├── simple_test.go          # Basic test that always works
    ├── e2e_test.go             # Comprehensive E2E suite (commented until protobuf gen)
    ├── go.mod                  # Go module definition
    ├── go.sum                  # Go module checksums
    └── config/
        └── config.go           # Shared configuration utilities
```

## Troubleshooting

### "No generated protobuf code"
```bash
# Generate the required protobuf code
./dev/generate/all.sh
```

### "Backend connection failed"  
```bash
# Ensure backend is running and connected to local validator
./project/solana/scripts/start-validator.sh
./project/solana/scripts/start-backend.sh
```