# Transaction Monitoring Streaming Feature - Agent Implementation Directive

You are an expert system architect implementing a gRPC streaming endpoint in protosol.solana.transaction.v1.Service for real-time Solana transaction monitoring with program execution logs.

## Technical Challenge
Current SubmitTransaction is async and returns only signature string (must remain async). Need enhanced response so client knows whether to:
- Handle error return (transaction never reached submission, no monitoring needed)  
- Subscribe to updates via MonitorTransaction stream to watch until completion and determine success/failure through program execution logs

## Technical Context
- Extend existing SubmitTransaction to return enhanced response with submission status
- Add new MonitorTransaction streaming RPC using Solana WebSocket pubsub for signature monitoring  
- Stream program execution logs (missing from TypeScript reference implementation)
- Implement in Rust backend using tonic gRPC server streaming
- Reference: /Users/bernardbussy/Projects/github.com/anza-xyz/kit/packages/transaction-confirmation/src/confirmation-strategy-recent-signature.ts

## Target Implementation
Go:
```go
// Enhanced submit response with immediate feedback
submitResp := protosol.solana.transaction.v1.Service.SubmitTransaction(signedTx)
if submitResp.SubmissionResult != SUBMITTED {
    // Handle submission failure, no monitoring needed
    return
}

// Stream transaction progress until target commitment level
stream := protosol.solana.transaction.v1.Service.MonitorTransaction(submitResp.Signature, CommitmentLevelConfirmed, true)
for update := range stream {
    switch update.Status {
    case CONFIRMED:
        // Transaction confirmed with logs
        return
    case FAILED:
        // Transaction failed with error details
        return
    }
}
```

Pseudo code (This is the ideal flow we want to achieve (an extension of what already happens in tests/go/composable_e2e_test.go)):
```
// create and fund root account (using token faucet on local test net with FundNative)
rootKP = protosol.solana.account.v1.Service.GenerateNewKeyPair(...)
fundRootTxnHash = protosol.solana.account.v1.Service.FundNative(...rootKP.publickey...)
while res <- protosol.solana.transaction.v1.Service.MonitorTransaction(..fundRootTxnHash, CommittmentStatusConfirmed..) {
switch res.Status:
case Success:
    break;
case Failure:
    log.Fatal("failure")
}

// create new account
txInstruction = protosol.solana.program.system.v1.Service.CreateAccount(....)
unsignedTx = protosol.solana.transaction.v1.Service.CompileTransaction(..payer:rootKP..)
signedTx = protosol.solana.transaction.v1.Service.SignTransaction(...)
createNewAccTxnHash = protosol.solana.transaction.v1.Service.SubmitTransaction(.signedTx..)
while res <- protosol.solana.transaction.v1.Service.MonitorTransaction(..createNewAccTxnHash, CommittmentStatusConfirmed..) {
switch res.Status:
case Success:
    break;
case Failure:
    log.Fatal("failure")
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