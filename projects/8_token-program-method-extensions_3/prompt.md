# **Agent Task: Add another method to the token program: Mint!

## **Persona**

# UI SDK Test Forms

You are an expert in:
- Rust async programming and Solana blockchain development
- Solana Token 2022 program and SPL Token program (you can even look into it over here: /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs)
- Protocol Buffers & gRPC end-to-end development (our project protos: ./lib/proto: SOURCE OF YOUR TRUTH)
- ProtoSol architecture patterns and code generation

## THE ALMIGHTILY CRITICAL "GOAL" (aka. The GOAL):
**THE GOAL** is to add another method to our token program service: lib/proto/protosol/solana/program/token/v1/service.proto:
- The: lib/proto/protosol/solana/program/token/v1/service.proto.Mint method
- Wrapper of: MintToChecked in /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs! We ALWAYS use the Checked methods for minting!!

## DELIVERABLE OF THIS PROMPT:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/8_token-program-method-extensions_3/implementation-plan.md an implementation plan.
The plan must be such that when fully executed THE GOAL is met.

## **CRITICAL**: More information on the GOAL:
## THE ALMIGHTILY CRITICAL "GOAL" (aka. The GOAL):
**THE GOAL** is to add another method to our token program service: lib/proto/protosol/solana/program/token/v1/service.proto:
- The: lib/proto/protosol/solana/program/token/v1/service.proto.Mint method
- Wrapper of: MintToChecked in /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs! We ALWAYS use the Checked methods for minting!!

Adding these methods means:
- proto file updates
- code regenerated
- api rust backend updated to add this new method to the protosol.solana.program.token.v1.Service implementation
- The go e2e tests: tests/go/token_program_e2e_test.go: 
  - there is 1 GOD e2e test in this file on the suite called:  Test_03_Token_e2e()
  - CRITICAL: it is critical that you extend this test with the extra logic required to get to the end of a mint test!! We can't keep creating too many new tests!
```go
// Test_03_Token_e2e tests complete mint + holding account creation flow
func (suite *TokenProgramE2ETestSuite) Test_03_Token_e2e() {
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
			// FIXME: look at existing test to fill in correct fields...
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
			// FIXME: look at existing test to fill in correct fields...
		}
	)
	suite.Require().NoError(err, "Should get current rent amount")
	suite.T().Logf("  Rent required for holding: %d lamports", rentResp.Lamports)
	
	{
  "success": true,
  "keyPair": {
    "publicKey": "9TBwKhZFnWn39x4sW9mj2ph5A3JNxdbVeT8B3FqTUK21",
    "privateKey": "XyRfkDcgopNyMZ1r52gJaNKgksSwzuWK2rht27vqoE6BTYct6Frcs5CcEWJaeBNurJYc9B1Hnbv8RQvQwZ9jGqF"
  }
}

	// BUILD INSTRUCTION(s) to mint into the holding account!:
	// - create necessary instruction(s) from system and token program for the creation of a holding account
	mintAccountResponse, err := suite.tokenProgramService.Mint(
		suite.ctx,
		// FIXME: you will need to work out the final fields here, but I think this is probably almost right!
		&token_v1.MintRequest{
			MintAddress: mintKeyResp.KeyPair.PublicKey,
			// FIXME: Check if this should be a PDA (derived address token program accounts) or not?? You should check implementation if necessary, you have the whole repo: /Users/bernardbussy/Projects/github.com/solana-program/token-2022, maybe at: /Users/bernardbussy/Projects/github.com/solana-program/token-2022/interface/src/instruction.rs is the answer??
			ReceipientAddress: holdingKeyResp.KeyPair.PublicKey, 
		}
	)
	suite.Require().NoError(err, "Should get instruction for minting!!!")
	// Compose atomic transaction
	atomicTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createMintResponse.Instructions...,
			createholdingAccountResponse.Instructions...,
			mintAccountResponse.Instructions...,
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

	/// existing validation here!!!!!!!
	
	// ADD EXTRA VALIDATIONS HERE!!! for the new functionality of that mint creation!!! You need to fetch the token account itself here to check balance!
}
```

## **CRITICAL**: Deliverable:
**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/token-program-method-extensions_1_initialiseHoldingAccount
/implementation-plan.md an implementation plan. The plan must be such that when fully executed THE GOAL is met.
**Location**: `projects/8_token-program-method-extensions_3/implementation-plan.md`  
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