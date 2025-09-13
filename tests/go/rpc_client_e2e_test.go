package apitest

import (
	"context"
	"testing"

	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	rpc_client_v1 "github.com/BRBussy/protochain/lib/go/protosol/solana/rpc_client/v1"
	type_v1 "github.com/BRBussy/protochain/lib/go/protosol/solana/type/v1"
)

// RpcClientE2ETestSuite tests the RPC Client service functionality
type RpcClientE2ETestSuite struct {
	suite.Suite
	ctx               context.Context
	cancel            context.CancelFunc
	grpcConn          *grpc.ClientConn
	rpcClientService  rpc_client_v1.ServiceClient
}

func (suite *RpcClientE2ETestSuite) SetupSuite() {
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

	// Initialize service client
	suite.rpcClientService = rpc_client_v1.NewServiceClient(suite.grpcConn)

	suite.T().Logf("✅ RPC Client test suite setup complete")
}

func (suite *RpcClientE2ETestSuite) TearDownSuite() {
	if suite.cancel != nil {
		suite.cancel()
	}
	if suite.grpcConn != nil {
		suite.grpcConn.Close()
	}
}

// Test_01_GetMinimumBalanceForRentExemption tests getting minimum balance for rent exemption
func (suite *RpcClientE2ETestSuite) Test_01_GetMinimumBalanceForRentExemption() {
	suite.T().Log("🎯 Testing GetMinimumBalanceForRentExemption")

	// Test with different data lengths - verify relative ordering without hardcoded values
	testCases := []struct {
		name       string
		dataLength uint64
	}{
		{
			name:       "Zero data length",
			dataLength: 0,
		},
		{
			name:       "Small data length",
			dataLength: 100,
		},
		{
			name:       "Medium data length",
			dataLength: 1000,
		},
		{
			name:       "Large data length",
			dataLength: 10000,
		},
	}

	var previousBalance uint64
	var previousDataLength uint64

	for i, tc := range testCases {
		suite.T().Run(tc.name, func(t *testing.T) {
			// Get minimum balance for rent exemption with specific data length
			resp, err := suite.rpcClientService.GetMinimumBalanceForRentExemption(suite.ctx, &rpc_client_v1.GetMinimumBalanceForRentExemptionRequest{
				DataLength: tc.dataLength,
			})
			suite.Require().NoError(err, "should succeed in getting minimum balance for rent exemption")
			suite.Require().NotZero(resp.Balance, "minimum balance for rent exemption should not be zero")
			
			// Verify the balance is reasonable (at least some minimum lamports)
			// Using 890_880 as a baseline since that's approximately the minimum for a 0-byte account
			// but not hardcoding exact values since they can change with Solana updates
			suite.Assert().Greater(resp.Balance, uint64(800_000), 
				"minimum balance should be at least 800,000 lamports (reasonable minimum)")
			
			// Verify that larger data requires more lamports (monotonic increase)
			if i > 0 {
				suite.Assert().Greater(resp.Balance, previousBalance,
					"data length %d should require more lamports than data length %d",
					tc.dataLength, previousDataLength)
			}
			
			suite.T().Logf("  Data length %d requires %d lamports for rent exemption", 
				tc.dataLength, resp.Balance)
			
			previousBalance = resp.Balance
			previousDataLength = tc.dataLength
		})
	}
}

// Test_02_GetMinimumBalanceWithCommitmentLevel tests with different commitment levels
func (suite *RpcClientE2ETestSuite) Test_02_GetMinimumBalanceWithCommitmentLevel() {
	suite.T().Log("🎯 Testing GetMinimumBalanceForRentExemption with different commitment levels")

	commitmentLevels := []struct {
		name  string
		level type_v1.CommitmentLevel
	}{
		{"Processed", type_v1.CommitmentLevel_COMMITMENT_LEVEL_PROCESSED},
		{"Confirmed", type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED},
		{"Finalized", type_v1.CommitmentLevel_COMMITMENT_LEVEL_FINALIZED},
	}

	for _, cl := range commitmentLevels {
		suite.T().Run(cl.name, func(t *testing.T) {
			resp, err := suite.rpcClientService.GetMinimumBalanceForRentExemption(suite.ctx, &rpc_client_v1.GetMinimumBalanceForRentExemptionRequest{
				DataLength:      100,
				CommitmentLevel: cl.level,
			})
			suite.Require().NoError(err, "should succeed with %s commitment level", cl.name)
			suite.Require().NotZero(resp.Balance, "balance should not be zero with %s commitment", cl.name)
			
			suite.T().Logf("  Commitment %s: %d lamports required", cl.name, resp.Balance)
		})
	}
}

func TestRpcClientE2ESuite(t *testing.T) {
	suite.Run(t, new(RpcClientE2ETestSuite))
}