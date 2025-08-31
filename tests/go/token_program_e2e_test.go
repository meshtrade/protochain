package apitest

import (
	"context"
	"testing"
	"time"

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
	_, err = suite.accountService.FundNative(suite.ctx, &account_v1.FundNativeRequest{
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
			CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED.Enum(),
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