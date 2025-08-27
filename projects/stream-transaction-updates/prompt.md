You are an expert system architect and developer with deep expertise in:
- **Solana blockchain architecture**: Transaction lifecycle, commitment levels, WebSocket RPC APIs, program execution
- **Rust async/gRPC development**: tonic server streaming, tokio ecosystem, WebSocket clients, concurrent stream management
- **Protocol Buffer API design**: Streaming RPC patterns, versioning strategies, cross-language SDK generation
- **Distributed systems**: Real-time data streaming, error handling, resource management, production resilience

## Mission & goal: Transaction Monitoring Streaming Feature
Add a gRPC streaming endpoint to the protosol.solana.transaction.v1.Service that we can use to monitor the progress of a transaction. Implement a production-ready gRPC streaming endpoint for real-time Solana transaction monitoring that provides immediate submission feedback and streams transaction progress with program execution logs.

# Challenge
Transaction submission is completely async right now: protosol.solana.transaction.v1.Service.SubmitTransaction. It should REMAIN that way as this is fundamentally an asynchronous process.
We need a way to return a response OR error from the submit method so that the client knows whether to:
- do nothing, handle error return. Transaction never even made it to submission, no point in monitoring.
- subscribe to updates for the transaction to watch/wait until submission is complete and looking at output of the updates whether or not it was successful or failed. Program that was invoked logs should help with that - i.e. it errored or not.

# Similar already Possible with TS SDK:
- see this: /Users/bernardbussy/Projects/github.com/anza-xyz/kit/packages/transaction-confirmation/src/confirmation-strategy-recent-signature.ts, sort of already does what we want but is missing streaming actual logs of the program execution. It is also of course in typescript in an sdk. We need to see how to do this in rust in our api backend, and then stream through to grpc.

# Ideal User Flow Pseudocode
This is the ideal flow we want to achieve (an extension of what already happens in tests/go/composable_e2e_test.go):

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

# Outcomes of this phase for you:
1. making a complete feature updgrade doc, you will store in projects/stream-transaction-updates/requirements.md. This must contain requirements, task overview, goals, key objectives to achieve to know we have met goal. CRITICAL: this is a requirements document for an agent to consume. So business/human type requirements are not required like: business value, or benefits of this approach or timelines or time estimations. Please none of that. Just technical requirements and outcomes that are relevant to an agent implementing.

2. A step by step implementation plan: A detailed, step-by-step blueprint for achieving everything in the feature requirements.md. In developing this plan you will create a A detailed, step-by-step blueprint for achieving everything in the feature requirements.md - then, once you have a solid plan, break it down into small, iterative chunks that build on each other. Look at these chunks and then go another round to break it into small steps. Review the results and make sure that the steps are small enough to be implemented safely with strong testing, but big enough to move the implementation forward. Iterate until you feel that the steps are right sized for this task. This must prioritize best practices, and incremental progress, ensuring no big jumps in complexity or massive one shotting at any stage. Make sure that each step builds on the previous steps, and ends with wiring things together and testing. There should be no hanging or orphaned code that isn't integrated into a previous step. This blueprint must serve as a thorough TODO that will be followed step by step by you to build this tool - checking off progress as you go along. Store the plan in: projects/stream-transaction-updates/implementation-plan.md. CRITICAL: this is a detailed step by step implementation todo document for an agent to consume. So business/human type requirements are not required like: business value, or benefits of this approach or timelines or time estimations. Please none of that. Just technical step by step implementation plan to track and monitor progress with testing details etc.

## CRITICAL IMPORTANT NOTES:
- this is a completely greenfields project DO NOT WORRY ABOUT making any breaking changes to any APIs or anything

## way of work tips:
- you are to go into deep research mode, make a comprehensive todo list for yourself of all the things you need to research. Consider:
  - existing system architecture
  - all the references I have given you
- everything you do here to be step by step. No one shotting. So for example while making the requirements.md, draft it, then read the draft, then update it section by section. Be critical and review your work.

# References CONSULT THESE, add references back to these if necessary into the final implementation or requirements:
- a similar ts example: /Users/bernardbussy/Projects/github.com/anza-xyz/kit/packages/transaction-confirmation/src/confirmation-strategy-recent-signature.ts (note of course missing the logs from the program)
- Full rust pubsub client you will be constructing and using: /Users/bernardbussy/Projects/github.com/anza-xyz/agave/pubsub-client
- solana transaction lifecycle: https://solana.com/nl/developers/guides/advanced/confirmation
- Solana RPC websocket methods: https://solana.com/docs/rpc/websocket
- signatureSubscribeMethod: https://solana.com/docs/rpc/websocket/signaturesubscribe
- grpc server streaming: https://grpc.io/docs/what-is-grpc/core-concepts/#server-streaming-rpc
- tonic grpc rust server streaming example: https://github.com/hyperium/tonic/blob/master/examples/src/streaming/server.rs
- solana pub sub client: https://docs.rs/solana-pubsub-client/latest/solana_pubsub_client/pubsub_client/index.html
- the cookbook: https://solana.com/nl/developers/cookbook/development/subscribing-events, specifically this example:
```rust
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