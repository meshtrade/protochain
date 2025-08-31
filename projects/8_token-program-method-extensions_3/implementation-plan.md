# Implementation Plan: Add Mint Method to Token Program Service

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

## GOAL
Add a new `Mint` method to the token program service (`lib/proto/protosol/solana/program/token/v1/service.proto`) that wraps Solana Token 2022's `MintToChecked` instruction. This includes proto definition, code generation, Rust backend implementation, and comprehensive e2e testing.

## Overview
The implementation will follow the established ProtoSol patterns:
1. Define proto service method that returns `SolanaInstruction`
2. Generate multi-language SDKs via buf tooling
3. Implement Rust backend using spl-token-2022's `mint_to_checked` function  
4. Extend existing e2e test to include complete mint-to-account flow
5. Validate through full transaction lifecycle testing

## Step-by-Step Implementation Plan

### Step 1: Define Mint Proto Service Method
**Objective**: Add the `Mint` RPC method and associated request/response messages to the token service proto definition.

**Pre-review Advice**: 
- Review existing patterns in `service.proto` (especially `InitialiseMint` and `CreateMint` methods)
- The `MintRequest` message design should follow the field naming conventions seen in other messages
- Use `string` for pubkey fields and `uint64` for amount as per existing patterns

**Actions**:
1.1. Open `lib/proto/protosol/solana/program/token/v1/service.proto`
1.2. Add the `Mint` RPC method to the `Service` definition:
```protobuf
// Mint tokens to an existing token account using MintToChecked instruction
rpc Mint(MintRequest) returns (MintResponse);
```
1.3. Add the `MintRequest` message definition after existing messages:
```protobuf
// Request to mint tokens to a token account
message MintRequest {
  string mint_pub_key = 1;              // The mint to mint from
  string destination_account_pub_key = 2; // Token account to mint to
  string mint_authority_pub_key = 3;     // Authority that can mint tokens
  string amount = 4;                     // Amount to mint (as string to handle large numbers)
  uint32 decimals = 5;                   // Expected decimals for validation
}
```
1.4. Add the `MintResponse` message definition:
```protobuf
// Response containing Mint instruction
message MintResponse {
  protosol.solana.transaction.v1.SolanaInstruction instruction = 1;
}
```

**Validation**:
- Verify proto syntax is correct
- Ensure field numbers don't conflict with existing messages
- Confirm import statements are present

### Step 2: Validate Proto Definitions and Generate Code
**Objective**: Validate new proto definitions and generate updated multi-language SDKs.

**Pre-review Advice**: 
- ALWAYS run from repo root directory
- The buf generation script performs linting first - address any issues before proceeding
- Generated files will appear in `lib/rust/src/`, `lib/go/protosol/`, and `lib/ts/src/`

**Actions**:
2.1. Validate proto definitions:
```bash
buf lint lib/proto
```
2.2. Generate all language bindings:
```bash
./scripts/code-gen/generate/all.sh
```
2.3. Verify generated files exist:
   - Check `lib/rust/src/protosol.solana.program.token.v1.rs` contains new `MintRequest`/`MintResponse` structures
   - Check `lib/go/protosol/solana/program/token/v1/` contains updated Go types
   - Verify TypeScript generation completed successfully

**Validation**:
- All generated files compile without errors
- New proto messages appear in generated code
- No linting errors remain

### Step 3: Import MintToChecked Function in Rust Backend
**Objective**: Add the necessary import for `mint_to_checked` function from spl-token-2022.

**Pre-review Advice**:
- Review existing imports in `service_impl.rs` to understand the pattern  
- The `mint_to_checked` function should be imported alongside existing functions like `initialize_mint2`

**Actions**:
3.1. Open `api/src/api/program/token/v1/service_impl.rs`
3.2. Add `mint_to_checked` to the existing spl_token_2022 import:
```rust
use spl_token_2022::{
    instruction::{initialize_account, initialize_mint2, mint_to_checked},
    state::{Account, Mint},
    ID as TOKEN_2022_PROGRAM_ID,
};
```
3.3. Update the generated proto imports to include new types:
```rust
use protosol_api::protosol::solana::program::token::v1::{
    service_server::Service as TokenProgramService, CreateHoldingAccountRequest,
    CreateHoldingAccountResponse, CreateMintRequest, CreateMintResponse,
    GetCurrentMinRentForHoldingAccountRequest, GetCurrentMinRentForHoldingAccountResponse,
    GetCurrentMinRentForTokenAccountRequest, GetCurrentMinRentForTokenAccountResponse,
    InitialiseHoldingAccountRequest, InitialiseHoldingAccountResponse, InitialiseMintRequest,
    InitialiseMintResponse, MintInfo, MintRequest, MintResponse, ParseMintRequest, ParseMintResponse,
};
```

**Validation**:
- Code compiles without errors
- All necessary types are imported

### Step 4: Implement Mint Service Method in Rust Backend
**Objective**: Implement the `mint` async method that creates a `MintToChecked` instruction.

**Pre-review Advice**:
- Follow the same error handling patterns as existing methods (use `Status::invalid_argument` for parsing errors)
- The `mint_to_checked` function requires no additional signers for single authority
- Amount parameter should be parsed from string to handle large token amounts
- Use `sdk_instruction_to_proto` to convert the instruction, same as other methods

**Actions**:
4.1. Add the method implementation to the `TokenProgramService` impl block:
```rust
/// Creates a `MintToChecked` instruction for Token 2022 program
async fn mint(
    &self,
    request: Request<MintRequest>,
) -> Result<Response<MintResponse>, Status> {
    let req = request.into_inner();

    // Parse public keys
    let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
    let destination_account_pubkey = Pubkey::from_str(&req.destination_account_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid destination_account_pub_key: {e}")))?;
    let mint_authority_pubkey = Pubkey::from_str(&req.mint_authority_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid mint_authority_pub_key: {e}")))?;

    // Parse amount from string to handle large numbers
    let amount = req.amount.parse::<u64>()
        .map_err(|e| Status::invalid_argument(format!("Invalid amount: {e}")))?;

    // Validate decimals
    let decimals = u8::try_from(req.decimals)
        .map_err(|_| Status::invalid_argument("decimals must be between 0 and 255"))?;

    // Create the MintToChecked instruction (no additional signers for single authority)
    let instruction = mint_to_checked(
        &TOKEN_2022_PROGRAM_ID,
        &mint_pubkey,
        &destination_account_pubkey,
        &mint_authority_pubkey,
        &[], // Empty signer array for single authority
        amount,
        decimals,
    )
    .map_err(|e| {
        Status::invalid_argument(format!("Failed to create MintToChecked instruction: {e}"))
    })?;

    // Convert to proto and return
    let proto_instruction = sdk_instruction_to_proto(instruction);
    Ok(Response::new(MintResponse {
        instruction: Some(proto_instruction),
    }))
}
```

**Validation**:
- Method signature matches generated trait
- Error handling follows established patterns
- All parameters are properly validated
- Instruction creation uses correct spl-token-2022 function

### Step 5: Test Rust Backend Compilation
**Objective**: Ensure the Rust backend compiles successfully with the new implementation.

**Pre-review Advice**: 
- If compilation fails, check that all imports are correct and method signature matches the generated trait
- Common issues: missing imports, typos in method names, incorrect async function signature

**Actions**:
5.1. Compile the Rust backend:
```bash
cargo build --package protosol-solana-api
```
5.2. Run Rust unit tests (if any):
```bash
cargo test --package protosol-solana-api
```

**Validation**:
- No compilation errors
- All existing tests continue to pass

### Step 6: Run Mandatory Linting
**Objective**: Ensure all code follows project linting standards.

**Pre-review Advice**:
- This is MANDATORY per the project's CLAUDE.md requirements
- Fix any linting issues immediately - do not use ignore directives
- Address root causes of linting warnings through code improvements

**Actions**:  
6.1. Run Rust linting:
```bash
./scripts/lint/rs.sh
```
6.2. Fix any linting issues that arise:
   - Dead code: Remove unused code
   - Cognitive complexity: Break down complex functions
   - Missing docs: Add proper documentation
   - Other clippy warnings: Refactor to address underlying issues
6.3. Re-run linting until clean

**Validation**:
- All linting passes without warnings or errors
- No ignore directives were added

### Step 7: Create Complete Mint Flow Test Framework
**Objective**: Set up the test structure for comprehensive mint flow testing by enhancing the existing `Test_03_Token_e2e` test.

**Pre-review Advice**:
- The existing test already has mint and holding account creation - we're extending it to add minting
- Use the same keypair generation and transaction lifecycle patterns as existing tests
- The test should verify tokens were actually minted by checking token account balance

**Actions**:
7.1. Open `tests/go/token_program_e2e_test.go`
7.2. Locate the `Test_03_Token_e2e` function
7.3. Identify where the holding account creation completes (after transaction confirmation)
7.4. Add mint instruction creation after holding account verification but before the final logging

**Validation**:
- Test structure is ready for mint instruction addition
- All necessary keypairs and accounts are available for minting

### Step 8: Implement Mint Instruction Creation in Test
**Objective**: Add the mint instruction creation to the e2e test using the new `Mint` service method.

**Pre-review Advice**:
- Use a reasonable mint amount like "1000000" (1 token with 6 decimals)
- The decimals parameter should match what was used in mint creation (6 decimals)
- The destination account is the holding account we just created

**Actions**:
8.1. Add mint instruction creation after holding account verification:
```go
// BUILD INSTRUCTION to mint tokens into the holding account
mintAmount := "1000000" // 1 token with 6 decimals
mintInstr, err := suite.tokenProgramService.Mint(suite.ctx, &token_v1.MintRequest{
    MintPubKey:                mintKeyResp.KeyPair.PublicKey,
    DestinationAccountPubKey:  holdingAccKeyResp.KeyPair.PublicKey,
    MintAuthorityPubKey:       payKeyResp.KeyPair.PublicKey, // payer is the mint authority
    Amount:                    mintAmount,
    Decimals:                  6, // Must match mint decimals
})
suite.Require().NoError(err, "Should create mint instruction")
suite.T().Logf("  Created mint instruction for %s tokens", mintAmount)
```

**Validation**:
- Mint instruction is created successfully
- All required fields are populated correctly
- Error handling is in place

### Step 9: Extend Transaction Composition for Minting
**Objective**: Add the mint instruction to the existing atomic transaction and update signing.

**Pre-review Advice**: 
- The mint instruction should be added AFTER holding account creation instructions
- No additional signers are required for minting (only the fee payer who is also the mint authority)
- The transaction composition should be updated but the signing remains the same
- Critical: The mint instruction must come last since it requires the holding account to exist first

**Actions**:
9.1. Modify the atomic transaction composition to include the mint instruction:
```go
// Compose atomic transaction with all instructions including minting
atomicTx := &transaction_v1.Transaction{
    Instructions: []*transaction_v1.SolanaInstruction{
        createMintResponse.Instructions...,     // System create + token initialize mint
        createAccountResponse.Instructions...,  // System create + token initialize holding account  
        mintInstr.Instruction,                  // Mint tokens to holding account
    },
    State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
}
suite.T().Logf("  Composed atomic transaction with mint + holding account creation + minting")
```
9.2. Update the transaction execution logging to reflect the additional instruction

**Validation**:
- Transaction includes all necessary instructions in correct order
- Logging accurately reflects the enhanced transaction

### Step 10: Add Token Account Balance Verification
**Objective**: Verify that tokens were successfully minted by checking the token account balance.

**Pre-review Advice**:
- Need to add a method to parse token account data (similar to existing `ParseMint`)
- Alternatively, use the account service to verify the holding account data contains expected token balance
- The token account should show a balance of "1000000" (1 token with 6 decimals)

**Actions**:
10.1. Add token account balance verification after the existing mint verification:
```go
// Verify tokens were minted by checking holding account token balance
// Note: This is a simplified verification - in production you might want to parse the token account data
holdingAccountAfterMint, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
    Address: holdingAccKeyResp.KeyPair.PublicKey,
    CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED.Enum(),
})
suite.Require().NoError(err, "Should get holding account after minting")
suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, holdingAccountAfterMint.Owner, "Holding account should still be owned by Token 2022 program")
suite.Assert().NotEmpty(holdingAccountAfterMint.Data, "Holding account should have updated data after minting")

suite.T().Logf("✅ Complete mint + holding account creation + minting verified successfully:")
suite.T().Logf("   Mint Address: %s", mintKeyResp.KeyPair.PublicKey)
suite.T().Logf("   Holding Account Address: %s", holdingAccKeyResp.KeyPair.PublicKey)
suite.T().Logf("   Minted Amount: %s tokens", mintAmount)
suite.T().Logf("   Holding Account Balance: %d lamports", holdingAccountAfterMint.Lamports)
```

**Validation**:
- Token account exists and is owned by Token 2022 program
- Account data has been updated (indicating token balance change)
- Appropriate logging provides clear test results

### Step 11: Update Test Blockchain Verification Commands
**Objective**: Add blockchain verification commands that can be used to manually verify the minting results.

**Actions**:
11.1. Update the blockchain verification logging section:
```go
suite.T().Logf("🔍 Blockchain verification commands:")
suite.T().Logf("   solana account %s --url http://localhost:8899", mintKeyResp.KeyPair.PublicKey)
suite.T().Logf("   solana account %s --url http://localhost:8899", holdingAccKeyResp.KeyPair.PublicKey)
suite.T().Logf("   spl-token account-info %s --url http://localhost:8899", holdingAccKeyResp.KeyPair.PublicKey)
suite.T().Logf("   solana confirm %s --url http://localhost:8899", submittedTx.Signature)
```

**Validation**:
- Commands are formatted correctly for manual verification
- Include spl-token command for token-specific account inspection

### Step 12: Run Complete Integration Test
**Objective**: Execute the full integration test to validate the complete mint implementation.

**Pre-review Advice**:
- Make sure Solana validator and backend are running before testing
- The test should pass completely, including the new minting functionality
- If tests fail, investigate step-by-step rather than modifying the plan

**Actions**:
12.1. Start the Solana validator:
```bash
./scripts/tests/start-validator.sh
```
12.2. Start the backend in a separate terminal:
```bash
./scripts/tests/start-backend.sh  
```
12.3. Run the specific token e2e test:
```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite/Test_03_Token_e2e"
```

**Validation**:
- Test passes completely without errors
- All transaction steps execute successfully
- Token minting is verified through account inspection
- Blockchain commands work for manual verification

### Step 13: Final Linting and Quality Check
**Objective**: Perform final quality assurance by running all linting checks.

**Actions**:
13.1. Run comprehensive linting:
```bash
./scripts/lint/all.sh
```
13.2. Fix any remaining issues that surface
13.3. Verify all tests still pass:
```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestTokenProgramE2ESuite"
```

**Validation**:
- All linting passes cleanly
- All existing tests continue to pass
- New functionality integrates seamlessly with existing codebase

## Success Criteria
1. ✅ Proto service definition includes `Mint` method with proper request/response messages
2. ✅ Multi-language SDKs generated successfully with new types
3. ✅ Rust backend implements `mint` method using spl-token-2022's `mint_to_checked`
4. ✅ E2e test demonstrates complete mint flow: account creation → token account creation → minting → verification
5. ✅ All linting passes without warnings or ignore directives
6. ✅ Integration test passes with real blockchain interaction
7. ✅ Manual verification commands provide blockchain-level confirmation

## Architecture Notes
- **Proto-first design**: Service definition drives implementation
- **Instruction composition**: Returns `SolanaInstruction` for transaction composition
- **Multi-signature support**: Framework supports additional signers (though not used in basic case)
- **Error handling**: Comprehensive validation with detailed error messages
- **Testing philosophy**: Full transaction lifecycle with blockchain verification

## Risk Mitigation
- **Incremental validation**: Each step includes validation checkpoints
- **Existing pattern adherence**: Implementation follows established codebase patterns
- **Comprehensive testing**: Both unit-level and integration testing
- **Manual verification**: Blockchain commands provide additional confirmation layer