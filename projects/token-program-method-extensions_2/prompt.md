# **Agent Task: Add another method to the token program: Merge create and initialise for token program**

## **Persona**

# UI SDK Test Forms

You are an expert in:
- Rust async programming and Solana blockchain development
- Solana Token 2022 program and SPL Token program (you can even look into it over here: /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs)
- Protocol Buffers & gRPC end-to-end development (our project protos: ./lib/proto: SOURCE OF YOUR TRUTH)
- ProtoSol architecture patterns and code generation

## THE ALMIGHTILY CRITICAL "GOAL" (aka. The GOAL):
**THE GOAL** is to add 2 new merged methods to our token program service: lib/proto/protosol/solana/program/token/v1/service.proto:
--> protosol.solana.program.token.v1.Service.CreateMint(), which in implementation in ./api will rely on :
---> the protosol.solana.program.token.v1.service.Create(), to create an account owned by the token2022 program
---> and then call its own protosol.solana.program.token.v1.Service.InitialiseMint method.
Then merge the instructions and return for transaction signing!!!

--> protosol.solana.program.token.v1.Service.CreateHoldingAccount(), which in implementation in ./api will rely on:
---> the protosol.solana.program.token.v1.service.Create(), to create an account owned by the token2022 program
---> and then call its own protosol.solana.program.token.v1.Service.InitialiseHoldingAccount method.
Then merge the instructions and return for transaction signing!!!


## DELIVERABLE OF THIS PROMPT:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/token-program-method-extensions_2/implementation-plan.md an implementation plan.
The plan must be such that when fully executed THE GOAL is met.

## **CRITICAL**: More information on the GOAL:
**THE GOAL** is to add 2 new merged methods to our token program service: lib/proto/protosol/solana/program/token/v1/service.proto:
--> protosol.solana.program.token.v1.Service.CreateMint(), which in implementation in ./api will rely on :
---> the protosol.solana.program.token.v1.service.Create(), to create an account owned by the token2022 program
---> and then call its own protosol.solana.program.token.v1.Service.GetCurrentMinRentForMintAccount
---> and then call its own protosol.solana.program.token.v1.Service.InitialiseMint method.
Then merge the instructions and return for transaction compilation!!!

--> protosol.solana.program.token.v1.Service.CreateHoldingAccount(), which in implementation in ./api will rely on:
---> the protosol.solana.program.token.v1.service.Create(), to create an account owned by the token2022 program
---> and then call its own protosol.solana.program.token.v1.Service.GetCurrentMinRentForHoldingAccount
---> and then call its own protosol.solana.program.token.v1.Service.InitialiseHoldingAccount method.
Then merge the instructions and return for transaction compilation!!!

Adding these methods means:
- proto file updates
- code regenerated
- api rust backend updated to add this new method to the protosol.solana.program.token.v1.Service implementation
- The go e2e tests: tests/go/token_program_e2e_test.go: we now want the acceptance integration test to be updgraded to have a new test method to test this token e2e...:
```go
func (suite *TokenProgramE2ETestSuite) Test_04_Token_e2e() {
	suite.T().Log("🎯 Testing Token 2022 Holding Account Creation and Initialization")

	// Generate payer account
	payKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate payer keypair")

	// Fund payer account!
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

	// BUILD INSTRUCTION(s) to:
	// - create necessary instruction(s) from system and token program for the creation of a mint account
	createMintResponse, err := suite.tokenProgramService.CreateMint(
		suite.ctx,
		&token_v1.CreateMintRequest{
			// System program create fields
			Payer:      payKeyResp.KeyPair.PublicKey,
			NewAccount: mintKeyResp.KeyPair.PublicKey,
			// Owner no longer required because inside suite.tokenProgramService on the backend we are just using the value out the solana sdk
			// Owner:      token_v1.TOKEN_2022_PROGRAM_ID,
			// Lamports, also not required - same reason as 'Owner'
			// Lamports:   mintAccountRentResp.Lamports,
			// Space, also not required - same reason as 'Owner'
			// Space:      token_v1.MINT_ACCOUNT_LEN,

			// token program initialise mint field
			MintPubKey:            mintKeyResp.KeyPair.PublicKey,
			MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
			FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
			Decimals:              2,			
		}
	)
	suite.Require().NoError(err, "Should get current rent amount")

	// Generate holding account keypair
	holdingKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate holding keypair")
	suite.T().Logf("  Generated holding account: %s", holdingKeyResp.KeyPair.PublicKey)

	// BUILD INSTRUCTION(s) to:
	// - create necessary instruction(s) from system and token program for the creation of a holding account
	createAccountResponse, err := suite.tokenProgramService.CreateAccount(
		suite.ctx,
		&token_v1.CreateAccountRequest{
			// System program create fields
			Payer:      payKeyResp.KeyPair.PublicKey,
			NewAccount: holdingKeyResp.KeyPair.PublicKey,
			// Owner no longer required because inside suite.tokenProgramService on the backend we are just using the value out the solana sdk
			// Owner:      token_v1.TOKEN_2022_PROGRAM_ID,
			// Lamports, also not required - same reason as 'Owner'
			// Lamports:   holdingAccountRentResp.Lamports,
			// Space, also not required - same reason as 'Owner'
			// Space:      token_v1.HOLDING_ACCOUNT_LEN,

			// token program initialise account fields
			// FIXME: add all the right fields here
			holdingAccountPubKey:     holdingKeyResp.KeyPair.PublicKey,
		}
	)
	suite.Require().NoError(err, "Should get current rent amount")
	suite.T().Logf("  Rent required for holding: %d lamports", rentResp.Lamports)

	// Compose atomic transaction
	atomicTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createMintResponse.Instructions...,
			createholdingAccountResponse.Instructions...,
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}

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
					holdingAccKeyResp.KeyPair.PrivateKey, // holding account signature
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

	// ADD EXTRA VALIDATIONS HERE!!! for the new functionality of that account creation!!! FIXME: YOU also NEED TO ADD A suite.tokenProgramService.ParseAccount service!!!!!
}
```

## **CRITICAL**: Deliverable:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/token-program-method-extensions_1_initialiseHoldingAccount
/implementation-plan.md an implementation plan. The plan must be such that when fully executed THE GOAL is met.
**Location**: `projects/token-program-method-extensions_2/implementation-plan.md`  
**Decription**: a step-by-step, purely technical, super comprehensive implementation plan for an agent to follow to achieve THE GOAL.
**Content**: Small, incremental technical steps
- Step-by-step validation checkpoints
- Technical dependencies and build ordering
- Resource management and cleanup requirements
- Each step builds on previous steps
- No orphaned or hanging code at any stage
- Prioritize incremental progress over large changes

**IMPORTANT: workflow while making this implementation plan**:
We must build up and break down the plan in passes. NO ONE SHOTTING. i.e. build up a sold solid plan bit by bit. Then look at it and, break it down into small, iterative chunks that build on each other. Look at these chunks and then go another round to break it into small steps. Review the results and make sure that the steps are small enough to be implemented safely with strong testing, but big enough to move the implementation forward. Iterate until you feel that the steps are right sized for achieving this goal!

This workflow ends with you taking a final look at implementation-plan.md: and taking a moment to write a "pre-review" advice section before each implementation step that could need it (i.e. this is optional) to add reassurances or review requirements like:
- note that code in this section is pseudo code and not final level, final implementation determined by agent! (i.e. here in the plan we have summary level code that you the implmementing agent must determine)
- check X files for special reference before doing this step!
- extra loosely associated file to look at before jumping in to this step: path, to some file

**FORBIDDEN CONTENT**:
- Implementation timelines, schedules, or time allocations
- Human workflow recommendations or project management advice
- References to daily/weekly/monthly patterns
- Time estimates for individual steps or overall completion

**Required HEDAER CONTENT for the implementation-plan.md file:**: some standard help for the Implementing Agent bout how to about its work:
"""
---
## CRITICAL: Context Management, Progress and Quality
Some critical information for the implemenation agent.

### Important Notice to Implementation Agent on Step Comletion

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
"""

## Research and Analysis Required
Construct yourself a comprehensive research todo list and execute it to get info you need as you build the implementation plan. It should cover:
- Existing lib/proto/protosol/solana api protobuf architecture and context. PROTOBUF files are source of truth in this repo.
- Existing ./api backend implementation of the services
- Existing tests/go integration e2e tests
- Rust service implementation patterns
- Code generation workflow requirements

THINK VERY HARD and lets get this perfect implemenation-plan.md written to achive the GOAL!!