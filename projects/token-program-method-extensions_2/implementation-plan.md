# Token Program Method Extensions 2: CreateMint & CreateHoldingAccount Implementation Plan

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

## Overview

This plan adds two new merged methods to the token program service:

1. **`CreateMint()`** - Merges system account creation + mint initialization in a single service call
2. **`CreateHoldingAccount()`** - Merges system account creation + holding account initialization in a single service call

Both methods return multiple instructions for atomic transaction composition, eliminating the need for clients to manually orchestrate the create → initialize flow.

## Goal Architecture

### Current Flow (Manual Orchestration):
```
Client → GetCurrentMinRentForTokenAccount() → rent amount
Client → SystemProgram.Create() → create instruction  
Client → TokenProgram.InitialiseMint() → init instruction
Client → Compose transaction with both instructions
```

### Target Flow (Merged Service):
```  
Client → TokenProgram.CreateMint() → [create instruction, init instruction]
Client → Compose transaction with returned instructions
```

## Implementation Steps

### Phase 1: Protocol Buffer Definitions

#### Step 1.1: Define CreateMint Proto Messages
**Pre-review**: The request message combines fields from system Create + token InitialiseMint. The response returns multiple instructions for atomic composition.

Update `lib/proto/protosol/solana/program/token/v1/service.proto`:

```protobuf
// Request to create and initialize a mint account in one call
message CreateMintRequest {
  // System program create fields
  string payer = 1;           // Account paying for creation (signer)
  string new_account = 2;     // Mint account to create (signer)
  
  // Token program initialize mint fields  
  string mint_pub_key = 3;              // Same as new_account for validation
  string mint_authority_pub_key = 4;    // Mint authority 
  string freeze_authority_pub_key = 5;  // Freeze authority (optional)
  uint32 decimals = 6;                  // Mint decimals
}

// Response containing both create and initialize instructions
message CreateMintResponse {
  repeated protosol.solana.transaction.v1.SolanaInstruction instructions = 1;
}
```

#### Step 1.2: Define CreateHoldingAccount Proto Messages

Add to the same proto file:

```protobuf
// Request to create and initialize a holding account in one call  
message CreateHoldingAccountRequest {
  // System program create fields
  string payer = 1;           // Account paying for creation (signer)
  string new_account = 2;     // Holding account to create (signer)
  
  // Token program initialize holding account fields
  string holding_account_pub_key = 3;   // Same as new_account for validation
  string mint_pub_key = 4;              // Mint this account will hold
  string owner_pub_key = 5;             // Owner of the holding account
}

// Response containing both create and initialize instructions
message CreateHoldingAccountResponse {
  repeated protosol.solana.transaction.v1.SolanaInstruction instructions = 1;
}
```

#### Step 1.3: Add Service Methods

Add to the `Service` definition in the same proto file:

```protobuf
// Creates both system account creation and mint initialization instructions
rpc CreateMint(CreateMintRequest) returns (CreateMintResponse);

// Creates both system account creation and holding account initialization instructions  
rpc CreateHoldingAccount(CreateHoldingAccountRequest) returns (CreateHoldingAccountResponse);
```

#### Step 1.4: Validate Proto Changes
Run proto validation:
```bash
buf lint lib/proto
```

### Phase 2: Code Generation

#### Step 2.1: Generate Code From Proto Definitions
**Pre-review**: This regenerates all SDKs. Ensure the generation script completes successfully before proceeding.

```bash
./scripts/code-gen/generate/all.sh
```

#### Step 2.2: Verify Generated Code Structure
Check that new messages and service methods appear in:
- `lib/rust/src/protosol.solana.program.token.v1.rs`  
- `lib/go/protosol/solana/program/token/v1/`
- `lib/ts/src/protosol/solana/program/token/v1/`

### Phase 3: Rust Backend Implementation

#### Step 3.1: Add Missing Imports
**Pre-review**: Check existing imports in `api/src/api/program/token/v1/service_impl.rs` to understand the import patterns.

Update imports in `api/src/api/program/token/v1/service_impl.rs`:

```rust
use protosol_api::protosol::solana::program::token::v1::{
    // ... existing imports ...
    CreateMintRequest, CreateMintResponse,
    CreateHoldingAccountRequest, CreateHoldingAccountResponse,
};
use protosol_api::protosol::solana::program::system::v1::CreateRequest as SystemCreateRequest;
```

#### Step 3.2: Implement CreateMint Method
**Pre-review**: This method internally calls existing service methods. The system program service is a pure instruction builder (no RPC client needed), but token program service needs RPC client for rent calculation.

Add to the `TokenProgramServiceImpl` impl block:

```rust
/// Creates both system account creation and mint initialization instructions
async fn create_mint(
    &self,
    request: Request<CreateMintRequest>, 
) -> Result<Response<CreateMintResponse>, Status> {
    let req = request.into_inner();
    
    // Validation
    if req.payer.is_empty() {
        return Err(Status::invalid_argument("Payer address is required"));
    }
    if req.new_account.is_empty() {
        return Err(Status::invalid_argument("New account address is required"));  
    }
    if req.mint_pub_key != req.new_account {
        return Err(Status::invalid_argument("mint_pub_key must match new_account"));
    }
    
    // Step 1: Get current rent for mint account
    let rent_response = self.get_current_min_rent_for_token_account(
        Request::new(GetCurrentMinRentForTokenAccountRequest {})
    ).await?.into_inner();
    
    // Step 2: Create system account creation instruction
    let system_service = SystemProgramServiceImpl::new();
    let create_instruction = system_service.create(Request::new(SystemCreateRequest {
        payer: req.payer.clone(),
        new_account: req.new_account.clone(),
        owner: TOKEN_2022_PROGRAM_ID.to_string(),
        lamports: rent_response.lamports,
        space: u64::from(Mint::LEN),
    })).await?.into_inner();
    
    // Step 3: Create mint initialization instruction  
    let init_response = self.initialise_mint(Request::new(InitialiseMintRequest {
        mint_pub_key: req.mint_pub_key,
        mint_authority_pub_key: req.mint_authority_pub_key,
        freeze_authority_pub_key: req.freeze_authority_pub_key,
        decimals: req.decimals,
    })).await?.into_inner();
    
    // Step 4: Compose response with both instructions
    let mut instructions = vec![create_instruction];
    if let Some(init_instruction) = init_response.instruction {
        instructions.push(init_instruction);
    }
    
    Ok(Response::new(CreateMintResponse {
        instructions,
    }))
}
```

#### Step 3.3: Implement CreateHoldingAccount Method
**Pre-review**: Similar pattern to CreateMint but for holding accounts.

Add to the same impl block:

```rust
/// Creates both system account creation and holding account initialization instructions
async fn create_holding_account(
    &self,
    request: Request<CreateHoldingAccountRequest>,
) -> Result<Response<CreateHoldingAccountResponse>, Status> {
    let req = request.into_inner();
    
    // Validation
    if req.payer.is_empty() {
        return Err(Status::invalid_argument("Payer address is required"));
    }
    if req.new_account.is_empty() {
        return Err(Status::invalid_argument("New account address is required"));
    }
    if req.holding_account_pub_key != req.new_account {
        return Err(Status::invalid_argument("holding_account_pub_key must match new_account"));
    }
    
    // Step 1: Get current rent for holding account
    let rent_response = self.get_current_min_rent_for_holding_account(
        Request::new(GetCurrentMinRentForHoldingAccountRequest {})
    ).await?.into_inner();
    
    // Step 2: Create system account creation instruction
    let system_service = SystemProgramServiceImpl::new();
    let create_instruction = system_service.create(Request::new(SystemCreateRequest {
        payer: req.payer.clone(),
        new_account: req.new_account.clone(),
        owner: TOKEN_2022_PROGRAM_ID.to_string(),
        lamports: rent_response.lamports,
        space: u64::from(Account::LEN),
    })).await?.into_inner();
    
    // Step 3: Create holding account initialization instruction
    let init_response = self.initialise_holding_account(Request::new(InitialiseHoldingAccountRequest {
        account_pub_key: req.holding_account_pub_key,
        mint_pub_key: req.mint_pub_key,
        owner_pub_key: req.owner_pub_key,
    })).await?.into_inner();
    
    // Step 4: Compose response with both instructions
    let mut instructions = vec![create_instruction];
    if let Some(init_instruction) = init_response.instruction {
        instructions.push(init_instruction);
    }
    
    Ok(Response::new(CreateHoldingAccountResponse {
        instructions,
    }))
}
```

#### Step 3.4: Add System Program Service Import
**Pre-review**: Check existing imports to see if SystemProgramServiceImpl is already imported. Add import if missing.

Add to imports if not present:
```rust
use crate::api::program::system::v1::service_impl::SystemProgramServiceImpl;
```

### Phase 4: Compilation and Validation

#### Step 4.1: Compile Rust Backend
**Pre-review**: Fix any compilation errors that arise. Common issues: missing imports, type mismatches, incorrect field names.

```bash
cargo build --package protosol-solana-api
```

#### Step 4.2: Run Rust Linting
```bash
./scripts/lint/rs.sh  
```

### Phase 5: Integration Testing

#### Step 5.1: Add New Test Method to Token E2E Suite
**Pre-review**: Check existing test patterns in `tests/go/token_program_e2e_test.go` for helper method usage and test structure.

Add to `tests/go/token_program_e2e_test.go`:

```go
// Test_04_Token_e2e tests the new merged CreateMint and CreateHoldingAccount methods
func (suite *TokenProgramE2ETestSuite) Test_04_Token_e2e() {
    suite.T().Log("🎯 Testing Token 2022 Merged CreateMint and CreateHoldingAccount")

    // Generate payer account
    payKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
    suite.Require().NoError(err, "Should generate payer keypair")

    // Fund payer account
    fundNativeResponse, err := suite.accountService.FundNative(suite.ctx, &account_v1.FundNativeRequest{
        Address: payKeyResp.KeyPair.PublicKey,
        Amount:  "5000000000", // 5 SOL
    })
    suite.Require().NoError(err, "Should fund payer account")
    suite.T().Logf("  Funded payer account: %s", payKeyResp.KeyPair.PublicKey)

    // Wait for payer account to be funded
    suite.monitorTransactionToCompletion(fundNativeResponse.GetSignature())
    suite.waitForAccountVisible(payKeyResp.KeyPair.PublicKey)

    // Generate mint account keypair
    mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
    suite.Require().NoError(err, "Should generate mint keypair")
    suite.T().Logf("  Generated mint account: %s", mintKeyResp.KeyPair.PublicKey)

    // Call new merged CreateMint method
    createMintResponse, err := suite.tokenProgramService.CreateMint(
        suite.ctx,
        &token_v1.CreateMintRequest{
            // System program create fields
            Payer:      payKeyResp.KeyPair.PublicKey,
            NewAccount: mintKeyResp.KeyPair.PublicKey,

            // Token program initialise mint fields
            MintPubKey:            mintKeyResp.KeyPair.PublicKey,
            MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
            FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
            Decimals:              2,            
        }
    )
    suite.Require().NoError(err, "Should create mint instructions")
    suite.Require().NotNil(createMintResponse.Instructions, "Should return instructions")
    suite.Require().Len(createMintResponse.Instructions, 2, "Should return exactly 2 instructions")
    suite.T().Logf("  CreateMint returned %d instructions", len(createMintResponse.Instructions))

    // Generate holding account keypair
    holdingKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
    suite.Require().NoError(err, "Should generate holding keypair")
    suite.T().Logf("  Generated holding account: %s", holdingKeyResp.KeyPair.PublicKey)

    // Call new merged CreateHoldingAccount method
    createHoldingAccountResponse, err := suite.tokenProgramService.CreateHoldingAccount(
        suite.ctx,
        &token_v1.CreateHoldingAccountRequest{
            // System program create fields
            Payer:      payKeyResp.KeyPair.PublicKey,
            NewAccount: holdingKeyResp.KeyPair.PublicKey,

            // Token program initialise holding account fields
            HoldingAccountPubKey: holdingKeyResp.KeyPair.PublicKey,
            MintPubKey:           mintKeyResp.KeyPair.PublicKey,
            OwnerPubKey:          payKeyResp.KeyPair.PublicKey,
        }
    )
    suite.Require().NoError(err, "Should create holding account instructions")
    suite.Require().NotNil(createHoldingAccountResponse.Instructions, "Should return instructions")
    suite.Require().Len(createHoldingAccountResponse.Instructions, 2, "Should return exactly 2 instructions")
    suite.T().Logf("  CreateHoldingAccount returned %d instructions", len(createHoldingAccountResponse.Instructions))

    // Compose atomic transaction with all instructions
    allInstructions := []*transaction_v1.SolanaInstruction{}
    allInstructions = append(allInstructions, createMintResponse.Instructions...)
    allInstructions = append(allInstructions, createHoldingAccountResponse.Instructions...)
    
    atomicTx := &transaction_v1.Transaction{
        Instructions: allInstructions,
        State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
    }
    suite.T().Logf("  Composed atomic transaction with %d instructions", len(allInstructions))

    // Execute transaction lifecycle
    compiledTx, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
        Transaction: atomicTx,
        FeePayer:    payKeyResp.KeyPair.PublicKey,
    })
    suite.Require().NoError(err, "Should compile transaction")

    // Sign transaction
    signedTx, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
        Transaction: compiledTx.Transaction,
        SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
            PrivateKeys: &transaction_v1.SignWithPrivateKeys{
                PrivateKeys: []string{
                    payKeyResp.KeyPair.PrivateKey,        // payer signature
                    mintKeyResp.KeyPair.PrivateKey,       // mint account signature
                    holdingKeyResp.KeyPair.PrivateKey,    // holding account signature
                },
            },
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

    // Verify holding account exists and is properly owned
    holdingAccount, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
        Address: holdingKeyResp.KeyPair.PublicKey,
        CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED.Enum(),
    })
    suite.Require().NoError(err, "Should get holding account")
    suite.Require().NotNil(holdingAccount, "Holding account should exist")
    suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, holdingAccount.Owner, "Holding account should be owned by Token 2022 program")
    suite.Assert().NotEmpty(holdingAccount.Data, "Holding account should have data")

    suite.T().Logf("✅ Holding account created and verified successfully:")
    suite.T().Logf("   Holding Account Address: %s", holdingKeyResp.KeyPair.PublicKey)
    suite.T().Logf("   Owner: %s", holdingAccount.Owner)
    suite.T().Logf("   Balance: %d lamports", holdingAccount.Lamports)

    suite.T().Logf("🔍 Blockchain verification commands:")
    suite.T().Logf("   solana account %s --url http://localhost:8899", mintKeyResp.KeyPair.PublicKey)
    suite.T().Logf("   solana account %s --url http://localhost:8899", holdingKeyResp.KeyPair.PublicKey)
    suite.T().Logf("   solana confirm %s --url http://localhost:8899", submittedTx.Signature)
}
```

### Phase 6: End-to-End Testing

#### Step 6.1: Start Test Environment
**Pre-review**: Ensure all services are running before attempting tests.

Start services in separate terminals:
```bash
# Terminal 1
./scripts/tests/start-validator.sh

# Terminal 2  
./scripts/tests/start-backend.sh
```

#### Step 6.2: Run Integration Tests
**Pre-review**: Run the new test specifically to isolate any issues.

```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite/Test_04_Token_e2e"
```

#### Step 6.3: Run All Token Tests
**Pre-review**: Ensure existing tests still pass with new changes.

```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite"
```

### Phase 7: Documentation and Cleanup

#### Step 7.1: Update Method Documentation
**Pre-review**: Add comprehensive documentation for the new methods in the proto file.

Update proto comments in `lib/proto/protosol/solana/program/token/v1/service.proto`:

```protobuf
service Service {
  // ... existing methods ...
  
  // Creates both system account creation and mint initialization instructions
  // Returns 2 instructions: [system create account, token initialize mint]
  // These should be executed atomically in a single transaction
  rpc CreateMint(CreateMintRequest) returns (CreateMintResponse);
  
  // Creates both system account creation and holding account initialization instructions  
  // Returns 2 instructions: [system create account, token initialize holding account]
  // These should be executed atomically in a single transaction
  rpc CreateHoldingAccount(CreateHoldingAccountRequest) returns (CreateHoldingAccountResponse);
}
```

#### Step 7.2: Add ParseHoldingAccount Method (If Not Exists)
**Pre-review**: Check if ParseHoldingAccount method exists in proto and implementation. If not, add it following the ParseMint pattern.

This may require additional proto definitions, implementation, and testing.

#### Step 7.3: Final Linting Pass
```bash
./scripts/lint/all.sh
```

## Validation Checkpoints

### After Each Phase:
- [ ] Code compiles successfully
- [ ] No linting errors  
- [ ] All existing tests pass
- [ ] New functionality works as expected

### Final Success Criteria:
- [ ] `CreateMint()` method returns 2 instructions (system create + token initialize mint)
- [ ] `CreateHoldingAccount()` method returns 2 instructions (system create + token initialize holding account)
- [ ] Both methods internally handle rent calculation automatically
- [ ] Integration test creates working mint + holding account atomically
- [ ] All existing token program tests continue to pass
- [ ] Blockchain verification confirms accounts are properly created and initialized

## Implementation Tips

### Error Handling:
- Validate that `mint_pub_key == new_account` in CreateMint
- Validate that `holding_account_pub_key == new_account` in CreateHoldingAccount  
- Propagate errors from internal service calls properly
- Use descriptive error messages

### Testing Strategy:
- Test merged methods individually first
- Test atomic composition of returned instructions  
- Verify account ownership and initialization state
- Test error conditions (invalid inputs, insufficient funds, etc.)

### Performance Considerations:
- Methods make internal RPC calls for rent calculation
- Consider caching rent values if called frequently
- Instructions should be lightweight (pure computation)

## Dependencies

### Internal Dependencies:
- System program service (`SystemProgramServiceImpl`)
- Existing token program methods (`initialise_mint`, `initialise_holding_account`)
- Rent calculation methods (`get_current_min_rent_*`)

### External Dependencies:
- solana-sdk for constants (Mint::LEN, Account::LEN)
- spl-token-2022 for TOKEN_2022_PROGRAM_ID
- Proper RPC client for rent calculation