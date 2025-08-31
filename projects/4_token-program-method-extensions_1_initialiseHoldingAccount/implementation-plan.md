# **Implementation Plan: Add InitializeAccount Method to Token Program Service**

---
## CRITICAL: Context Management, Progress and Quality
Some critical information for the implementation agent.

### Important Notice to Implementation Agent on Step Completion

**You DO NOT need to complete all steps in one session.** There is NO requirement to fit everything in a single context window. This implementation can and should be done methodically, with breaks and context resets as needed.

### On Progress Tracking

1. **Create Progress File**: Before starting implementation, create/update your progress task tracking file .claude-task in the same directory as this plan
2. **Update Progress**: After completing each step or substep, update the progress file with:
   - Timestamp
   - Step completed
   - Any important findings or deviations
   - Next step to tackle
3. **Context Reset Protocol**: If you need to clear context or feel overwhelmed:
   - Update progress file with current state
   - Note any pending work
   - When resuming, use the magic phrase: "carry on with this task implementation-plan and take a look at the progress.md to see where you got up to"
4. **Eventually Consistent**: The progress file is eventually consistent - you may have progressed further than the last entry. Always verify the actual state before continuing (only relevant on RESTARTING TASK).

### On Quality Over Speed

**NEVER** simulate or fake implementation. If you find yourself writing comments like "simulating token mint creation" instead of actual code, STOP immediately, update the progress file, and request a context reset.
---

## **Overview**

This implementation plan adds the `InitializeAccount` method (named `InitialiseHoldingAccount`) to the ProtoSol token program service. The goal is to enable creation and initialization of token holding accounts that can hold tokens from a specific mint.

**Key Additions:**
1. `InitialiseHoldingAccount` - Creates InitializeAccount instruction for token accounts
2. `GetCurrentMinRentForHoldingAccount` - Gets rent calculation for token holding accounts  
3. `HOLDING_ACCOUNT_LEN` constant - Token account size constant for Go SDK
4. Enhanced e2e test - Full mint + holding account creation flow

## **Technical Context**

**From Research Analysis:**
- **InitializeAccount Function**: `initialize_account(token_program_id, account_pubkey, mint_pubkey, owner_pubkey)`
- **Account Structure**: Requires 4 accounts: account (writable), mint (readonly), owner (readonly), rent sysvar (readonly)
- **Account Size**: `Account::LEN = 165` bytes for token holding accounts
- **Response Pattern**: Token program methods return `Response` containing `SolanaInstruction`

## **Implementation Steps**

### **Phase 1: Proto Service Definition Updates**

#### **Step 1.1: Update Token Program Proto Service**
**File**: `lib/proto/protosol/solana/program/token/v1/service.proto`

**Pre-review**: This step updates the core service definition - the source of truth for all generated code.

**Actions:**
1. Add `InitialiseHoldingAccount` RPC method to the Service
2. Add `GetCurrentMinRentForHoldingAccount` RPC method to the Service
3. Define `InitialiseHoldingAccountRequest` message
4. Define `InitialiseHoldingAccountResponse` message  
5. Define `GetCurrentMinRentForHoldingAccountRequest` message
6. Define `GetCurrentMinRentForHoldingAccountResponse` message

**Expected Changes:**
```protobuf
service Service {
  // ... existing methods ...
  
  // Creates an InitialiseHoldingAccount instruction for Token 2022 program
  rpc InitialiseHoldingAccount(InitialiseHoldingAccountRequest) returns (InitialiseHoldingAccountResponse);
  
  // Gets current minimum rent for a token holding account
  rpc GetCurrentMinRentForHoldingAccount(GetCurrentMinRentForHoldingAccountRequest) returns (GetCurrentMinRentForHoldingAccountResponse);
}

// Request to create InitialiseHoldingAccount instruction
message InitialiseHoldingAccountRequest {
  string account_pub_key = 1;
  string mint_pub_key = 2;
  string owner_pub_key = 3;
}

// Response containing InitialiseHoldingAccount instruction
message InitialiseHoldingAccountResponse {
  protosol.solana.transaction.v1.SolanaInstruction instruction = 1;
}

// Request to get current rent for holding account
message GetCurrentMinRentForHoldingAccountRequest {
  // No parameters needed - uses fixed Account::LEN size
}

// Response with current rent amount for holding account
message GetCurrentMinRentForHoldingAccountResponse {
  uint64 lamports = 1;
}
```

**Validation:**
- Ensure message field numbers don't conflict with existing messages
- Follow existing naming patterns (`pub_key` suffix, `lamports` field name)
- Request/Response pair for each RPC method

#### **Step 1.2: Validate Proto Syntax**
**Command**: `buf lint lib/proto` (from project root)

**Actions:**
1. Run buf lint to validate proto syntax
2. Fix any lint errors or warnings
3. Ensure all imports and field types are correct

**Success Criteria:**
- No lint errors or warnings
- All message references resolve correctly

### **Phase 2: Code Generation**

#### **Step 2.1: Generate All Language Bindings**
**Command**: `./scripts/code-gen/generate/all.sh`

**Pre-review**: This regenerates Go, Rust, and TypeScript code from proto definitions. Generated files should not be manually edited.

**Actions:**
1. Run the code generation script from project root
2. Verify all target directories have updated files
3. Check that new service methods appear in generated code

**Expected Generated Files:**
- `lib/rust/src/protosol.solana.program.token.v1.rs` (updated)
- `lib/go/protosol/solana/program/token/v1/service.pb.go` (updated)
- `lib/go/protosol/solana/program/token/v1/service_grpc.pb.go` (updated)
- `lib/go/protosol/solana/program/token/v1/service_*.passivgo.go` (updated via custom plugin)
- `lib/ts/src/` files (updated)

**Success Criteria:**
- Code generation completes without errors
- New RPC methods visible in generated Rust traits and Go interfaces

#### **Step 2.2: Add Holding Account Constants to Go SDK**
**File**: `lib/go/protosol/solana/program/token/v1/consts.go`

**Actions:**
1. Add `HOLDING_ACCOUNT_LEN` constant with value `165` (from `Account::LEN`)
2. Follow existing constant pattern and documentation style

**Expected Addition:**
```go
// HOLDING_ACCOUNT_LEN is the size in bytes of a token holding account
const HOLDING_ACCOUNT_LEN = 165
```

**Validation:**
- Constant follows naming convention
- Value matches `Account::LEN` from Token 2022 program
- Documentation comment provided

### **Phase 3: Rust Backend Implementation**

#### **Step 3.1: Import Required Dependencies**
**File**: `api/src/api/program/token/v1/service_impl.rs`

**Actions:**
1. Add import for `spl_token_2022::instruction::initialize_account`
2. Add import for `spl_token_2022::state::Account` if not already present
3. Verify all needed imports are available

**Expected Additions:**
```rust
use spl_token_2022::{instruction::{initialize_mint2, initialize_account}, state::{Mint, Account}, ID as TOKEN_2022_PROGRAM_ID};
```

#### **Step 3.2: Implement InitialiseHoldingAccount Method**
**File**: `api/src/api/program/token/v1/service_impl.rs`

**Pre-review**: This method creates the Solana Token 2022 InitializeAccount instruction. Reference the existing `initialise_mint` method for patterns.

**Actions:**
1. Implement the `initialise_holding_account` async method
2. Parse and validate all pubkey parameters
3. Create the instruction using `initialize_account` function
4. Convert to proto format using `sdk_instruction_to_proto`
5. Return wrapped in `InitialiseHoldingAccountResponse`

**Expected Implementation:**
```rust
/// Creates an `InitialiseHoldingAccount` instruction for Token 2022 program
async fn initialise_holding_account(
    &self,
    request: Request<InitialiseHoldingAccountRequest>,
) -> Result<Response<InitialiseHoldingAccountResponse>, Status> {
    let req = request.into_inner();

    // Parse public keys
    let account_pubkey = Pubkey::from_str(&req.account_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid account_pub_key: {e}")))?;
    let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
    let owner_pubkey = Pubkey::from_str(&req.owner_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid owner_pub_key: {e}")))?;

    // Create the InitializeAccount instruction
    let instruction = initialize_account(
        &TOKEN_2022_PROGRAM_ID,
        &account_pubkey,
        &mint_pubkey,
        &owner_pubkey,
    )
    .map_err(|e| {
        Status::invalid_argument(format!("Failed to create InitialiseHoldingAccount instruction: {e}"))
    })?;

    // Convert to proto and return
    let proto_instruction = sdk_instruction_to_proto(instruction);
    Ok(Response::new(InitialiseHoldingAccountResponse {
        instruction: Some(proto_instruction),
    }))
}
```

**Validation:**
- All pubkey parsing includes error handling
- Uses `TOKEN_2022_PROGRAM_ID` constant
- Instruction creation errors are handled
- Response follows existing patterns

#### **Step 3.3: Implement GetCurrentMinRentForHoldingAccount Method**
**File**: `api/src/api/program/token/v1/service_impl.rs`

**Pre-review**: This method calculates rent for token holding accounts. Reference the existing `get_current_min_rent_for_token_account` method for patterns.

**Actions:**
1. Implement the `get_current_min_rent_for_holding_account` async method  
2. Use RPC client to get minimum balance for rent exemption
3. Use `Account::LEN` for account size calculation
4. Return lamports amount in response

**Expected Implementation:**
```rust
/// Gets current minimum rent for a token holding account
async fn get_current_min_rent_for_holding_account(
    &self,
    _request: Request<GetCurrentMinRentForHoldingAccountRequest>,
) -> Result<Response<GetCurrentMinRentForHoldingAccountResponse>, Status> {
    // Get minimum balance for rent exemption using Account::LEN
    match self
        .rpc_client
        .get_minimum_balance_for_rent_exemption(Account::LEN)
    {
        Ok(lamports) => {
            let response = GetCurrentMinRentForHoldingAccountResponse { lamports };
            Ok(Response::new(response))
        }
        Err(e) => Err(Status::internal(format!(
            "Failed to get minimum balance for holding account: {e}"
        ))),
    }
}
```

**Validation:**
- Uses `Account::LEN` for holding account size
- Proper error handling for RPC client calls
- Response format matches existing patterns

#### **Step 3.4: Update Generated Imports**
**File**: `api/src/api/program/token/v1/service_impl.rs`

**Actions:**
1. Add imports for new request/response types from generated code
2. Update the use statement for the generated proto types

**Expected Import Updates:**
```rust
use protosol_api::protosol::solana::program::token::v1::{
    service_server::Service as TokenProgramService, 
    GetCurrentMinRentForTokenAccountRequest, GetCurrentMinRentForTokenAccountResponse,
    GetCurrentMinRentForHoldingAccountRequest, GetCurrentMinRentForHoldingAccountResponse,
    InitialiseMintRequest, InitialiseMintResponse,
    InitialiseHoldingAccountRequest, InitialiseHoldingAccountResponse,
    MintInfo, ParseMintRequest, ParseMintResponse,
};
```

### **Phase 4: Build and Validation**

#### **Step 4.1: Build Rust Backend**
**Command**: `cargo build --package protosol-solana-api`

**Actions:**
1. Build the Rust backend to verify compilation
2. Fix any compilation errors
3. Ensure all dependencies resolve correctly

**Success Criteria:**
- Clean compilation with no errors or warnings
- All new method implementations compile successfully

#### **Step 4.2: Run Mandatory Linting**
**Command**: `./scripts/lint/rs.sh`

**Pre-review**: This step is MANDATORY and must not be skipped. Fix all linting issues, do not use allow directives.

**Actions:**
1. Run Rust linting to check code quality
2. Fix all clippy warnings and formatting issues
3. Ensure documentation comments are proper

**Success Criteria:**
- No clippy warnings or errors
- All code properly formatted
- Documentation comments follow patterns

### **Phase 5: Enhanced E2E Test Implementation**

#### **Step 5.1: Add Test Method Stub**
**File**: `tests/go/token_program_e2e_test.go`

**Pre-review**: This adds a comprehensive test that creates both mint and holding accounts in a single atomic transaction.

**Actions:**
1. Add `Test_03_Token_e2e` method to `TokenProgramE2ETestSuite`
2. Follow the structure provided in the prompt requirements
3. Use proper error handling and assertions throughout

**Expected Test Structure:**
```go
func (suite *TokenProgramE2ETestSuite) Test_03_Token_e2e() {
    suite.T().Log("🎯 Testing Token 2022 Mint Creation and Initialization")
    
    // Generate and fund payer account
    // Generate mint account keypair  
    // Get rent for mint account
    // Create mint account instruction
    // Initialize mint instruction
    // Generate holding account keypair
    // Get rent for holding account
    // Create holding account instruction
    // Initialize holding account instruction
    // Compose atomic transaction with all 4 instructions
    // Execute transaction lifecycle (compile, sign, submit)
    // Wait for confirmation
    // Verify mint account parsing
    // Verify holding account creation
}
```

#### **Step 5.2: Implement Test Body with All Required Steps**
**File**: `tests/go/token_program_e2e_test.go`

**Pre-review**: This comprehensive test validates the entire flow. Ensure all error handling follows existing patterns and all assertions are meaningful.

**Actions:**
1. Generate payer and fund with SOL
2. Generate mint account keypair
3. Get rent for mint account using existing method
4. Create system program instruction for mint account
5. Create initialize mint instruction
6. Generate holding account keypair
7. Get rent for holding account using NEW method
8. Create system program instruction for holding account
9. Create initialize holding account instruction using NEW method
10. Compose atomic transaction with all 4 instructions
11. Execute full transaction lifecycle
12. Verify mint account via parsing
13. Add validation for holding account (parsing not yet available, but ensure creation succeeded)

**Key Implementation Points:**
- Use `token_v1.HOLDING_ACCOUNT_LEN` constant for holding account space
- Handle both mint and holding account signatures in transaction signing
- Include proper logging throughout the test
- Wait for account visibility after funding
- Monitor transaction to completion

**Expected Code Sections:**
```go
// Get rent for holding account using new method
holdingAccountRentResp, err := suite.tokenProgramService.GetCurrentMinRentForHoldingAccount(suite.ctx, &token_v1.GetCurrentMinRentForHoldingAccountRequest{})
suite.Require().NoError(err, "Should get current rent amount for token holding account")

// Create holding account instruction
createHoldingAccInstr, err := suite.systemProgramService.Create(suite.ctx, &system_v1.CreateRequest{
    Payer:      payKeyResp.KeyPair.PublicKey,
    NewAccount: holdingAccKeyResp.KeyPair.PublicKey,
    Owner:      token_v1.TOKEN_2022_PROGRAM_ID,
    Lamports:   holdingAccountRentResp.Lamports,
    Space:      token_v1.HOLDING_ACCOUNT_LEN,
})

// Initialize holding account instruction
initialiseHoldingAccountInstr, err := suite.tokenProgramService.InitialiseHoldingAccount(suite.ctx, &token_v1.InitialiseHoldingAccountRequest{
    AccountPubKey: holdingAccKeyResp.KeyPair.PublicKey,
    MintPubKey:    mintKeyResp.KeyPair.PublicKey,
    OwnerPubKey:   payKeyResp.KeyPair.PublicKey,
})
suite.Require().NoError(err, "Should initialize holding account instruction")
```

#### **Step 5.3: Test Execution and Validation**
**Actions:**
1. Ensure test services are running (validator + backend)
2. Run the specific test to validate implementation
3. Verify all assertions pass
4. Check that both accounts are created successfully on-chain

**Commands for Testing:**
```bash
# Terminal 1: Start validator
./scripts/tests/start-validator.sh

# Terminal 2: Start backend  
cargo run --package protosol-solana-api

# Terminal 3: Run specific test
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite/Test_03_Token_e2e"
```

### **Phase 6: Final Integration and Validation**

#### **Step 6.1: Run Full Test Suite**
**Actions:**
1. Run all token program tests to ensure no regressions
2. Verify all existing tests still pass
3. Confirm new test executes successfully

**Command**: `cd tests/go && RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite"`

#### **Step 6.2: Final Linting Check**
**Command**: `./scripts/lint/all.sh`

**Actions:**
1. Run comprehensive linting across all languages
2. Fix any remaining issues
3. Ensure code quality standards are met

#### **Step 6.3: Build Verification**
**Actions:**
1. Build the entire project to ensure no compilation issues
2. Verify all language bindings compile correctly
3. Check that generated code is consistent

**Commands:**
```bash
cargo build --package protosol-solana-api
cd lib/go && go build ./...
cd lib/ts && npm run build  # if build script exists
```

## **Success Criteria**

**Functional Requirements:**
- ✅ `InitialiseHoldingAccount` RPC method implemented
- ✅ `GetCurrentMinRentForHoldingAccount` RPC method implemented  
- ✅ `HOLDING_ACCOUNT_LEN` constant available in Go SDK
- ✅ Complete e2e test creates mint + holding account atomically
- ✅ All instructions return proper `SolanaInstruction` proto format

**Technical Requirements:**
- ✅ Proto definitions pass `buf lint`
- ✅ Code generation completes without errors
- ✅ Rust backend compiles without warnings
- ✅ All linting checks pass
- ✅ Integration tests pass consistently
- ✅ No regressions in existing functionality

**Quality Requirements:**
- ✅ Proper error handling throughout implementation
- ✅ Consistent naming and patterns with existing code
- ✅ Comprehensive test coverage
- ✅ Clear logging and debugging output
- ✅ Documentation follows established patterns

## **Rollback Plan**

If critical issues are discovered during implementation:

1. **Revert Proto Changes**: Restore `service.proto` to previous state
2. **Regenerate Code**: Run code generation to remove generated method stubs  
3. **Remove Rust Implementation**: Remove new method implementations from service
4. **Remove Test Changes**: Revert test file to previous state
5. **Remove Constants**: Remove `HOLDING_ACCOUNT_LEN` from constants file

## **Dependencies and Prerequisites**

**External Dependencies:**
- Local Solana test validator running on port 8899
- ProtoSol gRPC backend running on port 50051  
- `buf` CLI tool installed and accessible
- Rust toolchain with required crates available

**Internal Dependencies:**
- Existing token program service implementations
- System program service for account creation
- Transaction service for atomic execution
- Account service for keypair generation and funding

## **Risk Mitigation**

**Common Issues and Solutions:**
- **Proto Field Numbering**: Verify no conflicts with existing or future fields
- **Account Size Mismatch**: Double-check `Account::LEN` value matches Solana Token 2022
- **Transaction Signing**: Ensure both mint and holding account private keys included
- **Timing Issues**: Use proper waiting patterns for account visibility and transaction confirmation
- **Error Message Clarity**: Provide descriptive errors for debugging failed token operations

**Testing Isolation:**
- Each test generates new keypairs to avoid conflicts
- Tests fund accounts independently
- Proper cleanup and resource management in test suite