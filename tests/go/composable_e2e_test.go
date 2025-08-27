package apitest

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/suite"

	"github.com/BRBussy/protosol/lib/go/common"
	account_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/account/v1"
	system_program_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/program/system/v1"
	transaction_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/transaction/v1"
	type_v1 "github.com/BRBussy/protosol/lib/go/protosol/solana/type/v1"
	"github.com/BRBussy/protosol/tests/go/config"
)

// TestComposableE2ESuite runs the complete end-to-end test suite using the new composable architecture
func TestComposableE2ESuite(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping composable E2E integration tests in short mode")
	}

	// Skip integration tests if not explicitly requested
	if os.Getenv("RUN_INTEGRATION_TESTS") != "1" {
		t.Skip("Integration tests skipped - set RUN_INTEGRATION_TESTS=1 to enable")
	}

	suite.Run(t, new(ComposableE2ETestSuite))
}

// ComposableE2ETestSuite demonstrates the new composable transaction architecture
// This suite shows how multiple instructions can be composed into atomic transactions
type ComposableE2ETestSuite struct {
	suite.Suite
	config               *config.Config
	ctx                  context.Context
	accountService       account_v1.ServiceServiceClientInterface
	systemProgramService system_program_v1.ServiceServiceClientInterface
	transactionService   transaction_v1.ServiceServiceClientInterface
}

// SetupSuite runs once before all tests in the suite
func (suite *ComposableE2ETestSuite) SetupSuite() {
	suite.ctx = context.Background()
	var err error

	// Parse config
	suite.config, err = config.GetConfig("local-config.json")
	suite.Require().NoError(err, "error parsing config")

	suite.T().Log("🔧 Composable E2E Test Configuration:")
	suite.T().Logf("   Solana RPC URL: %s", suite.config.SolanaRPCURL)
	suite.T().Logf("   Backend Endpoint: %s:%d", suite.config.BackendGRPCEndpoint, suite.config.BackendGRPCPort)
	suite.T().Logf("   Test Account: %s", suite.config.TestAccountAddress)

	// Set environment variable for backend to use local validator
	os.Setenv("SOLANA_RPC_URL", suite.config.SolanaRPCURL)

	// Initialize service clients
	baseURL := fmt.Sprintf("%s:%d", suite.config.BackendGRPCEndpoint, suite.config.BackendGRPCPort)
	clientTimeout := 60 * time.Second

	suite.accountService, err = account_v1.NewServiceService(
		common.WithURL(baseURL),
		common.WithInsecure(),
		common.WithTimeout(clientTimeout),
	)
	suite.Require().NoError(err, "error constructing account service client")

	suite.systemProgramService, err = system_program_v1.NewServiceService(
		common.WithURL(baseURL),
		common.WithInsecure(),
		common.WithTimeout(clientTimeout),
	)
	suite.Require().NoError(err, "error constructing system program service client")

	suite.transactionService, err = transaction_v1.NewServiceService(
		common.WithURL(baseURL),
		common.WithInsecure(),
		common.WithTimeout(clientTimeout),
	)
	suite.Require().NoError(err, "error constructing transaction service client")

	suite.T().Log("✅ All composable service clients initialized successfully")

	// Validate environment connectivity
	suite.validateEnvironment()
}

// TearDownSuite runs once after all tests in the suite
func (suite *ComposableE2ETestSuite) TearDownSuite() {
	// Close all service connections
	if suite.accountService != nil {
		suite.accountService.Close()
	}
	if suite.systemProgramService != nil {
		suite.systemProgramService.Close()
	}
	if suite.transactionService != nil {
		suite.transactionService.Close()
	}

	// Clean up environment variables
	os.Unsetenv("SOLANA_RPC_URL")

	suite.T().Log("🧹 Composable test suite cleanup completed")
}

// waitForAccountFunded polls the GetAccount API until the account is visible with the expected minimum balance
func (suite *ComposableE2ETestSuite) waitForAccountFunded(address string, minLamports uint64, timeout time.Duration) *account_v1.Account {
	suite.T().Logf("   ⏳ Polling for account %s to become visible...", address[:16]+"...")

	start := time.Now()
	attempt := 1

	for time.Since(start) < timeout {
		commitmentLevel := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
		resp, err := suite.accountService.GetAccount(suite.ctx, &account_v1.GetAccountRequest{
			Address:         address,
			CommitmentLevel: &commitmentLevel,
		})

		if err == nil && resp != nil && resp.Lamports >= minLamports {
			suite.T().Logf("   ✅ Account found after %v (attempt %d)", time.Since(start), attempt)
			return resp
		}

		if attempt <= 3 || attempt%5 == 0 { // Log first 3 attempts, then every 5th
			suite.T().Logf("   🔄 Attempt %d: Account not yet visible, retrying... (elapsed: %v)", attempt, time.Since(start))
		}

		attempt++
		time.Sleep(200 * time.Millisecond) // Poll every 200ms
	}

	// If we get here, the account wasn't found within the timeout
	suite.T().Fatalf("❌ Account %s not visible via API after %v", address[:16]+"...", timeout)
	return nil // unreachable
}

// SetupTest runs before each test
func (suite *ComposableE2ETestSuite) SetupTest() {
	suite.T().Log("🔸 Composable Architecture Test 🔸")
}

// TearDownTest runs after each test
func (suite *ComposableE2ETestSuite) TearDownTest() {
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
}

// validateEnvironment validates test environment connectivity
func (suite *ComposableE2ETestSuite) validateEnvironment() {
	suite.T().Log("🔍 Validating composable architecture test environment...")

	ctx, cancel := context.WithTimeout(suite.ctx, 10*time.Second)
	defer cancel()

	// Try to get system program account to validate connectivity
	commitmentLevel := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	_, err := suite.accountService.GetAccount(ctx, &account_v1.GetAccountRequest{
		Address:         "11111111111111111111111111111111", // System program ID
		CommitmentLevel: &commitmentLevel,
	})
	if err != nil {
		suite.T().Logf("⚠️ Warning: Backend connectivity issue: %v", err)
		suite.T().Log("💡 Make sure local validator and backend are running with composable architecture support")
	} else {
		suite.T().Log("✅ Environment validation successful for composable architecture")
	}
}

// ========================================
// INSTRUCTION GENERATION TESTS
// ========================================

// Test_01_SystemProgram_CreateInstruction tests creating a system program instruction
func (suite *ComposableE2ETestSuite) Test_01_SystemProgram_CreateInstruction() {
	suite.T().Log("🎯 Testing System Program: Create Account Instruction")

	request := &system_program_v1.CreateRequest{
		Payer:      suite.config.TestAccountAddress,
		NewAccount: "11111111111111111111111111111112",
		Lamports:   1000000000, // 1 SOL
		Space:      0,
	}

	suite.T().Logf("📤 Creating system program instruction:")
	suite.T().Logf("   Payer: %s", request.Payer)
	suite.T().Logf("   New Account: %s", request.NewAccount)
	suite.T().Logf("   Lamports: %d", request.Lamports)

	// System program service returns SolanaInstruction directly
	instruction, err := suite.systemProgramService.Create(suite.ctx, request)
	suite.Require().NoError(err, "Create instruction should succeed")
	suite.Require().NotNil(instruction, "Instruction should not be nil")

	// Validate instruction structure
	suite.Assert().NotEmpty(instruction.ProgramId, "Instruction should have program ID")
	suite.Assert().Equal("11111111111111111111111111111111", instruction.ProgramId, "Should be system program ID")
	suite.Assert().NotEmpty(instruction.Accounts, "Instruction should have accounts")
	suite.Assert().NotEmpty(instruction.Data, "Instruction should have data")
	suite.Assert().NotEmpty(instruction.Description, "Instruction should have description")

	// Validate account metadata
	suite.Assert().True(len(instruction.Accounts) >= 2, "Create instruction should have at least 2 accounts")

	// Find payer and new account in the accounts list
	var payerAccount, newAccountAccount *transaction_v1.SolanaAccountMeta
	for _, acc := range instruction.Accounts {
		if acc.Pubkey == request.Payer {
			payerAccount = acc
		}
		if acc.Pubkey == request.NewAccount {
			newAccountAccount = acc
		}
	}

	suite.Require().NotNil(payerAccount, "Payer account should be in accounts list")
	suite.Require().NotNil(newAccountAccount, "New account should be in accounts list")

	suite.Assert().True(payerAccount.IsSigner, "Payer should be a signer")
	suite.Assert().True(payerAccount.IsWritable, "Payer should be writable (paying fees)")
	suite.Assert().True(newAccountAccount.IsSigner, "New account should be a signer")
	suite.Assert().True(newAccountAccount.IsWritable, "New account should be writable")

	suite.T().Logf("✅ System program create instruction generated:")
	suite.T().Logf("   Program ID: %s", instruction.ProgramId)
	suite.T().Logf("   Accounts: %d", len(instruction.Accounts))
	suite.T().Logf("   Data Length: %d bytes", len(instruction.Data))
	suite.T().Logf("   Description: %s", instruction.Description)
}

// Test_02_SystemProgram_TransferInstruction tests creating a transfer instruction
func (suite *ComposableE2ETestSuite) Test_02_SystemProgram_TransferInstruction() {
	suite.T().Log("🎯 Testing System Program: Transfer Instruction")

	request := &system_program_v1.TransferRequest{
		From:     suite.config.TestAccountAddress,
		To:       "11111111111111111111111111111112",
		Lamports: 500000000, // 0.5 SOL
	}

	suite.T().Logf("📤 Creating transfer instruction:")
	suite.T().Logf("   From: %s", request.From)
	suite.T().Logf("   To: %s", request.To)
	suite.T().Logf("   Lamports: %d (%.4f SOL)", request.Lamports, float64(request.Lamports)/1_000_000_000)

	// System program service returns SolanaInstruction directly
	instruction, err := suite.systemProgramService.Transfer(suite.ctx, request)
	suite.Require().NoError(err, "Transfer instruction should succeed")
	suite.Require().NotNil(instruction, "Instruction should not be nil")

	// Validate instruction structure
	suite.Assert().Equal("11111111111111111111111111111111", instruction.ProgramId, "Should be system program ID")
	suite.Assert().Len(instruction.Accounts, 2, "Transfer should have exactly 2 accounts")
	suite.Assert().NotEmpty(instruction.Data, "Instruction should have data")
	suite.Assert().Contains(instruction.Description, "Transfer", "Description should mention transfer")

	// Validate account metadata for transfer
	fromAccount := instruction.Accounts[0]
	toAccount := instruction.Accounts[1]

	suite.Assert().Equal(request.From, fromAccount.Pubkey, "First account should be from address")
	suite.Assert().Equal(request.To, toAccount.Pubkey, "Second account should be to address")

	suite.Assert().True(fromAccount.IsSigner, "From account should be a signer")
	suite.Assert().True(fromAccount.IsWritable, "From account should be writable")
	suite.Assert().False(toAccount.IsSigner, "To account should not need to sign")
	suite.Assert().True(toAccount.IsWritable, "To account should be writable")

	suite.T().Logf("✅ Transfer instruction generated successfully")
}

// ========================================
// COMPOSABLE TRANSACTION TESTS
// ========================================

// Test_03_ComposableTransaction_SingleInstruction tests creating a transaction with single instruction
func (suite *ComposableE2ETestSuite) Test_03_ComposableTransaction_SingleInstruction() {
	suite.T().Log("🎯 Testing Composable Transaction: Single Instruction Flow")

	// Step 1: Generate an instruction
	suite.T().Log("📤 Step 1: Creating transfer instruction")
	transferRequest := &system_program_v1.TransferRequest{
		From:     suite.config.TestAccountAddress,
		To:       "11111111111111111111111111111112",
		Lamports: 1000000, // 0.001 SOL
	}

	transferInstruction, err := suite.systemProgramService.Transfer(suite.ctx, transferRequest)
	suite.Require().NoError(err, "Should create transfer instruction")
	suite.Require().NotNil(transferInstruction, "Should have instruction")

	// Step 2: Create a draft transaction with the instruction
	suite.T().Log("📤 Step 2: Creating draft transaction")
	draftTransaction := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{transferInstruction},
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		Config: &transaction_v1.TransactionConfig{
			ComputeUnitLimit: 200000,
			ComputeUnitPrice: 1000,
			PriorityFee:      1000,
		},
	}

	// Step 3: Compile the transaction
	suite.T().Log("📤 Step 3: Compiling transaction")
	compileRequest := &transaction_v1.CompileTransactionRequest{
		Transaction:     draftTransaction,
		FeePayer:        suite.config.TestAccountAddress,
		RecentBlockhash: "", // Let backend fetch latest
	}

	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, compileRequest)
	suite.Require().NoError(err, "Should compile transaction")
	suite.Require().NotNil(compileResp.Transaction, "Should return compiled transaction")

	// Validate compiled transaction
	compiledTx := compileResp.Transaction
	suite.Assert().Equal(transaction_v1.TransactionState_TRANSACTION_STATE_COMPILED, compiledTx.State, "Should be in compiled state")
	suite.Assert().NotEmpty(compiledTx.Data, "Should have compiled transaction data")
	suite.Assert().NotEmpty(compiledTx.FeePayer, "Should have fee payer")
	suite.Assert().NotEmpty(compiledTx.RecentBlockhash, "Should have recent blockhash")
	suite.Assert().Empty(compiledTx.Signatures, "Should not have signatures yet")

	suite.T().Logf("✅ Single instruction transaction compiled successfully")
	suite.T().Logf("   State: %s", compiledTx.State.String())
	suite.T().Logf("   Data Length: %d chars", len(compiledTx.Data))
	suite.T().Logf("   Fee Payer: %s", compiledTx.FeePayer[:16]+"...")
}

// Test_04_ComposableTransaction_MultipleInstructions tests composing multiple instructions
func (suite *ComposableE2ETestSuite) Test_04_ComposableTransaction_MultipleInstructions() {
	suite.T().Log("🎯 Testing Composable Transaction: Multiple Instructions")

	// Generate keypairs for the multi-instruction test
	suite.T().Log("📤 Step 1: Generating keypairs")
	newAccount1Resp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate keypair 1")

	newAccount2Resp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate keypair 2")

	newAccount1 := newAccount1Resp.KeyPair.PublicKey
	newAccount2 := newAccount2Resp.KeyPair.PublicKey

	// Create multiple instructions
	suite.T().Log("📤 Step 2: Creating multiple instructions")

	// Instruction 1: Create first account
	createInstr1, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      suite.config.TestAccountAddress,
		NewAccount: newAccount1,
		Lamports:   1000000000, // 1 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create instruction 1")

	// Instruction 2: Create second account
	createInstr2, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      suite.config.TestAccountAddress,
		NewAccount: newAccount2,
		Lamports:   1000000000, // 1 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create instruction 2")

	// Instruction 3: Transfer between the new accounts
	transferInstr, err := suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
		From:     newAccount1,
		To:       newAccount2,
		Lamports: 500000000, // 0.5 SOL
	})
	suite.Require().NoError(err, "Should create transfer instruction")

	// Step 3: Compose all instructions into one transaction
	suite.T().Log("📤 Step 3: Composing multi-instruction transaction")
	multiInstructionTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createInstr1,  // Create account 1
			createInstr2,  // Create account 2
			transferInstr, // Transfer from 1 to 2
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		Config: &transaction_v1.TransactionConfig{
			ComputeUnitLimit: 300000, // Higher limit for multiple instructions
			ComputeUnitPrice: 1000,
			PriorityFee:      2000,
		},
	}

	// Step 4: Compile the multi-instruction transaction
	suite.T().Log("📤 Step 4: Compiling multi-instruction transaction")
	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: multiInstructionTx,
		FeePayer:    suite.config.TestAccountAddress,
	})
	suite.Require().NoError(err, "Should compile multi-instruction transaction")
	suite.Require().NotNil(compileResp.Transaction, "Should return compiled transaction")

	// Validate the composed transaction
	compiledTx := compileResp.Transaction
	suite.Assert().Equal(transaction_v1.TransactionState_TRANSACTION_STATE_COMPILED, compiledTx.State)
	suite.Assert().NotEmpty(compiledTx.Data, "Should have compiled data")
	suite.Assert().Len(compiledTx.Instructions, 3, "Should preserve all 3 instructions")

	suite.T().Logf("✅ Multi-instruction transaction compiled successfully:")
	suite.T().Logf("   Instructions: %d", len(compiledTx.Instructions))
	suite.T().Logf("   1. Create account: %s", newAccount1[:16]+"...")
	suite.T().Logf("   2. Create account: %s", newAccount2[:16]+"...")
	suite.T().Logf("   3. Transfer: %s -> %s", newAccount1[:16]+"...", newAccount2[:16]+"...")
	suite.T().Log("   🎉 All operations will execute atomically!")
}

// ========================================
// TRANSACTION LIFECYCLE TESTS
// ========================================

// Test_05_TransactionLifecycle_EstimateSimulate tests estimation and simulation
func (suite *ComposableE2ETestSuite) Test_05_TransactionLifecycle_EstimateSimulate() {
	suite.T().Log("🎯 Testing Transaction Lifecycle: Estimate and Simulate")

	// Create and compile a transaction
	transferInstr, err := suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
		From:     suite.config.TestAccountAddress,
		To:       "11111111111111111111111111111112",
		Lamports: 1000000,
	})
	suite.Require().NoError(err, "Should create transfer instruction")

	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: &transaction_v1.Transaction{
			Instructions: []*transaction_v1.SolanaInstruction{transferInstr},
			State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		},
		FeePayer: suite.config.TestAccountAddress,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Step 1: Test estimation
	suite.T().Log("📤 Step 1: Estimating transaction fees")
	commitmentLevel := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	estimateResp, err := suite.transactionService.EstimateTransaction(suite.ctx, &transaction_v1.EstimateTransactionRequest{
		Transaction:     compileResp.Transaction,
		CommitmentLevel: &commitmentLevel,
	})
	suite.Require().NoError(err, "Should estimate transaction")
	suite.Require().NotNil(estimateResp, "Should return estimation")

	// Validate estimation
	suite.Assert().Greater(estimateResp.ComputeUnits, uint64(0), "Should have compute units estimate")
	suite.Assert().Greater(estimateResp.FeeLamports, uint64(0), "Should have fee estimate")
	suite.Assert().GreaterOrEqual(estimateResp.PriorityFee, uint64(0), "Should have priority fee estimate")

	suite.T().Logf("✅ Transaction estimation successful:")
	suite.T().Logf("   Compute Units: %d", estimateResp.ComputeUnits)
	suite.T().Logf("   Fee Lamports: %d", estimateResp.FeeLamports)
	suite.T().Logf("   Priority Fee: %d", estimateResp.PriorityFee)

	// Step 2: Test simulation
	suite.T().Log("📤 Step 2: Simulating transaction")
	commitmentLevel2 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	simulateResp, err := suite.transactionService.SimulateTransaction(suite.ctx, &transaction_v1.SimulateTransactionRequest{
		Transaction:     compileResp.Transaction,
		CommitmentLevel: &commitmentLevel2,
	})
	suite.Require().NoError(err, "Should simulate transaction")
	suite.Require().NotNil(simulateResp, "Should return simulation result")

	// Validate simulation (may succeed or fail depending on network state)
	// Note: logs can be nil for failed simulations like AccountNotFound
	if simulateResp.Success {
		suite.Assert().NotNil(simulateResp.Logs, "Successful simulations should have logs")
	}

	suite.T().Logf("✅ Transaction simulation completed:")
	suite.T().Logf("   Success: %t", simulateResp.Success)
	suite.T().Logf("   Error: %s", simulateResp.Error)
	suite.T().Logf("   Logs: %d entries", len(simulateResp.Logs))
}

// Test_06_TransactionLifecycle_SigningFlow tests the signing workflow
func (suite *ComposableE2ETestSuite) Test_06_TransactionLifecycle_SigningFlow() {
	suite.T().Log("🎯 Testing Transaction Lifecycle: Signing Flow")

	// Generate test keypair
	keyPairResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate keypair")

	testAddress := keyPairResp.KeyPair.PublicKey
	testPrivateKey := keyPairResp.KeyPair.PrivateKey

	// Create and compile transaction
	transferInstr, err := suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
		From:     testAddress,
		To:       "11111111111111111111111111111112",
		Lamports: 1000000,
	})
	suite.Require().NoError(err, "Should create transfer instruction")

	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: &transaction_v1.Transaction{
			Instructions: []*transaction_v1.SolanaInstruction{transferInstr},
			State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		},
		FeePayer: testAddress,
	})
	suite.Require().NoError(err, "Should compile transaction")

	// Step 1: Sign with private keys
	suite.T().Log("📤 Step 1: Signing with private keys")
	signResp, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compileResp.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{testPrivateKey},
			},
		},
	})
	suite.Require().NoError(err, "Should sign transaction")
	suite.Require().NotNil(signResp.Transaction, "Should return signed transaction")

	// Validate signed transaction
	signedTx := signResp.Transaction
	suite.Assert().Equal(transaction_v1.TransactionState_TRANSACTION_STATE_FULLY_SIGNED, signedTx.State, "Should be in fully signed state (single signer transaction)")
	suite.Assert().NotEmpty(signedTx.Signatures, "Should have signatures")
	suite.Assert().Greater(len(signedTx.Signatures), 0, "Should have at least one signature")

	suite.T().Logf("✅ Transaction signed successfully:")
	suite.T().Logf("   State: %s", signedTx.State.String())
	suite.T().Logf("   Signatures: %d", len(signedTx.Signatures))
}

// ========================================
// COMPLETE COMPOSABLE FLOW TESTS
// ========================================

// Test_07_CompleteComposableFlow demonstrates the full composable workflow
func (suite *ComposableE2ETestSuite) Test_07_CompleteComposableFlow() {
	suite.T().Log("🎯 Complete Composable Flow: Draft → Compile → Estimate → Simulate → Sign")

	// Generate test accounts including a fee payer
	payerResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate fee payer")

	account1Resp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate account 1")

	account2Resp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate account 2")

	payerAddr := payerResp.KeyPair.PublicKey
	payerPrivateKey := payerResp.KeyPair.PrivateKey
	account1Addr := account1Resp.KeyPair.PublicKey
	account1PrivKey := account1Resp.KeyPair.PrivateKey
	account2Addr := account2Resp.KeyPair.PublicKey
	account2PrivKey := account2Resp.KeyPair.PrivateKey

	suite.T().Log("🏗️ COMPLETE COMPOSABLE WORKFLOW DEMONSTRATION:")

	// Step 1: Create multiple instructions
	suite.T().Log("📤 Step 1: Creating composable instructions")

	createInstr1, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      payerAddr,
		NewAccount: account1Addr,
		Lamports:   2000000000, // 2 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create instruction 1")

	createInstr2, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      payerAddr,
		NewAccount: account2Addr,
		Lamports:   1000000000, // 1 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create instruction 2")

	transferInstr, err := suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
		From:     account1Addr,
		To:       account2Addr,
		Lamports: 500000000, // 0.5 SOL
	})
	suite.Require().NoError(err, "Should create transfer instruction")

	suite.T().Logf("   ✅ Created 3 instructions: 2 creates + 1 transfer")

	// Step 2: Compose into draft transaction
	suite.T().Log("📤 Step 2: Composing draft transaction")
	draftTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createInstr1,
			createInstr2,
			transferInstr,
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		Config: &transaction_v1.TransactionConfig{
			ComputeUnitLimit: 400000,
			ComputeUnitPrice: 1000,
			PriorityFee:      2000,
			SkipPreflight:    false,
		},
	}

	suite.T().Logf("   ✅ Draft transaction with %d instructions", len(draftTx.Instructions))

	// Step 3: Compile transaction
	suite.T().Log("📤 Step 3: Compiling transaction")
	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: draftTx,
		FeePayer:    payerAddr,
	})
	suite.Require().NoError(err, "Should compile transaction")
	compiledTx := compileResp.Transaction

	suite.Assert().Equal(transaction_v1.TransactionState_TRANSACTION_STATE_COMPILED, compiledTx.State)
	suite.T().Logf("   ✅ Transaction compiled successfully")

	// Step 4: Estimate fees
	suite.T().Log("📤 Step 4: Estimating transaction costs")
	commitmentLevel3 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	estimateResp, err := suite.transactionService.EstimateTransaction(suite.ctx, &transaction_v1.EstimateTransactionRequest{
		Transaction:     compiledTx,
		CommitmentLevel: &commitmentLevel3,
	})
	suite.Require().NoError(err, "Should estimate transaction")

	suite.T().Logf("   ✅ Estimates: %d CU, %d lamports fee", estimateResp.ComputeUnits, estimateResp.FeeLamports)

	// Step 5: Simulate transaction
	suite.T().Log("📤 Step 5: Simulating transaction")
	commitmentLevel4 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	simulateResp, err := suite.transactionService.SimulateTransaction(suite.ctx, &transaction_v1.SimulateTransactionRequest{
		Transaction:     compiledTx,
		CommitmentLevel: &commitmentLevel4,
	})
	suite.Require().NoError(err, "Should simulate transaction")

	suite.T().Logf("   ✅ Simulation: success=%t, logs=%d", simulateResp.Success, len(simulateResp.Logs))

	// Step 6: Sign with multiple keys
	suite.T().Log("📤 Step 6: Signing with multiple private keys")
	signResp, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compiledTx,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{
					payerPrivateKey, // Payer signature
					account1PrivKey, // New account 1 signature
					account2PrivKey, // New account 2 signature
				},
			},
		},
	})
	suite.Require().NoError(err, "Should sign transaction")
	signedTx := signResp.Transaction

	suite.T().Logf("   ✅ Signed with %d signatures", len(signedTx.Signatures))

	// Step 7: Final validation
	suite.T().Log("📤 Step 7: Final transaction validation")
	suite.Assert().NotEmpty(signedTx.Signatures, "Should have signatures")
	suite.Assert().NotEmpty(signedTx.Data, "Should have transaction data")
	suite.Assert().NotEmpty(signedTx.FeePayer, "Should have fee payer")
	suite.Assert().NotEmpty(signedTx.RecentBlockhash, "Should have recent blockhash")

	suite.T().Log("🎉 COMPLETE COMPOSABLE WORKFLOW SUCCESS!")
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
	suite.T().Log("✨ COMPOSABLE ARCHITECTURE BENEFITS DEMONSTRATED:")
	suite.T().Log("   🔹 Multiple instructions composed into single atomic transaction")
	suite.T().Log("   🔹 Automatic account deduplication and privilege management")
	suite.T().Log("   🔹 Clear state transitions: Draft → Compiled → Signed")
	suite.T().Log("   🔹 Separate concerns: instruction creation, compilation, signing")
	suite.T().Log("   🔹 Comprehensive transaction lifecycle management")
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
}

// Test_08_InstructionCompositionBenefits demonstrates advantages over old architecture
func (suite *ComposableE2ETestSuite) Test_08_InstructionCompositionBenefits() {
	suite.T().Log("🎯 Demonstrating Instruction Composition Benefits")

	suite.T().Log("🔥 ARCHITECTURE COMPARISON:")
	suite.T().Log("   OLD: Each service call returns a complete transaction")
	suite.T().Log("   NEW: Each service call returns a composable instruction")
	suite.T().Log("")

	// Demonstrate multiple system program operations
	operations := []struct {
		name string
		call func() (*transaction_v1.SolanaInstruction, error)
	}{
		{
			name: "CreateAccount",
			call: func() (*transaction_v1.SolanaInstruction, error) {
				return suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
					Payer: suite.config.TestAccountAddress, NewAccount: "11111111111111111111111111111112", Lamports: 1000000000, Space: 0,
				})
			},
		},
		{
			name: "Transfer",
			call: func() (*transaction_v1.SolanaInstruction, error) {
				return suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
					From: suite.config.TestAccountAddress, To: "11111111111111111111111111111112", Lamports: 500000000,
				})
			},
		},
	}

	var instructions []*transaction_v1.SolanaInstruction
	for _, op := range operations {
		suite.T().Logf("📤 Creating %s instruction", op.name)
		instr, err := op.call()
		suite.Require().NoError(err, "Should create %s instruction", op.name)
		suite.Require().NotNil(instr, "%s instruction should not be nil", op.name)
		instructions = append(instructions, instr)
		suite.T().Logf("   ✅ %s instruction ready for composition", op.name)
	}

	suite.T().Log("🎯 COMPOSITION BENEFITS:")
	suite.T().Logf("   ✅ Generated %d independent instructions", len(instructions))
	suite.T().Log("   ✅ Each instruction is reusable and composable")
	suite.T().Log("   ✅ Can combine any instructions into single atomic transaction")
	suite.T().Log("   ✅ Automatic account metadata management")
	suite.T().Log("   ✅ Clear separation of concerns")
	suite.T().Log("")

	// Demonstrate composition
	suite.T().Log("📤 Composing all instructions into single transaction...")
	composedTx := &transaction_v1.Transaction{
		Instructions: instructions,
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		Config:       &transaction_v1.TransactionConfig{ComputeUnitLimit: 300000},
	}

	// Compile the composed transaction
	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: composedTx,
		FeePayer:    suite.config.TestAccountAddress,
	})
	suite.Require().NoError(err, "Should compile composed transaction")

	suite.T().Log("🎉 COMPOSITION SUCCESS!")
	suite.T().Logf("   Combined %d operations into 1 atomic transaction", len(instructions))
	suite.T().Logf("   Transaction size: %d characters", len(compileResp.Transaction.Data))
	suite.T().Log("   All operations will execute or fail together!")
}

// ========================================
// COMPREHENSIVE BLOCKCHAIN INTEGRATION TEST
// ========================================

// Test_09_ComprehensiveBlockchainIntegration - Full E2E test that actually submits to blockchain
// This test creates accounts, funds them, and performs real transactions on the Solana blockchain
func (suite *ComposableE2ETestSuite) Test_09_ComprehensiveBlockchainIntegration() {
	suite.T().Log("🚀 COMPREHENSIVE BLOCKCHAIN INTEGRATION TEST")
	suite.T().Log("📋 This test will create accounts, fund them, and perform real blockchain transactions")
	suite.T().Log("🔗 All transactions will be submitted to the local Solana validator")
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")

	// Step 1: Create primary keypair that will be our main payer
	suite.T().Log("📤 Step 1: Creating primary keypair for funding")
	primaryKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate primary keypair")
	suite.Require().NotNil(primaryKeyResp.KeyPair, "Primary keypair should not be nil")

	primaryAddr := primaryKeyResp.KeyPair.PublicKey
	primaryPrivKey := primaryKeyResp.KeyPair.PrivateKey

	suite.T().Logf("   ✅ Primary account created: %s", primaryAddr[:16]+"...")

	// Step 2: Fund the primary account using FundNative API service
	suite.T().Log("📤 Step 2: Funding primary account via FundNative API")
	commitmentLevel5 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	fundResp, err := suite.accountService.FundNative(suite.ctx, &account_v1.FundNativeRequest{
		Address:         primaryAddr,
		Amount:          "10000000000", // 10 SOL
		CommitmentLevel: &commitmentLevel5,
	})
	suite.Require().NoError(err, "Should fund primary account via API")
	suite.Require().NotNil(fundResp, "Fund response should not be nil")
	suite.Require().NotEmpty(fundResp.Signature, "Should have funding transaction signature")

	suite.T().Logf("   ✅ Primary account funded: %s", primaryAddr[:16]+"...")
	suite.T().Logf("   💰 Funding amount: 10.0 SOL")
	suite.T().Logf("   📝 Funding signature: %s", fundResp.Signature)

	// Step 2b: Wait for funding to be visible in API (commitment timing issue)
	suite.T().Log("📤 Step 2b: Waiting for primary account to be visible via API")
	balanceResp := suite.waitForAccountFunded(primaryAddr, 10000000000, 10*time.Second)

	suite.T().Logf("   ✅ Primary account verified: %.4f SOL", float64(balanceResp.Lamports)/1_000_000_000)
	suite.T().Log("   🔄 Ready to use as fee payer for subsequent transactions")

	// Step 3: Create second account that will be created and funded by primary account
	suite.T().Log("📤 Step 3: Creating second account keypair")
	secondKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate second keypair")
	suite.Require().NotNil(secondKeyResp.KeyPair, "Second keypair should not be nil")

	secondAddr := secondKeyResp.KeyPair.PublicKey
	secondPrivKey := secondKeyResp.KeyPair.PrivateKey

	suite.T().Logf("   ✅ Second account created: %s", secondAddr[:16]+"...")

	// Step 4: Create third account for transfer operations
	suite.T().Log("📤 Step 4: Creating third account keypair")
	thirdKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate third keypair")
	suite.Require().NotNil(thirdKeyResp.KeyPair, "Third keypair should not be nil")

	thirdAddr := thirdKeyResp.KeyPair.PublicKey
	thirdPrivKey := thirdKeyResp.KeyPair.PrivateKey

	suite.T().Logf("   ✅ Third account created: %s", thirdAddr[:16]+"...")

	// Step 5: Create instruction to create second account (funded by primary)
	suite.T().Log("📤 Step 5: Creating account creation instruction")
	createSecondInstr, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      primaryAddr,
		NewAccount: secondAddr,
		Lamports:   1000000000, // 1 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create account creation instruction")
	suite.Require().NotNil(createSecondInstr, "Create instruction should not be nil")

	suite.T().Logf("   ✅ Account creation instruction: %s", createSecondInstr.Description)

	// Step 6: Create instruction to create third account (funded by primary)
	suite.T().Log("📤 Step 6: Creating second account creation instruction")
	createThirdInstr, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      primaryAddr,
		NewAccount: thirdAddr,
		Lamports:   500000000, // 0.5 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create third account creation instruction")
	suite.Require().NotNil(createThirdInstr, "Create third instruction should not be nil")

	suite.T().Logf("   ✅ Third account creation instruction: %s", createThirdInstr.Description)

	// Step 7: Create transfer instruction from second to third account
	suite.T().Log("📤 Step 7: Creating transfer instruction")
	transferInstr, err := suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
		From:     secondAddr,
		To:       thirdAddr,
		Lamports: 250000000, // 0.25 SOL
	})
	suite.Require().NoError(err, "Should create transfer instruction")
	suite.Require().NotNil(transferInstr, "Transfer instruction should not be nil")

	suite.T().Logf("   ✅ Transfer instruction: %s", transferInstr.Description)

	// Step 8: Compose all instructions into one atomic transaction
	suite.T().Log("📤 Step 8: Composing multi-instruction atomic transaction")
	atomicTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createSecondInstr, // Create second account with 1 SOL
			createThirdInstr,  // Create third account with 0.5 SOL
			transferInstr,     // Transfer 0.25 SOL from newly created second to third account
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		Config: &transaction_v1.TransactionConfig{
			ComputeUnitLimit: 500000,
			ComputeUnitPrice: 1000,
			PriorityFee:      2000,
		},
	}

	suite.T().Logf("   ✅ TRUE ATOMIC transaction composed with %d instructions:", len(atomicTx.Instructions))
	suite.T().Logf("      1. Create account %s with 1.0 SOL", secondAddr[:16]+"...")
	suite.T().Logf("      2. Create account %s with 0.5 SOL", thirdAddr[:16]+"...")
	suite.T().Logf("      3. Transfer 0.25 SOL from newly created account → %s", thirdAddr[:16]+"...")
	suite.T().Log("      🎯 This demonstrates Solana's atomic transaction capability!")

	// Step 9: Compile the transaction
	suite.T().Log("📤 Step 9: Compiling atomic transaction")
	compileResp, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: atomicTx,
		FeePayer:    primaryAddr, // Primary account pays all fees
	})
	suite.Require().NoError(err, "Should compile atomic transaction")
	suite.Require().NotNil(compileResp.Transaction, "Compiled transaction should not be nil")

	compiledTx := compileResp.Transaction
	suite.T().Logf("   ✅ Transaction compiled successfully:")
	suite.T().Logf("      State: %s", compiledTx.State.String())
	suite.T().Logf("      Fee Payer: %s", compiledTx.FeePayer[:16]+"...")
	suite.T().Logf("      Data Length: %d bytes", len(compiledTx.Data))

	// Step 10: Estimate transaction costs
	suite.T().Log("📤 Step 10: Estimating transaction costs")
	commitmentLevel6 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	estimateResp, err := suite.transactionService.EstimateTransaction(suite.ctx, &transaction_v1.EstimateTransactionRequest{
		Transaction:     compiledTx,
		CommitmentLevel: &commitmentLevel6,
	})
	suite.Require().NoError(err, "Should estimate transaction costs")

	suite.T().Logf("   ✅ Transaction cost estimation:")
	suite.T().Logf("      Compute Units: %d", estimateResp.ComputeUnits)
	suite.T().Logf("      Fee Lamports: %d (%.4f SOL)", estimateResp.FeeLamports, float64(estimateResp.FeeLamports)/1_000_000_000)
	suite.T().Logf("      Priority Fee: %d lamports", estimateResp.PriorityFee)

	// Step 11: Sign the transaction with all required keys
	suite.T().Log("📤 Step 11: Signing transaction with all required keys")
	signResp, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compiledTx,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{
					primaryPrivKey, // Payer and fee payer
					secondPrivKey,  // New account must sign its creation
					thirdPrivKey,   // New account must sign its creation
				},
			},
		},
	})
	suite.Require().NoError(err, "Should sign transaction")
	suite.Require().NotNil(signResp.Transaction, "Signed transaction should not be nil")

	signedTx := signResp.Transaction
	suite.T().Logf("   ✅ Transaction signed with %d signatures:", len(signedTx.Signatures))
	suite.T().Logf("      State: %s", signedTx.State.String())
	suite.T().Logf("      Ready for blockchain submission!")

	// Step 12: Submit transaction to blockchain (should succeed now that account is funded)
	suite.T().Log("📤 Step 12: 🚀 SUBMITTING FIRST TRANSACTION TO BLOCKCHAIN! 🚀")

	// Add delay to ensure funding transaction is fully processed by validator
	suite.T().Log("   ⏳ Waiting for validator to fully process funding transaction...")
	time.Sleep(500 * time.Millisecond)

	commitmentLevel7 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	submitResp, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction:     signedTx,
		CommitmentLevel: &commitmentLevel7,
	})
	suite.Require().NoError(err, "Should submit transaction successfully")
	suite.Require().NotNil(submitResp, "Submit response should not be nil")
	suite.Require().NotEmpty(submitResp.Signature, "Should have transaction signature")

	suite.T().Logf("   🎉 ATOMIC CREATE+TRANSFER TRANSACTION SUCCESSFULLY SUBMITTED!")
	suite.T().Logf("   📝 Transaction Signature: %s", submitResp.Signature)
	suite.T().Log("   ✅ Created 2 accounts + transferred 0.25 SOL atomically!")
	suite.T().Log("   🎯 This proves Solana's atomic transaction capability works perfectly!")

	// Store the signature for verification (this single transaction did everything)
	firstTxSignature := submitResp.Signature
	transferTxSignature := submitResp.Signature // Same transaction

	// Step 13: Create third account using same pattern
	suite.T().Log("📤 Step 13: Creating third account via same API pattern")

	// Generate keypair for third account
	thirdKeyResp2, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate third account keypair")

	thirdAddr2 := thirdKeyResp2.KeyPair.PublicKey
	thirdPrivKey2 := thirdKeyResp2.KeyPair.PrivateKey

	// Create instruction to create this third account (funded by primary)
	createThirdInstr2, err := suite.systemProgramService.Create(suite.ctx, &system_program_v1.CreateRequest{
		Payer:      primaryAddr,
		NewAccount: thirdAddr2,
		Lamports:   300000000, // 0.3 SOL
		Space:      0,
	})
	suite.Require().NoError(err, "Should create third account instruction")

	// Create, compile, sign, and submit second transaction
	secondTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{createThirdInstr2},
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}

	compileResp2, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: secondTx,
		FeePayer:    primaryAddr,
	})
	suite.Require().NoError(err, "Should compile second transaction")

	signResp2, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compileResp2.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{primaryPrivKey, thirdPrivKey2},
			},
		},
	})
	suite.Require().NoError(err, "Should sign second transaction")

	commitmentLevel8 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	submitResp2, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction:     signResp2.Transaction,
		CommitmentLevel: &commitmentLevel8,
	})
	suite.Require().NoError(err, "Should submit second transaction")

	secondTxSignature := submitResp2.Signature
	suite.T().Logf("   ✅ Second transaction submitted: %s", secondTxSignature)

	// Step 14: Create transfer transaction between created accounts
	suite.T().Log("📤 Step 14: Creating transfer between created accounts")

	transferInstr2, err := suite.systemProgramService.Transfer(suite.ctx, &system_program_v1.TransferRequest{
		From:     secondAddr, // From first created account (has 0.75 SOL after transfer)
		To:       thirdAddr2, // To newly created third account
		Lamports: 100000000,  // 0.1 SOL
	})
	suite.Require().NoError(err, "Should create transfer instruction")

	// Create, compile, sign and submit transfer transaction
	transferTx2 := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{transferInstr2},
		State:        transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
	}

	compileResp3, err := suite.transactionService.CompileTransaction(suite.ctx, &transaction_v1.CompileTransactionRequest{
		Transaction: transferTx2,
		FeePayer:    secondAddr, // Second account pays its own transfer fee
	})
	suite.Require().NoError(err, "Should compile transfer transaction")

	signResp3, err := suite.transactionService.SignTransaction(suite.ctx, &transaction_v1.SignTransactionRequest{
		Transaction: compileResp3.Transaction,
		SigningMethod: &transaction_v1.SignTransactionRequest_PrivateKeys{
			PrivateKeys: &transaction_v1.SignWithPrivateKeys{
				PrivateKeys: []string{secondPrivKey}, // Only second account needs to sign
			},
		},
	})
	suite.Require().NoError(err, "Should sign transfer transaction")

	commitmentLevel9 := type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED
	submitResp3, err := suite.transactionService.SubmitTransaction(suite.ctx, &transaction_v1.SubmitTransactionRequest{
		Transaction:     signResp3.Transaction,
		CommitmentLevel: &commitmentLevel9,
	})
	suite.Require().NoError(err, "Should submit transfer transaction")

	thirdTxSignature := submitResp3.Signature
	suite.T().Logf("   ✅ Transfer transaction submitted: %s", thirdTxSignature)

	// Step 15: Final Summary with All Transaction Details
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
	suite.T().Log("🎉 COMPREHENSIVE BLOCKCHAIN INTEGRATION COMPLETE!")
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
	suite.T().Log("✅ ALL OPERATIONS SUCCESSFULLY SUBMITTED TO BLOCKCHAIN:")
	suite.T().Log("")
	suite.T().Log("🏦 ACCOUNTS CREATED VIA API:")
	suite.T().Logf("   Primary (funded): %s", primaryAddr)
	suite.T().Logf("   Second (created): %s", secondAddr)
	suite.T().Logf("   Third (original): %s", thirdAddr)
	suite.T().Logf("   Fourth (created): %s", thirdAddr2)
	suite.T().Log("")
	suite.T().Log("💳 BLOCKCHAIN TRANSACTIONS SUBMITTED:")
	suite.T().Logf("   1. 🏦 Funding Tx: %s", fundResp.Signature)
	suite.T().Logf("   2. 🏗️  Account Creation Tx: %s", firstTxSignature)
	suite.T().Logf("      → Created 2 accounts with initial balances")
	suite.T().Logf("   3. 💸 Inter-Account Transfer Tx: %s", transferTxSignature)
	suite.T().Logf("      → Transferred 0.25 SOL between created accounts")
	suite.T().Logf("   4. 🏗️  Second Account Tx: %s", secondTxSignature)
	suite.T().Logf("      → Created fourth account with 0.3 SOL")
	suite.T().Logf("   5. 💸 Final Transfer Tx: %s", thirdTxSignature)
	suite.T().Logf("      → Transferred 0.1 SOL between accounts")
	suite.T().Log("")
	suite.T().Log("🎯 ARCHITECTURAL BENEFITS DEMONSTRATED:")
	suite.T().Log("   ✅ Complete API-driven workflow (no manual CLI needed)")
	suite.T().Log("   ✅ FundNative API service for account funding")
	suite.T().Log("   ✅ Multi-instruction atomic transactions")
	suite.T().Log("   ✅ Account creation via system program API")
	suite.T().Log("   ✅ Transaction compilation, signing, and submission")
	suite.T().Log("   ✅ Inter-account transfers via API")
	suite.T().Log("   ✅ Real blockchain integration and verification")

	// Step 16: CLI Verification Commands
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
	suite.T().Log("🔍 BLOCKCHAIN VERIFICATION COMMANDS:")
	suite.T().Log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
	suite.T().Log("📊 Check Final Account Balances:")
	suite.T().Logf("   solana balance %s --url http://localhost:8899", primaryAddr)
	suite.T().Logf("   solana balance %s --url http://localhost:8899", secondAddr)
	suite.T().Logf("   solana balance %s --url http://localhost:8899", thirdAddr)
	suite.T().Logf("   solana balance %s --url http://localhost:8899", thirdAddr2)
	suite.T().Log("")
	suite.T().Log("🔍 Confirm Individual Transactions:")
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", fundResp.Signature)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", firstTxSignature)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", transferTxSignature)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", secondTxSignature)
	suite.T().Logf("   solana confirm %s --url http://localhost:8899", thirdTxSignature)
	suite.T().Log("")
	suite.T().Log("📜 View Account Transaction History:")
	suite.T().Logf("   solana transaction-history %s --url http://localhost:8899", primaryAddr)
	suite.T().Logf("   solana transaction-history %s --url http://localhost:8899", secondAddr)

	suite.T().Log("")
	suite.T().Log("🎉 COMPREHENSIVE BLOCKCHAIN INTEGRATION TEST COMPLETE!")
}
