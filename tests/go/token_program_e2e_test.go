package apitest

import (
	"context"
	"encoding/base64"
	"encoding/binary"
	"encoding/json"
	"io"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	account_v1 "github.com/BRBussy/protochain/lib/go/protochain/solana/account/v1"
	system_v1 "github.com/BRBussy/protochain/lib/go/protochain/solana/program/system/v1"
	token_v1 "github.com/BRBussy/protochain/lib/go/protochain/solana/program/token/v1"
	transaction_v1 "github.com/BRBussy/protochain/lib/go/protochain/solana/transaction/v1"
	type_v1 "github.com/BRBussy/protochain/lib/go/protochain/solana/type/v1"
)

// TokenProgramE2ETestSuite tests the Token Program service functionality
type TokenProgramE2ETestSuite struct {
	suite.Suite
	ctx                  context.Context
	cancel               context.CancelFunc
	grpcConn             *grpc.ClientConn
	accountService       account_v1.ServiceClient
	transactionService   transaction_v1.ServiceClient
	systemProgramService system_v1.ServiceClient
	tokenProgramService  token_v1.ServiceClient
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
		_ = suite.grpcConn.Close()
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
	suite.waitForAccountVisible(fundResp.GetSignature(), payKeyResp.KeyPair.PublicKey)

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
			createMintInstr,
			initialiseMintInstr.Instruction,
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
					payKeyResp.KeyPair.PrivateKey,  // payer signature
					mintKeyResp.KeyPair.PrivateKey, // mint account signature
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

	// Ensure mint account visible before parsing
	suite.waitForAccountVisible(submittedTx.Signature, mintKeyResp.KeyPair.PublicKey)

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

// Test_02_5_GetCurrentMinRentForHoldingAccount tests rent calculation for holding accounts
func (suite *TokenProgramE2ETestSuite) Test_02_5_GetCurrentMinRentForHoldingAccount() {
	suite.T().Log("🎯 Testing Holding Account Rent Calculation")

	// Get rent for holding account using our new method
	resp, err := suite.tokenProgramService.GetCurrentMinRentForHoldingAccount(suite.ctx, &token_v1.GetCurrentMinRentForHoldingAccountRequest{})
	suite.Require().NoError(err, "Should get holding account rent successfully")
	suite.Require().NotZero(resp.Lamports, "Holding account rent should not be zero")

	// Validate reasonable rent amount (holding accounts are 165 bytes)
	suite.Assert().Greater(resp.Lamports, uint64(1_000_000), "Rent should be at least 1M lamports for holding account")
	suite.T().Logf("  Holding account rent: %d lamports", resp.Lamports)
}

// Test_02_6_InitialiseHoldingAccountInstruction tests holding account instruction creation
func (suite *TokenProgramE2ETestSuite) Test_02_6_InitialiseHoldingAccountInstruction() {
	suite.T().Log("🎯 Testing InitialiseHoldingAccount Instruction Creation")

	// Use hardcoded valid public keys for instruction creation test
	testAccountPubKey := "11111111111111111111111111111112"          // System Program
	testMintPubKey := "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"  // Token 2022 Program
	testOwnerPubKey := "So11111111111111111111111111111111111111112" // Wrapped SOL

	// Create holding account instruction using memo transfer configuration
	resp, err := suite.tokenProgramService.InitialiseHoldingAccount(suite.ctx, &token_v1.InitialiseHoldingAccountRequest{
		AccountPubKey:      testAccountPubKey,
		MintPubKey:         testMintPubKey,
		OwnerPubKey:        testOwnerPubKey,
		MemoTransferConfig: &token_v1.MemoTransferConfig{RequireIncomingMemo: true},
	})
	suite.Require().NoError(err, "Should create holding account instruction successfully")
	suite.Require().NotNil(resp.Instruction, "Instruction should not be nil")
	suite.Require().Len(resp.Instructions, 2, "Should include initialise and memo-enable instructions")

	suite.Assert().Equal(resp.Instruction.ProgramId, resp.Instructions[0].ProgramId, "First instruction should match legacy field")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, resp.Instructions[1].ProgramId, "Memo enable instruction should target Token 2022 program")
	suite.Assert().Greater(len(resp.Instructions[1].Data), 0, "Memo enable instruction should have non-empty data")

	suite.T().Logf("✅ InitialiseHoldingAccount returned %d instructions (memo enabled)", len(resp.Instructions))

	// Validate default behaviour when memo config is omitted
	defaultResp, err := suite.tokenProgramService.InitialiseHoldingAccount(suite.ctx, &token_v1.InitialiseHoldingAccountRequest{
		AccountPubKey: testAccountPubKey,
		MintPubKey:    testMintPubKey,
		OwnerPubKey:   testOwnerPubKey,
	})
	suite.Require().NoError(err, "Should create holding account instruction without memo config")
	suite.Require().NotNil(defaultResp.Instruction, "Instruction should not be nil for default response")
	suite.Require().Len(defaultResp.Instructions, 1, "Default response should only contain initialise instruction")
}

// Test_03_Token_e2e tests complete mint + holding account creation flow
func (suite *TokenProgramE2ETestSuite) Test_03_Token_e2e() {
	suite.T().Log("🎯 Testing Token 2022 Mint Creation and Holding Account Initialization")

	// Generate and fund payer account
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
	suite.waitForAccountVisible(fundResp.GetSignature(), payKeyResp.KeyPair.PublicKey)

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
		Decimals:              6,
	})
	suite.Require().NoError(err, "Should create initialise mint instruction")

	// Generate holding account keypair
	holdingAccKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate holding account keypair")
	suite.T().Logf("  Generated holding account: %s", holdingAccKeyResp.KeyPair.PublicKey)

	// Get baseline rent for holding account
	holdingAccountRentResp, err := suite.tokenProgramService.GetCurrentMinRentForHoldingAccount(suite.ctx, &token_v1.GetCurrentMinRentForHoldingAccountRequest{})
	suite.Require().NoError(err, "Should get current rent amount for token holding account")
	suite.T().Logf("  Holding account rent: %d lamports", holdingAccountRentResp.Lamports)

	// Get memo-enabled rent for holding account
	holdingRentWithMemo, err := suite.tokenProgramService.GetCurrentMinRentForHoldingAccount(suite.ctx, &token_v1.GetCurrentMinRentForHoldingAccountRequest{
		MemoTransferConfig: &token_v1.MemoTransferConfig{RequireIncomingMemo: true},
	})
	suite.Require().NoError(err, "Should get memo-enabled holding account rent")
	suite.Assert().Greater(holdingRentWithMemo.Lamports, holdingAccountRentResp.Lamports, "Memo-enabled rent should exceed baseline")
	suite.T().Logf("  Holding account rent with memo: %d lamports", holdingRentWithMemo.Lamports)

	// Build holding account instructions (system create + initialise + memo enable)
	createHoldingAccountResp, err := suite.tokenProgramService.CreateHoldingAccount(suite.ctx, &token_v1.CreateHoldingAccountRequest{
		Payer:                payKeyResp.KeyPair.PublicKey,
		NewAccount:           holdingAccKeyResp.KeyPair.PublicKey,
		HoldingAccountPubKey: holdingAccKeyResp.KeyPair.PublicKey,
		MintPubKey:           mintKeyResp.KeyPair.PublicKey,
		OwnerPubKey:          payKeyResp.KeyPair.PublicKey,
		MemoTransferConfig:   &token_v1.MemoTransferConfig{RequireIncomingMemo: true},
	})
	suite.Require().NoError(err, "Should create holding account instruction bundle")
	suite.Require().Len(createHoldingAccountResp.Instructions, 3, "CreateHoldingAccount should include create + initialise + memo instructions")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, createHoldingAccountResp.Instructions[2].ProgramId, "Third instruction should enable memo transfers")
	suite.Require().GreaterOrEqual(len(createHoldingAccountResp.Instructions[0].Data), 20, "Create instruction should encode header, lamports, and space")
	instructionData := createHoldingAccountResp.Instructions[0].Data
	const systemInstructionHeaderBytes = 4
	memoLamports := binary.LittleEndian.Uint64(instructionData[systemInstructionHeaderBytes : systemInstructionHeaderBytes+8])
	suite.Require().EqualValues(holdingRentWithMemo.Lamports, memoLamports, "Lamports in create instruction should match memo rent")
	memoAccountSpace := int(binary.LittleEndian.Uint64(instructionData[systemInstructionHeaderBytes+8 : systemInstructionHeaderBytes+16]))
	suite.Require().Greater(memoAccountSpace, int(token_v1.HOLDING_ACCOUNT_LEN), "Memo-enabled account should allocate additional space")
	suite.T().Logf("  Memo-enabled holding account space: %d bytes", memoAccountSpace)

	// Compose atomic transaction with mint + holding account instructions (including memo enable)
	atomicTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createMintInstr,
			initialiseMintInstr.Instruction,
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}
	atomicTx.Instructions = append(atomicTx.Instructions, createHoldingAccountResp.Instructions...)
	suite.T().Logf("  Composed atomic transaction with %d instructions", len(atomicTx.Instructions))

	// Execute transaction lifecycle (compile, sign, submit)
	compiledTx, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: atomicTx,
		FeePayer:    payKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Sign transaction (needs both mint and holding account signatures)
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

	// Ensure mint and holding accounts are visible before parsing and fetching
	suite.waitForAccountVisible(submittedTx.Signature, mintKeyResp.KeyPair.PublicKey)
	suite.waitForAccountVisible(submittedTx.Signature, holdingAccKeyResp.KeyPair.PublicKey)

	// Verify mint account parsing
	parsedMint, err := suite.tokenProgramService.ParseMint(suite.ctx, &token_v1.ParseMintRequest{
		AccountAddress: mintKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should parse mint account")
	suite.Require().NotNil(parsedMint.Mint, "Parsed mint should not be nil")

	// Validate mint properties
	suite.Assert().Equal(uint32(6), parsedMint.Mint.Decimals, "Mint should have 6 decimals")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.MintAuthorityPubKey, "Mint authority should match")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.FreezeAuthorityPubKey, "Freeze authority should match")
	suite.Assert().Equal("0", parsedMint.Mint.Supply, "Initial supply should be zero")
	suite.Assert().True(parsedMint.Mint.IsInitialized, "Mint should be initialized")

	// Verify holding account creation (ensure it exists and is owned by token program)
	holdingAccount, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
		Address:         holdingAccKeyResp.KeyPair.PublicKey,
		CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
	})
	suite.Require().NoError(err, "Should get holding account")
	suite.Require().NotNil(holdingAccount, "Holding account should exist")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, holdingAccount.Owner, "Holding account should be owned by Token 2022 program")
	suite.Require().NotEmpty(holdingAccount.Data, "Holding account should have data")
	decodedData := decodeAccountDataBytes(suite, holdingAccount.Data)
	suite.Assert().Equal(memoAccountSpace, len(decodedData), "Holding account data length should match memo-enabled space")

	// BUILD INSTRUCTION to mint tokens into the holding account
	mintAmount := "1000000" // 1 token with 6 decimals
	mintInstr, err := suite.tokenProgramService.Mint(suite.ctx, &token_v1.MintRequest{
		MintPubKey:               mintKeyResp.KeyPair.PublicKey,
		DestinationAccountPubKey: holdingAccKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:      payKeyResp.KeyPair.PublicKey, // payer is the mint authority
		Amount:                   mintAmount,
		Decimals:                 6, // Must match mint decimals
	})
	suite.Require().NoError(err, "Should create mint instruction")
	suite.T().Logf("  Created mint instruction for %s tokens", mintAmount)

	// Compose atomic transaction with minting instruction
	mintTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			mintInstr.Instruction, // Mint tokens to holding account
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}
	suite.T().Logf("  Composed mint transaction")

	// Execute mint transaction lifecycle (compile, sign, submit)
	compiledMintTx, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: mintTx,
		FeePayer:    payKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should compile mint transaction")

	// Sign mint transaction (only needs mint authority signature)
	signedMintTx, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compiledMintTx.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{
					payKeyResp.KeyPair.PrivateKey, // mint authority signature
				},
			},
		},
	})
	suite.Require().NoError(err, "Should sign mint transaction")

	// Submit mint transaction
	submittedMintTx, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction: signedMintTx.Transaction,
	})
	suite.Require().NoError(err, "Should submit mint transaction")
	suite.T().Logf("  Mint transaction submitted: %s", submittedMintTx.Signature)

	// Wait for mint transaction confirmation (ensures account data updates)
	suite.monitorTransactionToCompletion(submittedMintTx.Signature)

	// Verify tokens were minted by checking holding account after minting
	holdingAccountAfterMint, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
		Address:         holdingAccKeyResp.KeyPair.PublicKey,
		CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
	})
	suite.Require().NoError(err, "Should get holding account after minting")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, holdingAccountAfterMint.Owner, "Holding account should still be owned by Token 2022 program")
	suite.Require().NotEmpty(holdingAccountAfterMint.Data, "Holding account should have updated data after minting")
	memoDecodedData := decodeAccountDataBytes(suite, holdingAccountAfterMint.Data)
	suite.Assert().Equal(memoAccountSpace, len(memoDecodedData), "Holding account data length should remain memo-enabled size")

	// Verify mint supply has increased
	var parsedMintAfterMinting *token_v1.ParseMintResponse
	for attempt := 1; attempt <= 10; attempt++ {
		parsedMintAfterMinting, err = suite.tokenProgramService.ParseMint(suite.ctx, &token_v1.ParseMintRequest{
			AccountAddress: mintKeyResp.KeyPair.PublicKey,
		})
		suite.Require().NoError(err, "Should parse mint account after minting (attempt %d)", attempt)

		if parsedMintAfterMinting != nil && parsedMintAfterMinting.Mint != nil && parsedMintAfterMinting.Mint.Supply == mintAmount {
			break
		}

		if attempt < 10 {
			time.Sleep(200 * time.Millisecond)
		}
	}
	suite.Require().NotNil(parsedMintAfterMinting, "ParseMint response should not be nil after minting")
	suite.Require().NotNil(parsedMintAfterMinting.Mint, "Parsed mint should not be nil after minting")
	suite.Assert().Equal(mintAmount, parsedMintAfterMinting.Mint.Supply, "Mint supply should match minted amount")

	suite.T().Logf("✅ Complete mint + holding account creation + minting verified successfully:")
	suite.T().Logf("   Mint Address: %s", mintKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   Mint Decimals: %d", parsedMint.Mint.Decimals)
	suite.T().Logf("   Mint Authority: %s", parsedMint.Mint.MintAuthorityPubKey)
	suite.T().Logf("   Mint Supply After Minting: %s", parsedMintAfterMinting.Mint.Supply)
	suite.T().Logf("   Holding Account Address: %s", holdingAccKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   Holding Account Owner: %s", holdingAccount.Owner)
	suite.T().Logf("   Holding Account Balance: %d lamports", holdingAccount.Lamports)
	suite.T().Logf("   Minted Amount: %s tokens", mintAmount)

	suite.T().Logf("🔍 Blockchain verification commands:")
	suite.T().Logf("   solana account %s --url http://localhost:8899", mintKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   solana account %s --url http://localhost:8899", holdingAccKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   spl-token account-info %s --url http://localhost:8899", holdingAccKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", submittedTx.Signature)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", submittedMintTx.Signature)
}

// Helper function to wait for account visibility
func (suite *TokenProgramE2ETestSuite) waitForAccountVisible(signature, address string) {
	if signature != "" {
		suite.monitorTransactionToCompletion(signature)
	}

	suite.T().Logf("  Waiting for account %s to become visible...", address)
	for attempt := 1; attempt <= 10; attempt++ {
		_, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
			Address:         address,
			CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
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
	suite.T().Logf("  Monitoring transaction %s for completion via streaming...", signature)

	stream, err := suite.transactionService.MonitorTransaction(suite.ctx, &transaction_v1.MonitorTransactionRequest{
		Signature:       signature,
		CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
		IncludeLogs:     false,
		TimeoutSeconds:  60,
	})
	suite.Require().NoError(err, "Must create monitoring stream for signature: %s", signature)

	confirmed := false
	for {
		resp, err := stream.Recv()
		if err == io.EOF {
			suite.Require().True(confirmed, "Stream ended without confirmation for signature: %s", signature)
			break
		}
		suite.Require().NoError(err, "Stream must not error for signature: %s", signature)

		suite.T().Logf("  Transaction %s status: %v", signature, resp.Status)

		if resp.Status == transaction_v1.TransactionStatus_TRANSACTION_STATUS_CONFIRMED ||
			resp.Status == transaction_v1.TransactionStatus_TRANSACTION_STATUS_FINALIZED {
			confirmed = true
			suite.T().Logf("  ✅ Transaction %s successfully confirmed", signature)
			break
		}

		if resp.Status == transaction_v1.TransactionStatus_TRANSACTION_STATUS_FAILED {
			suite.Require().Fail("Transaction FAILED", "Transaction %s failed with error: %s", signature, resp.GetErrorMessage())
			return
		}

		if resp.Status == transaction_v1.TransactionStatus_TRANSACTION_STATUS_TIMEOUT {
			suite.Require().Fail("Transaction TIMED OUT", "Transaction %s monitoring timed out", signature)
			return
		}

		if resp.Status == transaction_v1.TransactionStatus_TRANSACTION_STATUS_DROPPED {
			suite.Require().Fail("Transaction DROPPED", "Transaction %s was dropped by network", signature)
			return
		}
	}

	suite.Require().True(confirmed, "Transaction %s must reach CONFIRMED or FINALIZED status", signature)
}

func decodeAccountDataBytes(s *TokenProgramE2ETestSuite, raw string) []byte {
	var numericPayload []int
	if err := json.Unmarshal([]byte(raw), &numericPayload); err == nil && len(numericPayload) > 0 {
		bytes := make([]byte, len(numericPayload))
		for i, v := range numericPayload {
			s.Require().GreaterOrEqual(v, 0, "account data byte values must be non-negative")
			s.Require().LessOrEqual(v, 255, "account data byte values must be within byte range")
			bytes[i] = byte(v)
		}
		return bytes
	}

	var tuplePayload []any
	if err := json.Unmarshal([]byte(raw), &tuplePayload); err == nil && len(tuplePayload) == 2 {
		if encoded, ok := tuplePayload[0].(string); ok {
			decoded, err := base64.StdEncoding.DecodeString(encoded)
			s.Require().NoError(err, "Should decode base64 account payload")
			return decoded
		}
	}

	s.Require().Failf("decodeAccountDataBytes", "Unsupported account data format: %s", raw)
	return nil
}

func TestTokenProgramE2ESuite(t *testing.T) {
	suite.Run(t, new(TokenProgramE2ETestSuite))
}
