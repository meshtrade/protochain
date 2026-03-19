package apitest

import (
	"context"
	"fmt"
	"io"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"

	account_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/account/v1"
	system_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/program/system/v1"
	token_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/program/token/v1"
	transaction_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/transaction/v1"
	type_v1 "github.com/meshtrade/protochain/lib/go/protochain/solana/type/v1"
	"github.com/meshtrade/protochain/tests/go/config"
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

	conf, err := config.GetConfig("config.json")
	suite.Require().NoError(err, "Failed to get config")

	// Setup configuration
	grpcEndpoint := fmt.Sprintf("%s:%d", conf.BackendGRPCEndpoint, conf.BackendGRPCPort)

	// Connect to gRPC server
	var dialOpts []grpc.DialOption
	if conf.BackendGRPCTLS {
		dialOpts = append(dialOpts, grpc.WithTransportCredentials(credentials.NewClientTLSFromCert(nil, "")))
	} else {
		dialOpts = append(dialOpts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	suite.grpcConn, err = grpc.NewClient(grpcEndpoint, dialOpts...)
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

// Test_01_InitialiseMint_TOKEN2022 tests Token-2022 mint creation with metadata extension
func (suite *TokenProgramE2ETestSuite) Test_01_InitialiseMint_TOKEN2022() {
	suite.T().Log("🎯 Testing Token 2022 Mint Creation with Metadata Extension")

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

	// Wait for airdrop confirmation via websocket monitoring (bootstrap only)
	suite.monitorTransactionToCompletion(fundResp.GetSignature())

	// Generate mint account keypair
	mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate mint keypair")
	suite.T().Logf("  Generated mint account: %s", mintKeyResp.KeyPair.PublicKey)

	// Define metadata extension
	metadataExtension := &token_v1.Token2022Extension{
		Extension: &token_v1.Token2022Extension_Metadata{
			Metadata: &token_v1.Token2022ExtensionMetadata{
				Name:   "Test Token",
				Symbol: "TST",
				Uri:    "https://example.com/metadata.json",
				AdditionalMetadata: map[string]string{
					"description": "A test token with metadata",
				},
			},
		},
	}

	// Get current rent for Token-2022 mint account WITH metadata extension
	rentResp, err := suite.tokenProgramService.GetCurrentMinRentForToken2022MintAccount(suite.ctx, &token_v1.GetCurrentMinRentForToken2022MintAccountRequest{
		Extensions: []*token_v1.Token2022Extension{metadataExtension},
	})
	suite.Require().NoError(err, "Should get current rent amount")
	suite.Require().NotZero(rentResp.Space, "Space should not be zero")
	suite.Assert().Greater(rentResp.Space, uint64(token_v1.MINT_ACCOUNT_LEN), "Space with metadata should exceed base mint size")
	suite.T().Logf("  Rent required for mint with metadata: %d lamports (space: %d bytes)", rentResp.Lamports, rentResp.Space)

	// Create mint account instruction using space from rent response
	createMintInstr, err := suite.systemProgramService.Create(suite.ctx, &system_v1.CreateRequest{
		Payer:      payKeyResp.KeyPair.PublicKey,
		NewAccount: mintKeyResp.KeyPair.PublicKey,
		Owner:      token_v1.TOKEN_2022_PROGRAM_ID,
		Lamports:   rentResp.Lamports,
		Space:      rentResp.Space,
	})
	suite.Require().NoError(err, "Should create mint account instruction")

	// Initialize mint instruction with metadata extension
	initialiseMintResp, err := suite.tokenProgramService.InitialiseToken2022Mint(suite.ctx, &token_v1.InitialiseToken2022MintRequest{
		MintPubKey:            mintKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
		FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
		Decimals:              2,
		Extensions:            []*token_v1.Token2022Extension{metadataExtension},
	})
	suite.Require().NoError(err, "Should create initialise mint instructions")
	// With metadata: metadata_pointer_init + initialize_mint + token_metadata_init + update_field(description)
	suite.Require().Len(initialiseMintResp.Instructions, 4,
		"Should return 4 instructions: metadata_pointer_init, initialize_mint, token_metadata_init, update_field")
	suite.T().Logf("  InitialiseMint returned %d instructions", len(initialiseMintResp.Instructions))

	// Compose atomic transaction: system create + all init instructions
	instructions := []*transaction_v1.SolanaInstruction{createMintInstr.Instruction}
	instructions = append(instructions, initialiseMintResp.Instructions...)

	atomicTx := &transaction_v1.Transaction{
		Instructions: instructions,
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
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
	suite.T().Logf("  Transaction submitted: signature=%s result=%v error=%s", submittedTx.Signature, submittedTx.SubmissionResult, submittedTx.ErrorMessage)
	suite.Require().NotEmpty(submittedTx.Signature, "Submission must return a transaction signature")

	// Monitor transaction to confirmation via websocket before reading account state
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

	// Verify token_program is Token-2022
	suite.Assert().Equal(type_v1.TokenProgram_TOKEN_PROGRAM_2022, parsedMint.TokenProgram,
		"Token program should be TOKEN_PROGRAM_2022")

	// Verify extensions match the metadata configured during initialization
	suite.Require().Len(parsedMint.Extensions, 1, "Should have exactly 1 extension (Metadata)")
	metaExt := parsedMint.Extensions[0].GetMetadata()
	suite.Require().NotNil(metaExt, "Extension should be Metadata type")
	suite.Assert().Equal("Test Token", metaExt.Name, "Metadata name should match")
	suite.Assert().Equal("TST", metaExt.Symbol, "Metadata symbol should match")
	suite.Assert().Equal("https://example.com/metadata.json", metaExt.Uri, "Metadata URI should match")
	suite.Assert().Equal(mintKeyResp.KeyPair.PublicKey, metaExt.MetadataAddress,
		"Metadata address should be the mint itself (self-referencing)")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, metaExt.UpdateAuthorityPubKey,
		"Update authority should default to mint authority")
	suite.Require().Contains(metaExt.AdditionalMetadata, "description",
		"Additional metadata should contain 'description' key")
	suite.Assert().Equal("A test token with metadata", metaExt.AdditionalMetadata["description"],
		"Additional metadata 'description' value should match")

	suite.T().Logf("✅ Token-2022 Mint with metadata created and verified successfully:")
	suite.T().Logf("   Mint Address: %s", mintKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   Decimals: %d", parsedMint.Mint.Decimals)
	suite.T().Logf("   Authority: %s", parsedMint.Mint.MintAuthorityPubKey)
	suite.T().Logf("   Supply: %s", parsedMint.Mint.Supply)
}

// Test_02_InitialiseMint_SPL tests legacy SPL Token mint creation
func (suite *TokenProgramE2ETestSuite) Test_02_InitialiseMint_SPL() {
	suite.T().Log("🎯 Testing Legacy SPL Token Mint Creation")

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

	// Wait for airdrop confirmation via websocket monitoring (bootstrap only)
	suite.monitorTransactionToCompletion(fundResp.GetSignature())

	// Generate mint account keypair
	mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate mint keypair")
	suite.T().Logf("  Generated mint account: %s", mintKeyResp.KeyPair.PublicKey)

	// SPL legacy mint: fixed 82 bytes, no extensions
	rentResp, err := suite.tokenProgramService.GetCurrentMinRentForSPLTokenMintAccount(suite.ctx, &token_v1.GetCurrentMinRentForSPLTokenMintAccountRequest{})
	suite.Require().NoError(err, "Should get current rent amount")
	suite.T().Logf("  Rent required for SPL mint: %d lamports", rentResp.Lamports)

	// Create mint account instruction with SPL Token program as owner
	createMintInstr, err := suite.systemProgramService.Create(suite.ctx, &system_v1.CreateRequest{
		Payer:      payKeyResp.KeyPair.PublicKey,
		NewAccount: mintKeyResp.KeyPair.PublicKey,
		Owner:      token_v1.SPL_TOKEN_PROGRAM_ID,
		Lamports:   rentResp.Lamports,
		Space:      token_v1.MINT_ACCOUNT_LEN,
	})
	suite.Require().NoError(err, "Should create SPL mint account instruction")

	// Initialize mint instruction for legacy SPL Token program with Metaplex metadata
	initialiseMintResp, err := suite.tokenProgramService.InitialiseSPLTokenMint(suite.ctx, &token_v1.InitialiseSPLTokenMintRequest{
		MintPubKey:            mintKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
		FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
		Decimals:              6,
		PayerPubKey:           payKeyResp.KeyPair.PublicKey,
		Metadata: &token_v1.MetaplexTokenMetadata{
			Name:                 "Test SPL Token",
			Symbol:               "TSPL",
			Uri:                  "https://example.com/spl-metadata.json",
			SellerFeeBasisPoints: 0,
		},
	})
	suite.Require().NoError(err, "Should create SPL initialise mint instructions")
	suite.Require().Len(initialiseMintResp.Instructions, 2,
		"Should return 2 instructions: initialize_mint + create_metadata_account_v3")
	suite.T().Logf("  InitialiseSPLTokenMint returned %d instructions", len(initialiseMintResp.Instructions))

	// Compose atomic transaction: system create + init mint + create metadata
	instructions := []*transaction_v1.SolanaInstruction{createMintInstr.Instruction}
	instructions = append(instructions, initialiseMintResp.Instructions...)
	atomicTx := &transaction_v1.Transaction{
		Instructions: instructions,
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
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

	// Monitor transaction to confirmation via websocket before reading account state
	suite.monitorTransactionToCompletion(submittedTx.Signature)

	// Verify mint creation by parsing the account
	parsedMint, err := suite.tokenProgramService.ParseMint(suite.ctx, &token_v1.ParseMintRequest{
		AccountAddress: mintKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should parse SPL mint account")
	suite.Require().NotNil(parsedMint.Mint, "Parsed mint should not be nil")

	// Validate mint properties
	suite.Assert().Equal(uint32(6), parsedMint.Mint.Decimals, "Mint should have 6 decimals")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.MintAuthorityPubKey, "Mint authority should match")
	suite.Assert().Equal(payKeyResp.KeyPair.PublicKey, parsedMint.Mint.FreezeAuthorityPubKey, "Freeze authority should match")
	suite.Assert().Equal("0", parsedMint.Mint.Supply, "Initial supply should be zero")
	suite.Assert().True(parsedMint.Mint.IsInitialized, "Mint should be initialized")

	// Verify token_program is Legacy SPL Token
	suite.Assert().Equal(type_v1.TokenProgram_TOKEN_PROGRAM_LEGACY, parsedMint.TokenProgram,
		"Token program should be TOKEN_PROGRAM_LEGACY")

	// Legacy mints should have no extensions (metadata is stored in a separate PDA, not on the mint)
	suite.Assert().Empty(parsedMint.Extensions, "Legacy SPL mint should have no extensions")

	suite.T().Logf("✅ Legacy SPL Token Mint with Metaplex metadata created and verified successfully:")
	suite.T().Logf("   Mint Address: %s", mintKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   Decimals: %d", parsedMint.Mint.Decimals)
	suite.T().Logf("   Authority: %s", parsedMint.Mint.MintAuthorityPubKey)
	suite.T().Logf("   Supply: %s", parsedMint.Mint.Supply)
	suite.T().Logf("   Metaplex Metadata: name=\"Test SPL Token\" symbol=\"TSPL\"")
}

// Test_03_GetCurrentMinRentForMintAccount tests rent calculation for both Token-2022 and SPL Token
func (suite *TokenProgramE2ETestSuite) Test_03_GetCurrentMinRentForMintAccount() {
	suite.T().Log("🎯 Testing Token Account Rent Calculation")

	// Test Token-2022 mint rent (no extensions)
	token2022Resp, err := suite.tokenProgramService.GetCurrentMinRentForToken2022MintAccount(suite.ctx, &token_v1.GetCurrentMinRentForToken2022MintAccountRequest{})
	suite.Require().NoError(err, "Should get Token-2022 rent successfully")
	suite.Require().NotZero(token2022Resp.Lamports, "Token-2022 rent should not be zero")
	suite.Assert().Greater(token2022Resp.Lamports, uint64(1_000_000), "Token-2022 rent should be at least 1M lamports")
	suite.T().Logf("  Token-2022 mint account rent (no extensions): %d lamports (space: %d bytes)", token2022Resp.Lamports, token2022Resp.Space)

	// Test Token-2022 mint rent with metadata extension
	token2022MetaResp, err := suite.tokenProgramService.GetCurrentMinRentForToken2022MintAccount(suite.ctx, &token_v1.GetCurrentMinRentForToken2022MintAccountRequest{
		Extensions: []*token_v1.Token2022Extension{{
			Extension: &token_v1.Token2022Extension_Metadata{
				Metadata: &token_v1.Token2022ExtensionMetadata{
					Name:   "Test Token",
					Symbol: "TST",
					Uri:    "https://example.com/metadata.json",
				},
			},
		}},
	})
	suite.Require().NoError(err, "Should get Token-2022 metadata rent successfully")
	suite.Assert().Greater(token2022MetaResp.Space, token2022Resp.Space, "Token-2022 with metadata should require more space")
	suite.T().Logf("  Token-2022 mint account rent (with metadata): %d lamports (space: %d bytes)", token2022MetaResp.Lamports, token2022MetaResp.Space)

	// Test SPL Token mint rent
	splResp, err := suite.tokenProgramService.GetCurrentMinRentForSPLTokenMintAccount(suite.ctx, &token_v1.GetCurrentMinRentForSPLTokenMintAccountRequest{})
	suite.Require().NoError(err, "Should get SPL Token rent successfully")
	suite.Require().NotZero(splResp.Lamports, "SPL Token rent should not be zero")
	suite.Assert().Equal(uint64(token_v1.MINT_ACCOUNT_LEN), splResp.Space, "SPL Token mint should be exactly MINT_ACCOUNT_LEN bytes")
	suite.Assert().Greater(splResp.Lamports, uint64(1_000_000), "SPL Token rent should be at least 1M lamports")
	suite.T().Logf("  SPL Token mint account rent: %d lamports (space: %d bytes)", splResp.Lamports, splResp.Space)
}

// Test_03_5_GetCurrentMinRentForHoldingAccount tests rent calculation for holding accounts
func (suite *TokenProgramE2ETestSuite) Test_03_5_GetCurrentMinRentForHoldingAccount() {
	suite.T().Log("🎯 Testing Holding Account Rent Calculation")

	// Get rent for holding account using our new method
	resp, err := suite.tokenProgramService.GetCurrentMinRentForHoldingAccount(suite.ctx, &token_v1.GetCurrentMinRentForHoldingAccountRequest{})
	suite.Require().NoError(err, "Should get holding account rent successfully")
	suite.Require().NotZero(resp.Lamports, "Holding account rent should not be zero")

	// Validate reasonable rent amount (holding accounts are 165 bytes)
	suite.Assert().Greater(resp.Lamports, uint64(1_000_000), "Rent should be at least 1M lamports for holding account")
	suite.T().Logf("  Holding account rent: %d lamports", resp.Lamports)
}

// Test_03_6_InitialiseHoldingAccountInstruction tests holding account instruction creation
func (suite *TokenProgramE2ETestSuite) Test_03_6_InitialiseHoldingAccountInstruction() {
	suite.T().Log("🎯 Testing InitialiseHoldingAccount Instruction Creation")

	// Use hardcoded valid public keys for instruction creation test
	testPayerAccountKey := "11111111111111111111111111111113"
	testMintPubKey := "11111111111111111111111111111114"
	testOwnerPubKey := "So11111111111111111111111111111111111111112" // Wrapped SOL

	// Create holding account instruction using memo transfer configuration
	resp, err := suite.tokenProgramService.CreateHoldingAccount(suite.ctx, &token_v1.CreateHoldingAccountRequest{
		Payer:              testPayerAccountKey,
		MintPubKey:         testMintPubKey,
		OwnerPubKey:        testOwnerPubKey,
		TokenProgram:       type_v1.TokenProgram_TOKEN_PROGRAM_2022,
		MemoTransferConfig: &token_v1.MemoTransferConfig{RequireIncomingMemo: true},
	})
	suite.Require().NoError(err, "Should create holding account instruction successfully")
	suite.Require().NotNil(resp.Instructions, "Instruction should not be nil")
	suite.Require().Len(resp.Instructions, 3, "Should include create, initialise and memo-enable instructions")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, resp.Instructions[2].ProgramId, "Memo enable instruction should target Token 2022 program")
	suite.Assert().Greater(len(resp.Instructions[1].Data), 0, "Memo enable instruction should have non-empty data")

	suite.T().Logf("✅ InitialiseHoldingAccount returned %d instructions (memo enabled)", len(resp.Instructions))

	// Validate default behaviour when memo config is omitted
	defaultResp, err := suite.tokenProgramService.CreateHoldingAccount(suite.ctx, &token_v1.CreateHoldingAccountRequest{
		Payer:        testPayerAccountKey,
		MintPubKey:   testMintPubKey,
		OwnerPubKey:  testOwnerPubKey,
		TokenProgram: type_v1.TokenProgram_TOKEN_PROGRAM_2022,
	})
	suite.Require().NoError(err, "Should create holding account instruction without memo config")
	suite.Require().NotNil(defaultResp.Instructions, "Instruction should not be nil for default response")
	suite.Require().Len(defaultResp.Instructions, 1, "Default response should only contain create instruction")
}

func (suite *TokenProgramE2ETestSuite) Test_04_Mint_e2e() {
	suite.T().Log("🎯 Testing Token 2022 Mint Creation")

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

	// Wait for airdrop confirmation via websocket monitoring (bootstrap only)
	suite.monitorTransactionToCompletion(fundResp.GetSignature())

	// Generate mint account keypair
	mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate mint keypair")
	suite.T().Logf("  Generated mint account: %s", mintKeyResp.KeyPair.PublicKey)

	// Get current rent for Token-2022 mint account (no extensions)
	rentResp, err := suite.tokenProgramService.GetCurrentMinRentForToken2022MintAccount(suite.ctx, &token_v1.GetCurrentMinRentForToken2022MintAccountRequest{})
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

	// Initialize mint instructions (token program)
	initialiseMintResp, err := suite.tokenProgramService.InitialiseToken2022Mint(suite.ctx, &token_v1.InitialiseToken2022MintRequest{
		MintPubKey:            mintKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
		FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
		Decimals:              6,
	})
	suite.Require().NoError(err, "Should create initialise mint instructions")

	// Compose atomic transaction: system create + all init instructions
	instructions := []*transaction_v1.SolanaInstruction{createMintInstr.Instruction}
	instructions = append(instructions, initialiseMintResp.Instructions...)
	atomicTx := &transaction_v1.Transaction{
		Instructions: instructions,
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}
	suite.T().Logf("  Composed atomic transaction with %d instructions", len(atomicTx.Instructions))

	// Execute transaction lifecycle (compile, sign, submit)
	compiledTx, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: atomicTx,
		FeePayer:    payKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Sign transaction (payer for fees and mint creation; ATA is derived so doesn't sign)
	signedTx, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compiledTx.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{
					payKeyResp.KeyPair.PrivateKey,  // payer signature for fees
					mintKeyResp.KeyPair.PrivateKey, // mint account signature (system Create requires this)
				},
			},
		},
	})
	suite.Require().NoError(err, "Should sign transaction")

	// Submit transaction
	suite.T().Logf("  Signed transaction state: %v", signedTx.Transaction.State)
	suite.T().Logf("  Signed transaction instructions count: %d", len(signedTx.Transaction.Instructions))
	submittedTx, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction: signedTx.Transaction,
	})
	suite.Require().NoError(err, "Should submit transaction")
	suite.Require().NotEmpty(submittedTx.Signature, "Transaction signature should not be empty (error_message: %s)", submittedTx.ErrorMessage)
	suite.T().Logf("  Transaction submitted: %s", submittedTx.Signature)

	// Monitor transaction to confirmation via websocket before reading account state
	suite.monitorTransactionToCompletion(submittedTx.Signature)

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
}

// Test_05_Token_e2e tests complete mint + holding account creation flow
func (suite *TokenProgramE2ETestSuite) Test_05_Token_e2e() {
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

	// Wait for airdrop confirmation via websocket monitoring (bootstrap only)
	suite.monitorTransactionToCompletion(fundResp.GetSignature())

	/*												Mint 												*/
	// Generate mint account keypair
	mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate mint keypair")
	suite.T().Logf("  Generated mint account: %s", mintKeyResp.KeyPair.PublicKey)

	// Get current rent for Token-2022 mint account (no extensions)
	rentResp, err := suite.tokenProgramService.GetCurrentMinRentForToken2022MintAccount(suite.ctx, &token_v1.GetCurrentMinRentForToken2022MintAccountRequest{})
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

	// Initialize mint instructions (token program)
	initialiseMintResp, err := suite.tokenProgramService.InitialiseToken2022Mint(suite.ctx, &token_v1.InitialiseToken2022MintRequest{
		MintPubKey:            mintKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
		FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
		Decimals:              6,
	})
	suite.Require().NoError(err, "Should create initialise mint instructions")

	mintInstructions := []*transaction_v1.SolanaInstruction{createMintInstr.Instruction}
	mintInstructions = append(mintInstructions, initialiseMintResp.Instructions...)
	createMintTxn := &transaction_v1.Transaction{
		Instructions: mintInstructions,
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}

	// Execute transaction lifecycle (compile, sign, submit)
	compiledCreateMintTxn, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: createMintTxn,
		FeePayer:    payKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Sign transaction (payer for fees and mint creation; ATA is derived so doesn't sign)
	signedCreateMintTxn, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compiledCreateMintTxn.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{
					payKeyResp.KeyPair.PrivateKey,  // payer signature for fees
					mintKeyResp.KeyPair.PrivateKey, // mint account signature (system Create requires this)
				},
			},
		},
	})
	suite.Require().NoError(err, "Should sign transaction")

	// Submit transaction
	suite.T().Logf("  Signed transaction state: %v", signedCreateMintTxn.Transaction.State)
	suite.T().Logf("  Signed transaction instructions count: %d", len(signedCreateMintTxn.Transaction.Instructions))
	submittedTx, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction: signedCreateMintTxn.Transaction,
	})
	suite.Require().NoError(err, "Should submit transaction")
	suite.Require().NotEmpty(submittedTx.Signature, "Transaction signature should not be empty (error_message: %s)", submittedTx.ErrorMessage)
	suite.T().Logf("  Transaction submitted: %s", submittedTx.Signature)

	// Monitor transaction to confirmation via websocket before reading account state
	suite.monitorTransactionToCompletion(submittedTx.Signature)

	/*											Holding Account 									*/
	// Generate holding account keypair
	walletAccKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate wallet account keypair")
	suite.T().Logf("  Generated wallet account: %s", walletAccKeyResp.KeyPair.PublicKey)

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

	// Build holding account instructions with memo transfer enabled
	createHoldingAccountResp, err := suite.tokenProgramService.CreateHoldingAccount(suite.ctx, &token_v1.CreateHoldingAccountRequest{
		Payer:              payKeyResp.KeyPair.PublicKey,
		MintPubKey:         mintKeyResp.KeyPair.PublicKey,
		OwnerPubKey:        walletAccKeyResp.KeyPair.PublicKey,
		MemoTransferConfig: &token_v1.MemoTransferConfig{RequireIncomingMemo: true},
		TokenProgram:       type_v1.TokenProgram_TOKEN_PROGRAM_2022,
	})
	suite.Require().NoError(err, "Should create holding account instruction bundle")
	// With memo: System Create + InitializeMemoTransfer
	suite.Require().Len(createHoldingAccountResp.Instructions, 3, "CreateHoldingAccount with memo should return creation, initialise and enable memo transfer config instructions")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, createHoldingAccountResp.Instructions[2].ProgramId, "Second instruction should initialize memo transfers")
	suite.T().Logf("  Created holding account with memo transfer (3 instructions)")

	// Compose atomic transaction with mint + holding account instructions (including memo enable)
	createHoldingAccountTxn := &transaction_v1.Transaction{
		Instructions: createHoldingAccountResp.Instructions,
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}
	suite.T().Logf("  Composed atomic transaction with %d instructions", len(createHoldingAccountTxn.Instructions))

	// Execute transaction lifecycle (compile, sign, submit)
	holdingAccountCompiledTxn, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: createHoldingAccountTxn,
		FeePayer:    payKeyResp.KeyPair.PublicKey,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Sign transaction (payer for fees and mint creation; ATA is derived so doesn't sign)
	signedCreateHoldingAccountTxn, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: holdingAccountCompiledTxn.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{
					payKeyResp.KeyPair.PrivateKey, // payer signature for fees
					walletAccKeyResp.KeyPair.PrivateKey,
				},
			},
		},
	})
	suite.Require().NoError(err, "Should sign transaction")

	// Submit transaction
	suite.T().Logf("  Signed transaction state: %v", signedCreateHoldingAccountTxn.Transaction.State)
	suite.T().Logf("  Signed transaction instructions count: %d", len(signedCreateHoldingAccountTxn.Transaction.Instructions))
	submittedTx, err = suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction: signedCreateHoldingAccountTxn.Transaction,
	})
	suite.Require().NoError(err, "Should submit transaction")
	suite.Require().NotEmpty(submittedTx.Signature, "Transaction signature should not be empty (error_message: %s)", submittedTx.ErrorMessage)
	suite.T().Logf("  Transaction submitted: %s", submittedTx.Signature)

	// determine the derived address
	ataAddressResp, err := suite.accountService.GetAssociatedTokenAddress(
		suite.ctx,
		&account_v1.GetAssociatedTokenAddressRequest{
			OwnerAddress: walletAccKeyResp.KeyPair.PublicKey,
			MintAddress:  mintKeyResp.KeyPair.PublicKey,
			TokenProgram: type_v1.TokenProgram_TOKEN_PROGRAM_2022,
		},
	)
	suite.Require().NoError(err, "Should get associated token account")

	// Monitor transaction to confirmation via websocket before reading account state
	suite.monitorTransactionToCompletion(submittedTx.Signature)

	// Verify holding account creation (ensure it exists and is owned by token program)
	holdingAccountResp, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
		Address:         ataAddressResp.Address,
		CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
	})
	suite.Require().NoError(err, "Should get holding account")
	suite.Require().NotNil(holdingAccountResp, "Holding account should exist")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, holdingAccountResp.Account.Owner, "Holding account should be owned by Token 2022 program")
	suite.Require().NotEmpty(holdingAccountResp.Account.Data, "Holding account should have data")

	// BUILD INSTRUCTION to mint tokens into the holding account
	mintAmount := "1000000" // 1 token with 6 decimals
	mintInstr, err := suite.tokenProgramService.Mint(suite.ctx, &token_v1.MintRequest{
		MintPubKey:               mintKeyResp.KeyPair.PublicKey,
		DestinationAccountPubKey: ataAddressResp.Address,       // mint into the token account
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
		Address:         ataAddressResp.Address,
		CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
	})
	suite.Require().NoError(err, "Should get holding account after minting")
	suite.Assert().Equal(token_v1.TOKEN_2022_PROGRAM_ID, holdingAccountAfterMint.Account.Owner, "Holding account should still be owned by Token 2022 program")
	suite.Require().NotEmpty(holdingAccountAfterMint.Account.Data, "Holding account should have updated data after minting")

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
	suite.T().Logf("   Mint Supply After Minting: %s", parsedMintAfterMinting.Mint.Supply)
	suite.T().Logf("   Holding Account Address: %s", walletAccKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   Holding Account Owner: %s", holdingAccountResp.Account.Owner)
	suite.T().Logf("   Holding Account Balance: %d lamports", holdingAccountResp.Account.Lamports)
	suite.T().Logf("   Minted Amount: %s tokens", mintAmount)

	suite.T().Logf("🔍 Blockchain verification commands:")
	suite.T().Logf("   solana account %s --url http://localhost:8899", mintKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   solana account %s --url http://localhost:8899", walletAccKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   spl-token account-info %s --url http://localhost:8899", walletAccKeyResp.KeyPair.PublicKey)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", submittedTx.Signature)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", submittedMintTx.Signature)
}

// monitorTransactionToCompletion monitors a transaction via websocket streaming
// until it reaches CONFIRMED or FINALIZED status, failing the test on error/timeout/drop.
func (suite *TokenProgramE2ETestSuite) monitorTransactionToCompletion(signature string) {
	suite.T().Logf("  Monitoring transaction %s for completion via streaming...", signature)

	stream, err := suite.transactionService.MonitorTransaction(suite.ctx, &transaction_v1.MonitorTransactionRequest{
		Signature:       signature,
		CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_FINALIZED,
		IncludeLogs:     false,
		TimeoutSeconds:  180,
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

func TestTokenProgramE2ESuite(t *testing.T) {
	suite.Run(t, new(TokenProgramE2ETestSuite))
}
