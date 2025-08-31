# Implementation Plan: RPC Client Wrapper

## CRITICAL: Context Management and Progress Tracking

### Important Notice to Implementation Agent

**You DO NOT need to complete all steps in one session.** There is NO requirement to fit everything in a single context window. This implementation can and should be done methodically, with breaks and context resets as needed.

### Progress Tracking Protocol

1. **Create Progress File**: Before starting implementation, create `implementation-plan-progress.md` in the same directory as this plan
2. **Update Progress**: After completing each step or substep, update the progress file with:
   - Timestamp
   - Step completed
   - Any important findings or deviations
   - Next step to tackle
3. **Context Reset Protocol**: If you need to clear context or feel overwhelmed:
   - Update progress file with current state
   - Note any pending work
   - When resuming, use the magic phrase: "carry on with this task implementation-plan and take a look at the progress.md to see where you got up to"
4. **Eventually Consistent**: The progress file is eventually consistent - you may have progressed further than the last entry. Always verify the actual state before continuing.

### Progress File Format Example

```markdown
# Implementation Progress Log

## 2025-08-31 10:30:00
- Starting Step 1: Proto verification
- Proto file already exists at correct location
- Running buf lint to validate

## 2025-08-31 10:35:00
- Step 1 complete: Proto validated successfully
- Starting Step 2: Rust implementation
- Creating directory structure api/src/api/rpc_client/v1/

## 2025-08-31 10:45:00
- Created service_impl.rs with RpcClientServiceImpl
- Note: Found existing commitment_level_to_config helper in account service
- Could refactor to common module later (not blocking)

## 2025-08-31 11:00:00
- Step 2.1 complete: All Rust files created
- Running cargo build to verify compilation...
```

### Quality Over Speed

**NEVER** simulate or fake implementation. If you find yourself writing comments like "simulating transaction submission" instead of actual code, STOP immediately, update the progress file, and request a context reset.

## Step 1: Define the gRPC Service and Messages

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Goal Verification**: Verify this step contributes to upgrading the proto API library to add the rpc_client service
2. **Path Consistency Check**: Verify the file path includes `/v1/` versioning as per project standards
3. **Import Validation**: Check all proto imports exist and follow the project's import pattern
4. **Package Naming**: Ensure package name follows `protosol.solana.[domain].v1` structure
5. **Go Package Option**: Verify go_package follows the established pattern

**Path Validation Checklist:**
- ✓ Contains `/v1/` directory for versioning
- ✓ Follows pattern: `lib/proto/protosol/solana/[service]/v1/service.proto`
- ✓ Not missing any directory levels

### Action: `VERIFY_EXISTS`

### File Path: `lib/proto/protosol/solana/rpc_client/v1/service.proto`

### Required Code to Implement:

The proto definition already exists with the following content:

```protobuf
syntax = "proto3";

package protosol.solana.rpc_client.v1;

import "protosol/solana/type/v1/commitment_level.proto";

option go_package = "github.com/BRBussy/protosol/lib/go/protosol/solana/rpc_client/v1;rpc_client_v1";

service Service {
  rpc GetMinimumBalanceForRentExemption(GetMinimumBalanceForRentExemptionRequest) returns (GetMinimumBalanceForRentExemptionResponse);
}

message GetMinimumBalanceForRentExemptionRequest {
    uint64 data_length = 1;
    protosol.solana.type.v1.CommitmentLevel commitment_level = 2;
}

message GetMinimumBalanceForRentExemptionResponse {
    uint64 balance = 1;
}
```

### Validation:

- Run `buf lint` from repository root to ensure the protobuf definition compiles without errors
- Run `./scripts/code-gen/generate/all.sh` to generate code stubs for Rust and Go

## Step 2: Implement the Rust gRPC Service

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Directory Structure**: Verify the directory path `api/src/api/rpc_client/v1/` needs to be created
2. **Import Path Verification**: Check that all imports match generated code locations
3. **Service Pattern Consistency**: Compare with existing services (account, system) for consistency
4. **Dependency Injection**: Ensure Arc<RpcClient> pattern matches other services
5. **Error Handling**: Verify error handling matches project patterns (Status::internal, Status::invalid_argument)
6. **Commitment Level Helper**: Check if helper function duplicates existing code

**Implementation Validation Checklist:**
- ✓ Directory structure matches other services
- ✓ Uses Arc for thread-safe sharing
- ✓ Follows Clone trait pattern
- ✓ Error messages are descriptive
- ✓ No unwrap() calls - all errors handled gracefully

### Action: `CREATE`

### File Path: `api/src/api/rpc_client/v1/service_impl.rs`

### Required Code to Implement:

```rust
use std::sync::Arc;
use tonic::{Request, Response, Status};

use protosol_api::protosol::solana::rpc_client::v1::{
    service_server::Service as RpcClientService, GetMinimumBalanceForRentExemptionRequest,
    GetMinimumBalanceForRentExemptionResponse,
};
use protosol_api::protosol::solana::r#type::v1::CommitmentLevel;

use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

/// RPC Client service implementation for wrapping Solana RPC client methods
#[derive(Clone)]
pub struct RpcClientServiceImpl {
    /// Solana RPC client for blockchain interactions
    rpc_client: Arc<RpcClient>,
}

impl RpcClientServiceImpl {
    /// Creates a new `RpcClientServiceImpl` instance with the provided RPC client
    pub const fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }
}

/// Helper function to convert proto `CommitmentLevel` to Solana `CommitmentConfig`
fn commitment_level_to_config(commitment_level: Option<i32>) -> CommitmentConfig {
    commitment_level.map_or_else(CommitmentConfig::confirmed, |level| {
        match CommitmentLevel::try_from(level) {
            Ok(CommitmentLevel::Processed) => CommitmentConfig::processed(),
            Ok(CommitmentLevel::Confirmed) => CommitmentConfig::confirmed(),
            Ok(CommitmentLevel::Finalized) => CommitmentConfig::finalized(),
            Ok(CommitmentLevel::Unspecified) | Err(_) => CommitmentConfig::confirmed(),
        }
    })
}

#[tonic::async_trait]
impl RpcClientService for RpcClientServiceImpl {
    /// Gets the minimum balance required for rent exemption for a given data length
    async fn get_minimum_balance_for_rent_exemption(
        &self,
        request: Request<GetMinimumBalanceForRentExemptionRequest>,
    ) -> Result<Response<GetMinimumBalanceForRentExemptionResponse>, Status> {
        let req = request.into_inner();

        // Convert commitment level if provided
        let commitment = commitment_level_to_config(req.commitment_level);

        // Call the underlying Solana RPC client method
        match self
            .rpc_client
            .get_minimum_balance_for_rent_exemption_with_commitment(
                req.data_length as usize,
                commitment,
            ) {
            Ok(balance) => {
                let response = GetMinimumBalanceForRentExemptionResponse { balance };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!(
                "Failed to get minimum balance for rent exemption: {e}"
            ))),
        }
    }
}
```

### Action: `CREATE`

### File Path: `api/src/api/rpc_client/v1/mod.rs`

### Required Code to Implement:

```rust
/// RPC Client service implementation
pub mod service_impl;

pub use service_impl::RpcClientServiceImpl;
```

### Action: `CREATE`

### File Path: `api/src/api/rpc_client/v1/rpc_client_v1_api.rs`

### Required Code to Implement:

```rust
use std::sync::Arc;

use super::service_impl::RpcClientServiceImpl;
use crate::service_providers::ServiceProviders;

/// RPC Client API v1 wrapper
pub struct RpcClientV1API {
    /// The RPC Client service implementation
    pub rpc_client_service: Arc<RpcClientServiceImpl>,
}

impl RpcClientV1API {
    /// Creates a new RPC Client V1 API instance
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        Self {
            rpc_client_service: Arc::new(RpcClientServiceImpl::new(
                Arc::clone(&service_providers.solana_clients.rpc_client),
            )),
        }
    }
}
```

### Action: `CREATE`

### File Path: `api/src/api/rpc_client/mod.rs`

### Required Code to Implement:

```rust
/// RPC Client v1 services
pub mod v1;

pub use v1::rpc_client_v1_api::RpcClientV1API;
```

### Action: `MODIFY`

### File Path: `api/src/api/mod.rs`

### Required Code to Implement:

Add the following line to the existing module declarations:

```rust
/// RPC Client services for direct Solana RPC access
pub mod rpc_client;
```

### Action: `MODIFY`

### File Path: `api/src/api/aggregator.rs`

### Required Code to Implement:

Modify the existing file to add the RPC client API:

```rust
use std::sync::Arc;

use super::account::v1::AccountV1API;
use super::program::Program;
use super::rpc_client::RpcClientV1API;
use super::transaction::v1::TransactionV1API;
use crate::service_providers::ServiceProviders;

/// Main API aggregator that combines all service implementations
pub struct Api {
    /// Account management API v1
    pub account_v1: Arc<AccountV1API>,
    /// Transaction lifecycle API v1
    pub transaction_v1: Arc<TransactionV1API>,
    /// Program services (system, etc.)
    pub program: Arc<Program>,
    /// RPC Client API v1
    pub rpc_client_v1: Arc<RpcClientV1API>,
}

impl Api {
    /// Creates a new API instance with the provided service providers
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            account_v1: Arc::new(AccountV1API::new(&service_providers)),
            transaction_v1: Arc::new(TransactionV1API::new(&service_providers)),
            program: Arc::new(Program::new(Arc::clone(&service_providers))),
            rpc_client_v1: Arc::new(RpcClientV1API::new(&service_providers)),
        }
    }
}
```

### Action: `MODIFY`

### File Path: `api/src/main.rs`

### Required Code to Implement:

Add the following imports at the top of the file with the other protobuf service imports (around line 20):

```rust
use protosol_api::protosol::solana::rpc_client::v1::service_server::ServiceServer as RpcClientServiceServer;
```

Then modify the server setup section (around line 160-172) to include the RPC client service:

```rust
    // Clone the services from the Arc containers
    let transaction_service = (*api.transaction_v1.transaction_service).clone();
    let account_service = (*api.account_v1.account_service).clone();
    let system_program_service = (*api.program.system.v1.system_program_service).clone();
    let rpc_client_service = (*api.rpc_client_v1.rpc_client_service).clone();

    // Clone service providers for graceful shutdown
    let service_providers_shutdown = Arc::clone(&service_providers);

    // Set up graceful shutdown
    let server = Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .add_service(AccountServiceServer::new(account_service))
        .add_service(SystemProgramServiceServer::new(system_program_service))
        .add_service(RpcClientServiceServer::new(rpc_client_service))
        .serve(addr);
```

### Integration:

The service will be automatically wired into the main gRPC server through the API aggregator pattern.

### Validation:

- Run `cargo build` in the `/api` directory to ensure compilation
- Run `cargo test` to ensure all existing tests pass

## Step 3: Implement and Pass the End-to-End Test

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Test File Naming**: Verify the test file follows Go naming conventions and project patterns
2. **Import Paths**: Ensure generated Go client imports are correct after code generation
3. **Test Structure**: Compare with existing test suites (streaming_e2e_test.go) for consistency
4. **Service Availability**: Check that rpc_client_v1 package was generated successfully
5. **Test Data**: Avoid hardcoded values that may change with Solana updates
6. **Integration Pattern**: Follow the existing test suite setup/teardown patterns

**Test Quality Checklist:**
- ✓ Uses testify suite pattern consistently
- ✓ Proper context management with cancel
- ✓ gRPC connection cleanup in TearDown
- ✓ No hardcoded lamport values - use relative comparisons
- ✓ Descriptive test names and log messages

### Action: `CREATE`

### File Path: `tests/go/rpc_client_e2e_test.go`

### Target Implementation:

```go
package apitest

import (
	"context"
	"testing"

	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	rpc_client_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/rpc_client/v1"
	type_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/type/v1"
)

// RpcClientE2ETestSuite tests the RPC Client service functionality
type RpcClientE2ETestSuite struct {
	suite.Suite
	ctx               context.Context
	cancel            context.CancelFunc
	grpcConn          *grpc.ClientConn
	rpcClientService  rpc_client_v1.ServiceClient
}

func (suite *RpcClientE2ETestSuite) SetupSuite() {
	suite.ctx, suite.cancel = context.WithCancel(context.Background())

	// Setup configuration
	grpcEndpoint := "localhost:50051"

	// Connect to gRPC server
	var err error
	suite.grpcConn, err = grpc.NewClient(
		grpcEndpoint,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	suite.Require().NoError(err, "Failed to connect to gRPC server")

	// Initialize service client
	suite.rpcClientService = rpc_client_v1.NewServiceClient(suite.grpcConn)

	suite.T().Logf("✅ RPC Client test suite setup complete")
}

func (suite *RpcClientE2ETestSuite) TearDownSuite() {
	if suite.cancel != nil {
		suite.cancel()
	}
	if suite.grpcConn != nil {
		suite.grpcConn.Close()
	}
}

// Test_01_GetMinimumBalanceForRentExemption tests getting minimum balance for rent exemption
func (suite *RpcClientE2ETestSuite) Test_01_GetMinimumBalanceForRentExemption() {
	suite.T().Log("🎯 Testing GetMinimumBalanceForRentExemption")

	// Test with different data lengths - verify relative ordering without hardcoded values
	testCases := []struct {
		name       string
		dataLength uint64
	}{
		{
			name:       "Zero data length",
			dataLength: 0,
		},
		{
			name:       "Small data length",
			dataLength: 100,
		},
		{
			name:       "Medium data length",
			dataLength: 1000,
		},
		{
			name:       "Large data length",
			dataLength: 10000,
		},
	}

	var previousBalance uint64
	var previousDataLength uint64

	for i, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			// Get minimum balance for rent exemption with specific data length
			resp, err := suite.rpcClientService.GetMinimumBalanceForRentExemption(suite.ctx, &rpc_client_v1.GetMinimumBalanceForRentExemptionRequest{
				DataLength: tc.dataLength,
			})
			suite.Require().NoError(err, "should succeed in getting minimum balance for rent exemption")
			suite.Require().NotZero(resp.Balance, "minimum balance for rent exemption should not be zero")
			
			// Verify the balance is reasonable (at least some minimum lamports)
			// Using 890_880 as a baseline since that's approximately the minimum for a 0-byte account
			// but not hardcoding exact values since they can change with Solana updates
			suite.Assert().Greater(resp.Balance, uint64(800_000), 
				"minimum balance should be at least 800,000 lamports (reasonable minimum)")
			
			// Verify that larger data requires more lamports (monotonic increase)
			if i > 0 {
				suite.Assert().Greater(resp.Balance, previousBalance,
					"data length %d should require more lamports than data length %d",
					tc.dataLength, previousDataLength)
			}
			
			suite.T().Logf("  Data length %d requires %d lamports for rent exemption", 
				tc.dataLength, resp.Balance)
			
			previousBalance = resp.Balance
			previousDataLength = tc.dataLength
		})
	}
}

// Test_02_GetMinimumBalanceWithCommitmentLevel tests with different commitment levels
func (suite *RpcClientE2ETestSuite) Test_02_GetMinimumBalanceWithCommitmentLevel() {
	suite.T().Log("🎯 Testing GetMinimumBalanceForRentExemption with different commitment levels")

	commitmentLevels := []struct {
		name  string
		level type_v1.CommitmentLevel
	}{
		{"Processed", type_v1.CommitmentLevel_COMMITMENT_LEVEL_PROCESSED},
		{"Confirmed", type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED},
		{"Finalized", type_v1.CommitmentLevel_COMMITMENT_LEVEL_FINALIZED},
	}

	for _, cl := range commitmentLevels {
		suite.T().Run(cl.name, func(t *testing.T) {
			resp, err := suite.rpcClientService.GetMinimumBalanceForRentExemption(suite.ctx, &rpc_client_v1.GetMinimumBalanceForRentExemptionRequest{
				DataLength:      100,
				CommitmentLevel: cl.level.Enum(),
			})
			suite.Require().NoError(err, "should succeed with %s commitment level", cl.name)
			suite.Require().NotZero(resp.Balance, "balance should not be zero with %s commitment", cl.name)
			
			suite.T().Logf("  Commitment %s: %d lamports required", cl.name, resp.Balance)
		})
	}
}

func TestRpcClientE2ESuite(t *testing.T) {
	suite.Run(t, new(RpcClientE2ETestSuite))
}
```

### Validation:

- Start the local Solana validator: `./scripts/tests/start-validator.sh`
- Start the backend server: `./scripts/tests/start-backend.sh`
- Run the Go E2E test suite: `cd tests/go && go test -v -run TestRpcClientE2ESuite`
- The new `GetMinimumBalanceForRentExemption` tests **MUST** pass
- All other existing E2E tests **MUST** continue to pass

## Step 4: Finalize Implementation

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE FINALIZATION:**
1. **Code Generation Verification**: Confirm `./scripts/code-gen/generate/all.sh` ran successfully
2. **Compilation Check**: Verify both Rust (`cargo build`) and Go (`go build`) compile without errors
3. **Import Organization**: Check all imports are properly organized and no unused imports remain
4. **Module Exports**: Verify all new modules are properly exported in mod.rs files
5. **Service Registration**: Confirm the service is registered in main.rs
6. **Test Coverage**: Ensure all test cases pass and cover edge cases
7. **Linting**: Run `./scripts/lint/all.sh` and fix any issues

**Final Quality Checklist:**
- ✓ No TODO comments remain
- ✓ No debug print statements (unless intentionally kept)
- ✓ All error messages are descriptive and actionable
- ✓ Documentation is complete and accurate
- ✓ Code follows project conventions exactly
- ✓ No temporary or test code remains

### Action: `REVIEW & REFINE`

### File Paths:

- `lib/proto/protosol/solana/rpc_client/v1/service.proto`
- `api/src/api/rpc_client/v1/service_impl.rs`
- `api/src/api/rpc_client/v1/mod.rs`
- `api/src/api/rpc_client/v1/rpc_client_v1_api.rs`
- `api/src/api/rpc_client/mod.rs`
- `api/src/api/mod.rs`
- `api/src/api/aggregator.rs`
- `api/src/main.rs`
- `tests/go/rpc_client_e2e_test.go`

### Code Review Instructions:

1. Verify all imports are correct and properly organized
2. Ensure consistent error handling patterns matching existing services
3. Confirm commitment level conversion matches the pattern in account service
4. Validate that the service follows the dependency injection pattern
5. Check that all new modules are properly exported

### Documentation Instructions:

All code includes comprehensive inline documentation following Rust doc comment conventions. The documentation describes:
- Purpose of each struct and function
- Parameter meanings and requirements
- Return values and error conditions
- Integration points with other services

### Validation:

1. Run `buf lint` to ensure proto compliance
2. Run `./scripts/code-gen/generate/all.sh` to regenerate all SDKs
3. Run `cargo build` in the api directory
4. Run `cargo test` to ensure no regressions
5. Run the complete E2E test suite with both validators and backend running
6. Run `./scripts/lint/all.sh` to ensure code quality

## Success Criteria

1. All four steps above are completed in order
2. The `rpc_client_e2e_test.go` test passes successfully against a running service
3. The implementation is merged without breaking any existing functionality
4. The final code is self-reviewed and documented according to the criteria in Step 4