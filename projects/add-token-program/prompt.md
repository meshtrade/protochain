# Token Program addition Upgrade - MVP implementing only Mint Initialisation

You are an expert in:
- rust async programming
- solana blockchain development
- solana token 2022 program
- protobuf & gRPC e2e development

## Task
Start wrapping the solana token 2022 program so that instructions from the program can be constructed over gRPC - starting with ONLY the InitialiseMint instruction to limit scope.
This is to be done in the same way, following the same pattern that was followed to wrap the system program:
- protobuf definition: lib/proto/protosol/solana/program/system/v1/service.proto
- rust implementation: api/src/api/program/system/v1/service_impl.rs
- e2e test using the generated Go Client: tests/go/streaming_e2e_test.go

## Target Implementation
Once this has been implemented the following new e2e-mint-initialisation_test.go should be implemented and working:
Go:
```go
	// Generate new key pair for a Payer account
	payKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate payer keypair")

	// Fund the payer account using streaming to monitor completion
	fundResp, err := suite.accountService.FundNative(suite.ctx, &account_v1.FundNativeRequest{
		Address: payKeyResp.KeyPair.PublicKey,
		Amount:  "1000000000", // 1 SOL
	})
	suite.Require().NoError(err, "Should fund account")
	suite.Require().NotEmpty(fundResp.Signature, "Funding must return a transaction signature for proper testing")
	suite.monitorTransactionToCompletion(fundResp.Signature)

    // Generate a new keypair for the new mint
	mintKeyResp, err := suite.accountService.GenerateNewKeyPair(suite.ctx, &account_v1.GenerateNewKeyPairRequest{})
	suite.Require().NoError(err, "Should generate mint keypair")

    // create instruction to create the mint
	createMintInstr, err := suite.systemProgramService.Create(suite.ctx, &system_v1.CreateRequest{
		Payer:      payKeyResp.KeyPair.PublicKey,
		NewAccount: mintKeyResp.KeyPair.PublicKey,
		Lamports:   1000000000, // 1 SOL
		Space:      token_v1.MintSpace,
	})
	suite.Require().NoError(err, "Should create mint account instruction")

    // create instruction to intialise the mint
	initialiseMintInstr, err := suite.systemProgramService.Create(suite.ctx, &token_v1.InitialiseMintRequest{
		MintPubKey:            mintKeyResp.KeyPair.PublicKey,
		MintAuthorityPubKey:   payKeyResp.KeyPair.PublicKey,
		FreezeAuthorityPubKey: payKeyResp.KeyPair.PublicKey,
        Decimals:              2,
	})
	suite.Require().NoError(err, "Should initialise mint instruction")

	// Compose all instructions into one atomic transaction
	suite.T().Log("Composing multi-instruction atomic transaction")
	atomicTx := &transaction_v1.Transaction{
		Instructions: []*transaction_v1.SolanaInstruction{
			createMintInstr,
            initialiseMintInstr,
		},
		State: transaction_v1.TransactionState_TRANSACTION_STATE_DRAFT,
		Config: &transaction_v1.TransactionConfig{
			ComputeUnitLimit: 500000,
			ComputeUnitPrice: 1000,
			PriorityFee:      2000,
		},
	}    
```

## Research and Analysis Required
Create comprehensive research todo list covering:
- Existing protosol system architecture analysis
- All provided reference implementations
- Integration patterns with current codebase

Use step-by-step approach: draft, review, refine sections iteratively.

## Deliverables Required

### 1. Technical Requirements Document
**Location**: `projects/stream-transaction-updates/requirements.md`
**Content**: Pure technical specifications for agent consumption
- Protocol buffer message definitions and field requirements
- gRPC service method signatures with streaming patterns
- WebSocket integration architecture with pubsub client
- Error handling requirements and status codes
- Performance criteria and resource constraints  
- Integration testing requirements
- Validation criteria for success measurement

**FORBIDDEN CONTENT**:
- Any references to human workflow, project management, or scheduling
- Business justifications or value propositions  
- Time estimates or performance targets with specific durations
- Human-oriented success metrics or productivity measures

### 2. Implementation Step Sequence
**Location**: `projects/stream-transaction-updates/implementation-plan.md`  
**Content**: Small, incremental technical steps with strong testing
- Protocol buffer extensions with optimal field design
- Rust dependency management and service integration
- WebSocket client integration with error handling
- gRPC streaming implementation with tonic
- Testing integration with existing Go test suite
- Step-by-step validation checkpoints
- Technical dependencies and build ordering
- Resource management and cleanup requirements

**Requirements for step breakdown**:
- Small steps implementable safely with strong testing
- Each step builds on previous steps  
- All steps end with integration and testing
- No orphaned or hanging code at any stage
- Prioritize incremental progress over large changes

**FORBIDDEN CONTENT**:
- Implementation timelines, schedules, or time allocations
- Human workflow recommendations or project management advice
- References to daily/weekly/monthly patterns
- Time estimates for individual steps or overall completion

## Architecture References (Critical: Look at them all. ONLY THEN decide if the content is relevant)
- Solana transaction lifecycle: https://solana.com/nl/developers/guides/advanced/confirmation
- Solana WebSocket RPC: https://solana.com/docs/rpc/websocket
- signatureSubscribe method: https://solana.com/docs/rpc/websocket/signaturesubscribe
- gRPC server streaming concepts: https://grpc.io/docs/what-is-grpc/core-concepts/#server-streaming-rpc
- tonic server streaming: https://github.com/hyperium/tonic/blob/master/examples/src/streaming/server.rs
- Solana pubsub client: /Users/bernardbussy/Projects/github.com/anza-xyz/agave/pubsub-client
- Solana pubsub docs: https://docs.rs/solana-pubsub-client/latest/solana_pubsub_client/pubsub_client/index.html
- Solana events cookbook: https://solana.com/nl/developers/cookbook/development/subscribing-events

## Code Example Reference
```rust
// Solana WebSocket subscription pattern from cookbook
use anyhow::Result;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient, nonblocking::rpc_client::RpcClient,
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Signer,
    signer::keypair::Keypair,
};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let wallet = Keypair::new();
    let pubkey = wallet.pubkey();

    let connection = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );
    let ws_client = PubsubClient::new("ws://localhost:8900").await?;

    tokio::spawn(async move {
        let config = RpcAccountInfoConfig {
            commitment: Some(CommitmentConfig::confirmed()),
            encoding: None,
            data_slice: None,
            min_context_slot: None,
        };

        let (mut stream, _) = ws_client
            .account_subscribe(&pubkey, Some(config))
            .await
            .expect("Failed to subscribe to account updates");

        while let Some(account) = stream.next().await {
            println!("{:#?}", account);
        }
    });

    let airdrop_signature = connection
        .request_airdrop(&wallet.pubkey(), LAMPORTS_PER_SOL)
        .await?;
    loop {
        let confirmed = connection.confirm_transaction(&airdrop_signature).await?;
        if confirmed {
            break;
        }
    }
    Ok(())
}
```

## Constraints
- This is a greenfields implementation with no backward compatibility requirements
- All new fields should be required where technically appropriate
- Focus on optimal technical design without legacy constraints
- Implementation must integrate with existing protosol architecture in `/Users/bernardbussy/Projects/github.com/BRBussy/protosol`

## Enforcement Mechanism
Before generating any content, verify it contains:
- [ ] Zero timing, scheduling, or duration references
- [ ] Zero human workflow or project management concepts
- [ ] Zero business justifications or motivational content
- [ ] Only technical specifications and implementation steps
- [ ] Agent-oriented language throughout

Generate both documents with pure technical focus suitable for automated agent consumption.