# Implementation Plan: Token Program Wrapper

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
- Starting Step 1: Proto service definition
- Creating proto directory structure
- Running buf lint to validate imports

## 2025-08-31 10:35:00
- Step 1 complete: Proto service created successfully
- Starting Step 2: Rust implementation
- Creating directory structure api/src/api/program/token/v1/

## 2025-08-31 10:45:00
- Created service_impl.rs with TokenProgramServiceImpl
- Note: Found spl-token-2022 crate needs to be added to Cargo.toml
- Added Token 2022 program ID and mint length constants

## 2025-08-31 11:00:00
- Step 2.1 complete: All Rust files created
- Running cargo build to verify compilation...
```

### Quality Over Speed

**NEVER** simulate or fake implementation. If you find yourself writing comments like "simulating token mint creation" instead of actual code, STOP immediately, update the progress file, and request a context reset.

## Step 0: System Program Owner Parameter Fix (PREREQUISITE)

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Dependency Analysis**: Verify this change won't break existing system program tests
2. **Proto Compatibility**: Ensure the owner field addition follows proto versioning best practices
3. **SDK Generation**: Confirm constants files will be generated correctly for Go SDK
4. **Test Updates**: Check all existing system program tests for required updates

**Path Validation Checklist:**
- ✓ Proto modification maintains backward compatibility patterns
- ✓ Rust implementation handles owner parameter correctly
- ✓ Go constants files follow established patterns
- ✓ All existing tests updated with explicit owner parameter

### Action: `MODIFY`

### File Path: `lib/proto/protosol/solana/program/system/v1/service.proto`

### Required Code to Implement:

Add the `owner` field to the `CreateRequest` message:

```protobuf
message CreateRequest {
  string payer = 1;
  string new_account = 2;
  string owner = 3;      // <- ADD THIS FIELD
  uint64 lamports = 4;   // <- UPDATE FIELD NUMBERS
  uint64 space = 5;      // <- UPDATE FIELD NUMBERS
}
```

### Action: `MODIFY`

### File Path: `api/src/api/program/system/v1/service_impl.rs`

### Required Code to Implement:

Update the create account implementation to use the owner from the request:

```rust
impl SystemProgramService for SystemProgramServiceImpl {
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();

        // Parse public keys
        let payer = parse_pubkey(&req.payer, "payer")?;
        let new_account = parse_pubkey(&req.new_account, "new_account")?;
        let owner = parse_pubkey(&req.owner, "owner")?;  // <- USE FROM REQUEST

        // Create the instruction
        let instruction = system_instruction::create_account(
            &payer,
            &new_account,
            req.lamports,
            req.space,
            &owner,  // <- USE THE PARSED OWNER
        );

        // Convert to proto and return
        let proto_instruction = sdk_instruction_to_proto(instruction);
        Ok(Response::new(CreateResponse {
            instruction: Some(proto_instruction),
        }))
    }
}
```

### Action: `CREATE`

### File Path: `lib/go/protosol/solana/program/system/v1/consts.go`

### Required Code to Implement:

```go
package system_v1

// SystemProgramID is the public key of the System Program
const SystemProgramID = "11111111111111111111111111111112"
```

### Action: `MODIFY`

### File Path: `tests/go/composable_e2e_test.go`

Update all system program Create calls to include the owner parameter:

```go
// Example modification (repeat for all Create calls in tests)
createInstr, err := suite.systemProgramService.Create(suite.ctx, &system_v1.CreateRequest{
    Payer:      payerResp.KeyPair.PublicKey,
    NewAccount: accountResp.KeyPair.PublicKey,
    Owner:      system_v1.SystemProgramID,  // <- ADD THIS
    Lamports:   rentAmount,
    Space:      accountSpace,
})
```

### Validation:

- Run `buf lint` from repository root
- Run `./scripts/code-gen/generate/all.sh`
- Run existing integration tests: `cd tests/go && go test -v`
- Ensure all existing tests pass with owner parameter

## Step 1: Define the Token Program gRPC Service and Messages

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Goal Verification**: Verify this step contributes to creating Token 2022 program wrapper
2. **Path Consistency Check**: Verify the file path includes `/v1/` versioning as per project standards
3. **Import Validation**: Check all proto imports exist and follow the project's import pattern
4. **Package Naming**: Ensure package name follows `protosol.solana.program.token.v1` structure
5. **Go Package Option**: Verify go_package follows the established pattern

**Path Validation Checklist:**
- ✓ Contains `/v1/` directory for versioning
- ✓ Follows pattern: `lib/proto/protosol/solana/program/token/v1/service.proto`
- ✓ Not missing any directory levels

### Action: `CREATE`

### File Path: `lib/proto/protosol/solana/program/token/v1/service.proto`

### Required Code to Implement:

```protobuf
syntax = "proto3";

package protosol.solana.program.token.v1;

import "protosol/solana/transaction/v1/instruction.proto";

option go_package = "github.com/BRBussy/protosol/lib/go/protosol/solana/program/token/v1;token_v1";

// Token Program service for creating SPL Token 2022 instructions
service Service {
  // Creates an InitialiseMint instruction for Token 2022 program
  rpc InitialiseMint(InitialiseMintRequest) returns (InitialiseMintResponse);
  
  // Gets current minimum rent for a token account (mint size)
  rpc GetCurrentMinRentForTokenAccount(GetCurrentMinRentForTokenAccountRequest) returns (GetCurrentMinRentForTokenAccountResponse);
  
  // Parses mint account data into structured format
  rpc ParseMint(ParseMintRequest) returns (ParseMintResponse);
}

// Request to create InitialiseMint instruction
message InitialiseMintRequest {
  string mint_pub_key = 1;
  string mint_authority_pub_key = 2;
  string freeze_authority_pub_key = 3;
  uint32 decimals = 4;
}

// Response containing InitialiseMint instruction
message InitialiseMintResponse {
  protosol.solana.transaction.v1.SolanaInstruction instruction = 1;
}

// Request to get current rent for token account
message GetCurrentMinRentForTokenAccountRequest {
  // No parameters needed - uses fixed Mint::LEN size
}

// Response with current rent amount
message GetCurrentMinRentForTokenAccountResponse {
  uint64 lamports = 1;
}

// Request to parse mint account
message ParseMintRequest {
  string account_address = 1;
}

// Response with parsed mint data
message ParseMintResponse {
  MintInfo mint = 1;
}

// Structured mint account information
message MintInfo {
  string mint_authority_pub_key = 1;
  string freeze_authority_pub_key = 2;
  uint32 decimals = 3;
  string supply = 4;
  bool is_initialized = 5;
}
```

### Action: `CREATE`

### File Path: `lib/go/protosol/solana/program/token/v1/consts.go`

### Required Code to Implement:

```go
package token_v1

// TOKEN_2022_PROGRAM_ID is the public key of the Token 2022 Program
const TOKEN_2022_PROGRAM_ID = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"

// MINT_ACCOUNT_LEN is the size in bytes of a mint account
const MINT_ACCOUNT_LEN = 82
```

### Validation:

- Run `buf lint` from repository root to ensure the protobuf definition compiles without errors
- Run `./scripts/code-gen/generate/all.sh` to generate code stubs for Rust and Go

## Step 2: Implement the Rust Token Program Service

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Directory Structure**: Verify the directory path `api/src/api/program/token/v1/` needs to be created
2. **Dependency Management**: Ensure spl-token-2022 crate is added to Cargo.toml
3. **Import Path Verification**: Check that all imports match generated code locations
4. **Service Pattern Consistency**: Compare with system program service for consistency
5. **Error Handling**: Verify error handling matches project patterns

**Implementation Validation Checklist:**
- ✓ Directory structure matches system program pattern
- ✓ Uses same dependency injection pattern as other services
- ✓ Follows Clone trait pattern
- ✓ Error messages are descriptive
- ✓ No unwrap() calls - all errors handled gracefully

### Action: `MODIFY`

### File Path: `api/Cargo.toml`

### Required Code to Implement:

Add the spl-token-2022 dependency:

```toml
[dependencies]
# ... existing dependencies ...
spl-token-2022 = "3.0.0"
```

### Action: `CREATE`

### File Path: `api/src/api/program/token/v1/service_impl.rs`

### Required Code to Implement:

```rust
use std::sync::Arc;
use tonic::{Request, Response, Status};

use protosol_api::protosol::solana::program::token::v1::{
    service_server::Service as TokenProgramService, InitialiseMintRequest, InitialiseMintResponse,
    GetCurrentMinRentForTokenAccountRequest, GetCurrentMinRentForTokenAccountResponse,
    ParseMintRequest, ParseMintResponse, MintInfo,
};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token_2022::{
    instruction::initialize_mint2,
    state::Mint,
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::api::program::system::v1::conversion::{parse_pubkey, sdk_instruction_to_proto};

/// Token Program service implementation for Token 2022 operations
#[derive(Clone)]
pub struct TokenProgramServiceImpl {
    /// Solana RPC client for blockchain interactions
    rpc_client: Arc<RpcClient>,
}

impl TokenProgramServiceImpl {
    /// Creates a new `TokenProgramServiceImpl` instance with the provided RPC client
    pub const fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }
}

#[tonic::async_trait]
impl TokenProgramService for TokenProgramServiceImpl {
    /// Creates an InitialiseMint instruction for Token 2022 program
    async fn initialise_mint(
        &self,
        request: Request<InitialiseMintRequest>,
    ) -> Result<Response<InitialiseMintResponse>, Status> {
        let req = request.into_inner();

        // Parse public keys
        let mint_pubkey = parse_pubkey(&req.mint_pub_key, "mint_pub_key")?;
        let mint_authority = parse_pubkey(&req.mint_authority_pub_key, "mint_authority_pub_key")?;
        
        // Parse optional freeze authority
        let freeze_authority = if req.freeze_authority_pub_key.is_empty() {
            None
        } else {
            Some(parse_pubkey(&req.freeze_authority_pub_key, "freeze_authority_pub_key")?)
        };

        // Create the InitialiseMint instruction
        let instruction = initialize_mint2(
            &TOKEN_2022_PROGRAM_ID,
            &mint_pubkey,
            &mint_authority,
            freeze_authority.as_ref(),
            req.decimals as u8,
        )
        .map_err(|e| Status::invalid_argument(format!("Failed to create InitialiseMint instruction: {e}")))?;

        // Convert to proto and return
        let proto_instruction = sdk_instruction_to_proto(instruction);
        Ok(Response::new(InitialiseMintResponse {
            instruction: Some(proto_instruction),
        }))
    }

    /// Gets current minimum rent for a token account (mint size)
    async fn get_current_min_rent_for_token_account(
        &self,
        _request: Request<GetCurrentMinRentForTokenAccountRequest>,
    ) -> Result<Response<GetCurrentMinRentForTokenAccountResponse>, Status> {
        // Get minimum balance for rent exemption using Mint::LEN
        match self.rpc_client.get_minimum_balance_for_rent_exemption_with_commitment(
            Mint::LEN,
            CommitmentConfig::confirmed(),
        ) {
            Ok(lamports) => {
                let response = GetCurrentMinRentForTokenAccountResponse { lamports };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!(
                "Failed to get minimum balance for token account: {e}"
            ))),
        }
    }

    /// Parses mint account data into structured format
    async fn parse_mint(
        &self,
        request: Request<ParseMintRequest>,
    ) -> Result<Response<ParseMintResponse>, Status> {
        let req = request.into_inner();

        // Parse the account address
        let account_pubkey = parse_pubkey(&req.account_address, "account_address")?;

        // Get the account data
        let account = self.rpc_client
            .get_account_with_commitment(&account_pubkey, CommitmentConfig::confirmed())
            .map_err(|e| Status::internal(format!("Failed to get account: {e}")))?
            .value
            .ok_or_else(|| Status::not_found("Account not found"))?;

        // Verify the account is owned by the Token 2022 program
        if account.owner != TOKEN_2022_PROGRAM_ID {
            return Err(Status::invalid_argument(
                "Account is not owned by Token 2022 program"
            ));
        }

        // Unpack the mint account data
        let mint = Mint::unpack(&account.data)
            .map_err(|e| Status::invalid_argument(format!("Failed to parse mint account: {e}")))?;

        // Convert to proto format
        let mint_info = MintInfo {
            mint_authority_pub_key: mint.mint_authority
                .map(|key| key.to_string())
                .unwrap_or_default(),
            freeze_authority_pub_key: mint.freeze_authority
                .map(|key| key.to_string())
                .unwrap_or_default(),
            decimals: mint.decimals as u32,
            supply: mint.supply.to_string(),
            is_initialized: mint.is_initialized,
        };

        Ok(Response::new(ParseMintResponse {
            mint: Some(mint_info),
        }))
    }
}
```

### Action: `CREATE`

### File Path: `api/src/api/program/token/v1/mod.rs`

### Required Code to Implement:

```rust
/// Token program service implementation
pub mod service_impl;

pub use service_impl::TokenProgramServiceImpl;
```

### Action: `CREATE`

### File Path: `api/src/api/program/token/v1/token_v1_api.rs`

### Required Code to Implement:

```rust
use std::sync::Arc;

use super::service_impl::TokenProgramServiceImpl;
use crate::service_providers::ServiceProviders;

/// Token Program API v1 wrapper
pub struct TokenV1API {
    /// The Token Program service implementation
    pub token_program_service: Arc<TokenProgramServiceImpl>,
}

impl TokenV1API {
    /// Creates a new Token V1 API instance
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        Self {
            token_program_service: Arc::new(TokenProgramServiceImpl::new(
                Arc::clone(&service_providers.solana_clients.rpc_client),
            )),
        }
    }
}
```

### Action: `CREATE`

### File Path: `api/src/api/program/token/mod.rs`

### Required Code to Implement:

```rust
/// Token Program v1 services
pub mod v1;

pub use v1::token_v1_api::TokenV1API;
```

### Action: `MODIFY`

### File Path: `api/src/api/program/mod.rs`

### Required Code to Implement:

Add the token module:

```rust
/// System program services
pub mod system;
/// Token program services
pub mod token;

use std::sync::Arc;

use crate::service_providers::ServiceProviders;
use system::SystemV1API;
use token::TokenV1API;

/// Program services aggregator
pub struct Program {
    /// System program v1 services
    pub system: Arc<SystemV1API>,
    /// Token program v1 services  
    pub token: Arc<TokenV1API>,
}

impl Program {
    /// Creates a new Program instance with all program services
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            system: Arc::new(SystemV1API::new(&service_providers)),
            token: Arc::new(TokenV1API::new(&service_providers)),
        }
    }
}
```

### Action: `MODIFY`

### File Path: `api/src/main.rs`

### Required Code to Implement:

Add the following imports with the other protobuf service imports:

```rust
use protosol_api::protosol::solana::program::token::v1::service_server::ServiceServer as TokenProgramServiceServer;
```

Then modify the server setup section to include the token program service:

```rust
    // Clone the services from the Arc containers
    let transaction_service = (*api.transaction_v1.transaction_service).clone();
    let account_service = (*api.account_v1.account_service).clone();
    let system_program_service = (*api.program.system.v1.system_program_service).clone();
    let token_program_service = (*api.program.token.v1.token_program_service).clone();
    let rpc_client_service = (*api.rpc_client_v1.rpc_client_service).clone();

    // Clone service providers for graceful shutdown
    let service_providers_shutdown = Arc::clone(&service_providers);

    // Set up graceful shutdown
    let server = Server::builder()
        .add_service(TransactionServiceServer::new(transaction_service))
        .add_service(AccountServiceServer::new(account_service))
        .add_service(SystemProgramServiceServer::new(system_program_service))
        .add_service(TokenProgramServiceServer::new(token_program_service))
        .add_service(RpcClientServiceServer::new(rpc_client_service))
        .serve(addr);
```

### Validation:

- Run `cargo build` in the `/api` directory to ensure compilation
- Run `cargo test` to ensure all existing tests pass

## Step 3: Implement and Pass the Token Program E2E Test

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE IMPLEMENTATION:**
1. **Test File Structure**: Verify the test follows existing testify suite patterns
2. **Import Paths**: Ensure all generated Go client imports are correct
3. **Service Availability**: Check that token_v1 package was generated successfully  
4. **Integration Pattern**: Follow existing test patterns for setup/teardown
5. **Blockchain Validation**: Ensure complete transaction lifecycle testing

**Test Quality Checklist:**
- ✓ Uses testify suite pattern consistently with other tests
- ✓ Proper context management with cancel
- ✓ gRPC connection cleanup in TearDown
- ✓ Full transaction lifecycle (draft → compiled → signed → submitted)
- ✓ Real blockchain verification with account parsing

### Action: `CREATE`

### File Path: `tests/go/token_program_e2e_test.go`

### Required Code to Implement:

```go
package apitest

import (
	"context"
	"testing"

	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	account_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/account/v1"
	system_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/program/system/v1"
	token_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/program/token/v1"
	transaction_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/transaction/v1"
	type_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/type/v1"
)

// TokenProgramE2ETestSuite tests the Token Program service functionality
type TokenProgramE2ETestSuite struct {
	suite.Suite
	ctx                   context.Context
	cancel                context.CancelFunc
	grpcConn              *grpc.ClientConn
	accountService        account_v1.ServiceClient
	transactionService    transaction_v1.ServiceClient
	systemProgramService  system_v1.ServiceClient
	tokenProgramService   token_v1.ServiceClient
}

func (suite *TokenProgramE2ETestSuite) SetupSuite() {
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

	// Initialize service clients
	suite.accountService = account_v1.NewServiceClient(suite.grpcConn)
	suite.transactionService = transaction_v1.NewServiceClient(suite.grpcConn)
	suite.systemProgramService = system_v1.NewServiceClient(suite.grpcConn)
	suite.tokenProgramService = token_v1.NewServiceClient(suite.grpcConn)

	suite.T().Logf("✅ Token Program test suite setup complete")
}

func (suite *TokenProgramE2ETestSuite) TearDownSuite() {
	if suite.cancel != nil {
		suite.cancel()
	}
	if suite.grpcConn != nil {
		suite.grpcConn.Close()
	}
}

// Test_01_InitialiseMint tests complete mint creation and initialization
func (suite *TokenProgramE2ETestSuite) Test_01_InitialiseMint() {
	suite.T().Log("🎯 Testing Token 2022 Mint Creation and Initialization")

	// Generate payer account
	payKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate payer keypair")

	// Fund payer account
	fundResp, err := suite.accountService.FundNative(suite.ctx, &account_v1.FundNativeRequest{
		Address: payKeyResp.KeyPair.PublicKey,
		Amount:  "5000000000", // 5 SOL
	})
	suite.Require().NoError(err, "Should fund payer account")
	suite.T().Logf("  Funded payer account: %s", payKeyResp.KeyPair.PublicKey)

	// Wait for payer account to be funded
	suite.waitForAccountVisible(payKeyResp.KeyPair.PublicKey)

	// Generate mint account keypair
	mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate mint keypair")
	suite.T().Logf("  Generated mint account: %s", mintKeyResp.KeyPair.PublicKey)

	// Get current rent for token account
	rentResp, err := suite.tokenProgramService.GetCurrentMinRentForTokenAccount(suite.ctx, &token_v1.GetCurrentMinRentForTokenAccountRequest{})
	suite.Require().NoError(err, "Should get current rent amount")
	suite.T().Logf("  Rent required for mint: %d lamports", rentResp.Lamports)

	// Create mint account instruction (system program)
	createMintInstr, err := suite.systemProgramService.Create(suite.ctx, &system_v1.CreateRequest{
		Payer:      payKeyResp.KeyPair.PublicKey,
		NewAccount: mintKeyResp.KeyPair.PublicKey,
		Owner:      token_v1.TOKEN_2022_PROGRAM_ID, // Token 2022 program as owner
		Lamports:   rentResp.Lamports,
		Space:      token_v1.MINT_ACCOUNT_LEN,
	})
	suite.Require().NoError(err, "Should create mint account instruction")

	// Initialize mint instruction (token program)
	initialiseMintInstr, err := suite.tokenProgramService.InitialiseMint(suite.ctx, &token_v1.InitialiseMintRequest{
		MintPubKey:            mintKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
		FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
		Decimals:              2,
	})
	suite.Require().NoError(err, "Should create initialise mint instruction")

	// Compose atomic transaction
	atomicTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createMintInstr.Instruction,
			initialiseMintInstr.Instruction,
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}

	// Execute transaction lifecycle
	compiledTx, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: atomicTx,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Sign transaction
	signedTx, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compiledTx.Transaction,
		Signers: []*type_v1.KeyPair{
			payKeyResp.KeyPair,   // payer signature
			mintKeyResp.KeyPair,  // mint account signature
		},
	})
	suite.Require().NoError(err, "Should sign transaction")

	// Submit transaction
	submittedTx, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction: signedTx.Transaction,
	})
	suite.Require().NoError(err, "Should submit transaction")
	suite.T().Logf("  Transaction submitted: %s", submittedTx.Signature)

	// Wait for confirmation
	suite.monitorTransactionToCompletion(submittedTx.Signature)

	// Verify mint creation by parsing the account
	parsedMint, err := suite.tokenProgramService.ParseMint(suite.ctx, &token_v1.ParseMintRequest{
		AccountAddress: mintKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should parse mint account")
	suite.Require().NotNil(parsedMint.Mint, "Parsed mint should not be nil")

	// Validate mint properties
	suite.Assert().Equal(uint32(2), parsedMint.Mint.Decimals, "Mint should have 2 decimals")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.MintAuthorityPubKey, "Mint authority should match")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.FreezeAuthorityPubKey, "Freeze authority should match")
	suite.Assert().Equal("0", parsedMint.Mint.Supply, "Initial supply should be zero")
	suite.Assert().True(parsedMint.Mint.IsInitialized, "Mint should be initialized")

	suite.T().Logf("✅ Mint created and verified successfully:")
	suite.T().Logf("   Mint Address: %s", mintKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   Decimals: %d", parsedMint.Mint.Decimals)
	suite.T().Logf("   Authority: %s", parsedMint.Mint.MintAuthorityPubKey)
	suite.T().Logf("   Supply: %s", parsedMint.Mint.Supply)
}

// Test_02_GetCurrentMinRentForTokenAccount tests rent calculation
func (suite *TokenProgramE2ETestSuite) Test_02_GetCurrentMinRentForTokenAccount() {
	suite.T().Log("🎯 Testing Token Account Rent Calculation")

	// Get rent for token account
	resp, err := suite.tokenProgramService.GetCurrentMinRentForTokenAccount(suite.ctx, &token_v1.GetCurrentMinRentForTokenAccountRequest{})
	suite.Require().NoError(err, "Should get rent successfully")
	suite.Require().NotZero(resp.Lamports, "Rent should not be zero")

	// Validate reasonable rent amount (mint accounts are 82 bytes)
	suite.Assert().Greater(resp.Lamports, uint64(1_000_000), "Rent should be at least 1M lamports for mint account")
	suite.T().Logf("  Mint account rent: %d lamports", resp.Lamports)
}

// Helper function to wait for account visibility
func (suite *TokenProgramE2ETestSuite) waitForAccountVisible(address string) {
	suite.T().Logf("  Waiting for account %s to become visible...", address)
	for attempt := 1; attempt <= 10; attempt++ {
		_, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
			Address: address,
			CommitmentLevel: &type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
		})
		if err == nil {
			suite.T().Logf("  Account visible after %d attempts", attempt)
			return
		}
		if attempt < 10 {
			time.Sleep(200 * time.Millisecond)
		}
	}
	suite.T().Logf("  Account may still be processing...")
}

// Helper function to monitor transaction to completion
func (suite *TokenProgramE2ETestSuite) monitorTransactionToCompletion(signature string) {
	suite.T().Logf("  Monitoring transaction %s for completion...", signature)
	
	for attempt := 1; attempt <= 30; attempt++ {
		tx, err := suite.transactionService.GetTransaction(suite.ctx, &transaction_v1.GetTransactionRequest{
			Signature: signature,
		})
		
		if err == nil && tx.Transaction != nil {
			suite.T().Logf("  Transaction confirmed after %d attempts", attempt)
			return
		}
		
		if attempt < 30 {
			time.Sleep(1000 * time.Millisecond)
		}
	}
	
	suite.T().Logf("  Transaction monitoring completed")
}

func TestTokenProgramE2ESuite(t *testing.T) {
	suite.Run(t, new(TokenProgramE2ETestSuite))
}
```

### Validation:

- Start the local Solana validator: `./scripts/tests/start-validator.sh`
- Start the backend server: `./scripts/tests/start-backend.sh`  
- Run the Token Program E2E test suite: `cd tests/go && go test -v -run TestTokenProgramE2ESuite`
- The new Token Program tests **MUST** pass
- All other existing E2E tests **MUST** continue to pass

## Step 4: Finalize Implementation

### Pre-Review Requirements

**CRITICAL SELF-REVIEW BEFORE FINALIZATION:**
1. **Code Generation Verification**: Confirm `./scripts/code-gen/generate/all.sh` ran successfully
2. **Compilation Check**: Verify both Rust (`cargo build`) and Go (`go build`) compile without errors
3. **Import Organization**: Check all imports are properly organized and no unused imports remain
4. **Module Exports**: Verify all new modules are properly exported in mod.rs files
5. **Service Registration**: Confirm the service is registered in main.rs
6. **Test Coverage**: Ensure all test cases pass and cover the complete mint lifecycle
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

- `lib/proto/protosol/solana/program/token/v1/service.proto`
- `lib/go/protosol/solana/program/token/v1/consts.go`
- `api/src/api/program/token/v1/service_impl.rs`
- `api/src/api/program/token/v1/mod.rs`
- `api/src/api/program/token/v1/token_v1_api.rs`
- `api/src/api/program/token/mod.rs`
- `api/src/api/program/mod.rs`
- `api/src/main.rs`
- `tests/go/token_program_e2e_test.go`

### Code Review Instructions:

1. Verify all imports are correct and properly organized
2. Ensure consistent error handling patterns matching existing services
3. Confirm Token 2022 program integration follows best practices
4. Validate that the service follows the dependency injection pattern
5. Check that all new modules are properly exported
6. Ensure constants are correctly defined and accessible

### Documentation Instructions:

All code includes comprehensive inline documentation following Rust doc comment conventions. The documentation describes:
- Purpose of each struct and function
- Parameter meanings and requirements
- Return values and error conditions
- Integration points with Token 2022 program
- Blockchain interaction patterns

### Validation:

1. Run `buf lint` to ensure proto compliance
2. Run `./scripts/code-gen/generate/all.sh` to regenerate all SDKs
3. Run `cargo build` in the api directory
4. Run `cargo test` to ensure no regressions
5. Run the complete E2E test suite with both validators and backend running
6. Run `./scripts/lint/all.sh` to ensure code quality

## Success Criteria

1. All four steps above are completed in order
2. The prerequisite system program owner parameter fix is implemented
3. The `token_program_e2e_test.go` test passes successfully against a running service
4. Token 2022 mints can be created, initialized, and parsed via gRPC
5. Rent calculation works correctly for mint accounts
6. The implementation is merged without breaking any existing functionality
7. The final code is self-reviewed and documented according to the criteria in Step 4