# Solana E2E Integration Tests

This directory contains consolidated end-to-end integration tests for Solana gRPC API services. The tests are organized in a single comprehensive suite that covers all functionality and will be updated to use instruction composition once the new architecture is implemented.

## Directory Structure

```
project/solana/cmd/api-test/
├── e2e_test.go                 # Consolidated E2E test suite  
├── local-config.json           # Test configuration
├── go.mod                      # Go module definition
├── go.sum                      # Go module checksums
└── go/
    ├── README.md               # This file
    └── config/                 # Shared test configuration
        └── config.go
```

## Architecture Note

**Current State:** Tests use the current transaction-based architecture where each service creates complete transactions.

**Future State:** These tests will be updated to use the new instruction composition architecture where services return `SolanaInstruction` objects that can be composed into atomic transactions.

## Prerequisites

### Required Setup
1. **Go 1.21+** installed
2. **Solana CLI tools** installed (`solana-test-validator`, `solana-keygen`, etc.)
3. **Solana gRPC backend** available (Rust service at `project/solana/cmd/api`)
4. **Local Solana validator** for realistic testing

### Dependencies
Go integration tests use the generated client libraries:
```go
import (
    account_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/account/v1"
    system_program_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/program/system/v1"
    transaction_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/transaction/v1"
)
```

## Running Tests

### Option 1: Manual Setup
```bash
# 1. Start local Solana validator
./project/solana/scripts/start-validator.sh

# 2. Start Solana gRPC backend
export SOLANA_RPC_URL="http://localhost:8899"
./project/solana/scripts/start-backend.sh

# 3. Run E2E integration tests
cd project/solana/cmd/api-test
RUN_INTEGRATION_TESTS=1 go test -v

# 4. Clean up
./project/solana/scripts/stop-backend.sh
pkill solana-test-validator
```

### Option 2: Automated Integration Test (Recommended)
```bash
# Complete end-to-end test with automatic setup/cleanup
./project/solana/scripts/comprehensive-test.sh
```

## Test Configuration

### Client Configuration
Tests create clients using this pattern:
```go
service, err := account_v1.NewServiceService(
    api.WithURL("localhost:50051"),
    api.WithInsecure(),
    api.WithTimeout(60 * time.Second),
)
```

### Environment Variables
Tests automatically configure the backend via:
- **SOLANA_RPC_URL**: Set to `http://localhost:8899` for local validator
- **RUN_INTEGRATION_TESTS**: Set to `1` to enable integration tests

### Local Configuration
The `local-config.json` file contains:
```json
{
    "solana_rpc_url": "http://localhost:8899",
    "backend_grpc_endpoint": "localhost",
    "backend_grpc_port": 50051,
    "test_account_address": "5MvYgrb6DDznpeqejPzkJSxj7cBCu4UjTRVb1saMsGPr"
}
```

## Test Coverage

### Current Test Suite (e2e_test.go)

1. **Account Service Tests**
   - `Test_01_AccountService_GetAccount` - Account retrieval with various scenarios
   - `Test_02_AccountService_GenerateNewKeyPair` - Keypair generation

2. **System Program Service Tests**
   - `Test_03_SystemProgram_CreateAccountInstruction` - Account creation transactions
   - `Test_04_SystemProgram_TransferInstruction` - Transfer transactions

3. **Transaction Service Tests**
   - `Test_05_TransactionService_SignTransaction` - Single signature signing
   - `Test_06_TransactionService_SignWithMultipleKeys` - Multi-signature signing

4. **Complete Flow Tests**
   - `Test_07_CompleteFlow_CreateSignSubmit` - Full transaction pipeline
   - `Test_08_CompleteFlow_AccountCreationWithFunding` - Complex multi-step operations

5. **System Tests**
   - `Test_09_ErrorHandling` - Error scenarios and validation
   - `Test_10_PerformanceBaseline` - Performance metrics for comparison
   - `Test_11_FutureComposition` - Documentation of future architecture benefits

### API Service Coverage

#### Account Service (`account/v1`)
- ✅ `GetAccount` - Retrieve account by address
- ✅ `GenerateNewKeyPair` - Generate new Solana keypairs
- 🚧 `FundNative` - Fund accounts (available but not tested yet)

#### System Program Service (`program/system/v1`)
- ✅ `Create` - Create account transactions
- ✅ `Transfer` - Transfer SOL transactions
- 🚧 Additional system operations (11 more to be implemented)

#### Transaction Service (`transaction/v1`)
- ✅ `SignTransaction` - Sign transactions with private keys
- 🚧 `GetTransaction` - Transaction retrieval (basic testing)
- 🚧 `SubmitTransaction` - Transaction submission (basic testing)

## Future Architecture Integration

### Current Limitations
- Each operation creates a complete transaction
- Cannot combine multiple operations atomically
- Manual account deduplication required
- Complex multi-step operations require multiple transactions

### Planned Improvements
- Services will return `SolanaInstruction` objects
- Compose multiple instructions into single transactions
- Automatic account deduplication and privilege escalation
- Atomic execution of complex multi-step operations
- Support for all 13 system program operations

### Migration Strategy
When the instruction-based architecture is implemented:
1. Update service calls to return instructions instead of transactions
2. Add instruction composition tests
3. Update existing tests to use composed transactions
4. Add new tests for advanced composition scenarios

## Running Specific Tests

```bash
# Run all E2E tests
RUN_INTEGRATION_TESTS=1 go test -v

# Run specific test
RUN_INTEGRATION_TESTS=1 go test -v -run TestE2ESuite/Test_01_AccountService_GetAccount

# Run with timeout
RUN_INTEGRATION_TESTS=1 go test -v -timeout 10m
```

## Troubleshooting

### Common Issues

**Backend Connection Errors:**
```bash
# Verify backend is running and connected
./project/solana/scripts/start-backend.sh
curl -v localhost:50051  # Should connect
```

**Validator Issues:**
```bash
# Check validator status
solana cluster-version -u http://localhost:8899

# Restart if needed
pkill solana-test-validator
./project/solana/scripts/start-validator.sh
```

**Test Account Issues:**
```bash
# Check test account balance
solana balance 5MvYgrb6DDznpeqejPzkJSxj7cBCu4UjTRVb1saMsGPr -u http://localhost:8899

# Fund if needed
solana airdrop 10 5MvYgrb6DDznpeqejPzkJSxj7cBCu4UjTRVb1saMsGPr -u http://localhost:8899
```

### Debug Mode
```bash
# Enable verbose logging
export RUST_LOG=debug
./project/solana/scripts/start-backend.sh

# Run tests with extra logging
RUN_INTEGRATION_TESTS=1 go test -v -count=1
```

For more details, see the main [implementation plan](../docs/implementation-plan.md) and [transaction composition documentation](../docs/transaction-composition-architecture.md).