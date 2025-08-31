# Token Program Wrapper - MVP Implementation for InitialiseMint

You are an expert in:
- Rust async programming and Solana blockchain development
- Solana Token 2022 program and SPL Token program
- Protocol Buffers & gRPC end-to-end development
- ProtoSol architecture patterns and code generation

## Task Overview
Implement a gRPC wrapper for the Solana Token 2022 program, starting with ONLY the `InitialiseMint` instruction to limit scope. All tokens created through this SDK will be Token 2022 tokens (even without extensions) for future extensibility.

Follow the exact same architectural pattern as the system program:
- **Proto definition**: `lib/proto/protosol/solana/program/system/v1/service.proto`
- **Rust implementation**: `api/src/api/program/system/v1/service_impl.rs`
- **Integration test**: `tests/go/composable_e2e_test.go`

## Required Service Methods

### 1. InitialiseMint Instruction
```protobuf
rpc InitialiseMint(InitialiseMintRequest) returns (InitialiseMintResponse);
```

### 2. GetCurrentMinRentForTokenAccount
```protobuf
rpc GetCurrentMinRentForTokenAccount(GetCurrentMinRentForTokenAccountRequest) returns (GetCurrentMinRentForTokenAccountResponse);
```
- Must query the Solana RPC to get current rent-exempt balance for mint account size
- Use `Mint::LEN` from the Solana Token SDK for account size

### 3. ParseMint
```protobuf
rpc ParseMint(ParseMintRequest) returns (ParseMintResponse);
```
- Must parse raw account data into structured mint information
- Use Token 2022 SDK deserialization functions
- Return mint authority, freeze authority, decimals, supply, etc.

## Target Implementation Test
The following integration test must work after implementation:

```go
func (suite *TokenProgramE2ETestSuite) Test_InitialiseMint() {
    // Generate payer account
    payKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
    suite.Require().NoError(err, "Should generate payer keypair")

    // Fund payer account
    fundResp, err := suite.accountService.FundNative(suite.ctx, &account_v1.FundNativeRequest{
        Address: payKeyResp.KeyPair.PublicKey,
        Amount:  "5000000000", // 5 SOL
    })
    suite.Require().NoError(err, "Should fund payer account")

    // Generate mint account keypair
    mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
    suite.Require().NoError(err, "Should generate mint keypair")

    // Get current rent for token account
    rentResp, err := suite.tokenProgramService.GetCurrentMinRentForTokenAccount(suite.ctx, &token_v1.GetCurrentMinRentForTokenAccountRequest{})
    suite.Require().NoError(err, "Should get current rent amount")

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
            createMintInstr,
            initialiseMintInstr,
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

    // Wait for confirmation
    suite.monitorTransactionToCompletion(submittedTx.Signature)

    // Verify mint creation by parsing the account
    parsedMint, err := suite.tokenProgramService.ParseMint(suite.ctx, &token_v1.ParseMintRequest{
        AccountAddress: mintKeyResp.KeyPair.PublicKey,
    })
    suite.Require().NoError(err, "Should parse mint account")
    suite.Assert().Equal(uint32(2), parsedMint.Mint.Decimals, "Mint should have 2 decimals")
    suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.MintAuthorityPubKey, "Mint authority should match")
    suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.FreezeAuthorityPubKey, "Freeze authority should match")
    suite.Assert().Equal("0", parsedMint.Mint.Supply, "Initial supply should be zero")
}
```

## Research and Analysis Required
Create comprehensive research todo list covering:
- Existing ProtoSol system program architecture analysis 
- Token 2022 program SDK integration patterns
- Proto message design following ProtoSol conventions
- Rust service implementation patterns
- Code generation workflow requirements
- NOTE:
  - entire token-2022 rust sdk repository available here! /Users/bernardbussy/Projects/github.com/solana-program/token-2022
  - entire node repo here: /Users/bernardbussy/Projects/github.com/anza-xyz/agave

Use step-by-step approach: draft, review, refine sections iteratively.

## Technical Architecture Requirements

### 1. Proto Service Design
- **Namespace**: `protosol.solana.program.token.v1`
- **Methods**: InitialiseMint, GetCurrentMinRentForTokenAccount, ParseMint
- **Return Types**: All methods return `SolanaInstruction` for composability
- **Constants**: TOKEN_2022_PROGRAM_ID, MINT_ACCOUNT_LEN exposed as proto constants
- **Message Design**: Follow existing system program patterns exactly

### 2. Rust Backend Implementation  
- **Location**: `api/src/api/program/token/v1/`
- **Dependencies**: spl-token-2022, solana-sdk, solana-program
- **Service Pattern**: Follow system program service_impl.rs exactly
- **Conversion Layer**: Proto ↔ Solana SDK instruction conversion
- **RPC Integration**: Use existing Solana client for rent calculations

### 3. Integration Testing
- **Test Suite**: New TokenProgramE2ETestSuite in tests/go/
- **Validation**: Complete mint creation, initialization, and parsing cycle
- **Dependencies**: Must integrate with existing account and transaction services
- **Blockchain Verification**: Real transaction submission and account parsing

## Architecture References
**Critical References for Token Program Implementation:**

### ProtoSol System Program Pattern (MANDATORY TO STUDY):
- **Proto Definition**: `lib/proto/protosol/solana/program/system/v1/service.proto`
- **Rust Implementation**: `api/src/api/program/system/v1/service_impl.rs`
- **Conversion Functions**: `api/src/api/program/system/v1/conversion.rs`
- **Integration Tests**: `tests/go/composable_e2e_test.go`

### Solana Token 2022 SDK References:
- **Token 2022 Program ID**: Find in `/Users/bernardbussy/Projects/github.com/solana-program/token-2022`
- **Mint struct and LEN constant**: From `spl-token-2022` crate
- **initialize_mint2 instruction**: Token 2022 initialization function
- **Account parsing**: Token 2022 account deserialization functions

### External Documentation:
- **Solana Token Program Guide**: https://solana.com/developers/guides/token-extensions/getting-started
- **SPL Token 2022 Documentation**: https://solana-labs.github.io/solana-program-library/token-2022/
- **Protocol Buffer Style Guide**: https://protobuf.dev/programming-guides/style/

## Token Program Implementation Reference
```rust
// Reference implementation pattern for mint creation
use solana_sdk::{
    program_pack::Pack,
    system_instruction::create_account,
};
use spl_token_2022::{
    instruction::initialize_mint2,
    state::Mint,
    ID as TOKEN_2022_PROGRAM_ID,
};

// Constants needed for proto definitions
const MINT_ACCOUNT_LEN: u64 = Mint::LEN as u64;
const TOKEN_2022_PROGRAM_ID: Pubkey = TOKEN_2022_PROGRAM_ID;

// Mint account creation instruction
let create_mint_account_ix = create_account(
    &payer.pubkey(),
    &mint_account.pubkey(),
    rent_lamports,
    MINT_ACCOUNT_LEN,
    &TOKEN_2022_PROGRAM_ID,
);

// Mint initialization instruction
let initialize_mint_ix = initialize_mint2(
    &TOKEN_2022_PROGRAM_ID,
    &mint_account.pubkey(),
    &mint_authority.pubkey(),
    freeze_authority.as_ref().map(|k| &k.pubkey()),
    decimals,
)?;
```

## Critical Implementation Constraints

### System Program Dependency (MUST ADDRESS FIRST)
The test code reveals that the system program Create method currently hardcodes the owner to `system_program::id()`. This MUST be updated to accept an owner parameter for token account creation:

```rust
// Current system program implementation problem:
let instruction = system_instruction::create_account(
    &payer,
    &new_account, 
    req.lamports,
    req.space,
    &system_program::id(), // ❌ HARDCODED - needs to be from request
);
```

**Required Changes:**
1. Update `system/v1/service.proto` to add `owner` field to `CreateRequest`
2. Update system program service implementation to use the owner from request  
3. Generate constants files: `lib/go/protosol/solana/program/system/v1/consts.go` and `lib/go/protosol/solana/program/token/v1/consts.go`
4. Update existing system program tests to provide owner explicitly

### Technical Constraints
- **Token Program**: Use Token 2022 program ID for all operations (future extensibility)
- **Proto Design**: Follow exact patterns from system program (naming, structure, return types)
- **Testing**: Full end-to-end validation including blockchain state verification  
- **Dependencies**: Integration with existing ProtoSol services (account, transaction, system)
- **Code Generation**: Must work with existing buf.gen.yaml and protoc-gen-protosolgo
- **Architecture**: Pure gRPC wrapper - no business logic, just SDK instruction construction

### Success Criteria
Implementation is complete when:
1. ✅ Token 2022 mint can be created and initialized via gRPC
2. ✅ Current rent calculation works via dedicated RPC method
3. ✅ Mint account parsing returns structured data correctly
4. ✅ Full integration test passes with real blockchain interaction
5. ✅ System program owner parameter issue is resolved
6. ✅ Generated Go SDK includes proper constants and interfaces
7. ✅ All ProtoSol architectural patterns are followed exactly