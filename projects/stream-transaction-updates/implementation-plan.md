# Transaction Monitoring Streaming Feature - Implementation Plan

## Step 1: Protocol Buffer Extensions

**Objective**: Extend existing protobuf definitions with enhanced response and streaming messages

### 1.1: Extend SubmitTransactionResponse Message

**Files**: `lib/proto/protosol/solana/transaction/v1/service.proto`

```protobuf
message SubmitTransactionResponse {
  string signature = 1;
  SubmissionResult submission_result = 2;
  optional string error_message = 3;
}

enum SubmissionResult {
  SUBMISSION_RESULT_UNSPECIFIED = 0;
  SUBMITTED = 1;
  FAILED_VALIDATION = 2;
  FAILED_NETWORK_ERROR = 3;
  FAILED_INSUFFICIENT_FUNDS = 4;
  FAILED_INVALID_SIGNATURE = 5;
}
```

### 1.2: Add MonitorTransaction Message Definitions

**Files**: `lib/proto/protosol/solana/transaction/v1/service.proto`

```protobuf
message MonitorTransactionRequest {
  string signature = 1;
  protosol.solana.type.v1.CommitmentLevel commitment_level = 2;
  bool include_logs = 3;
  optional uint32 timeout_seconds = 4;
}

message MonitorTransactionResponse {
  string signature = 1;
  TransactionStatus status = 2;
  optional uint64 slot = 3;
  optional string error_message = 4;
  repeated string logs = 5;
  optional uint64 compute_units_consumed = 6;
  protosol.solana.type.v1.CommitmentLevel current_commitment = 7;
}

enum TransactionStatus {
  TRANSACTION_STATUS_UNSPECIFIED = 0;
  RECEIVED = 1;
  PROCESSED = 2;
  CONFIRMED = 3;
  FINALIZED = 4;
  FAILED = 5;
  DROPPED = 6;
  TIMEOUT = 7;
}
```

### 1.3: Add MonitorTransaction RPC Method

**Files**: `lib/proto/protosol/solana/transaction/v1/service.proto`

```protobuf
service Service {
  // Existing methods...
  rpc MonitorTransaction(MonitorTransactionRequest) returns (stream MonitorTransactionResponse);
}
```

### 1.4: Validate and Generate Code

```bash
buf lint
./scripts/code-gen/generate/all.sh
```

**Validation**: Verify generated Rust, Go, and TypeScript code compiles without errors

## Step 2: Rust Dependency Management

**Objective**: Add required WebSocket and streaming dependencies to Rust backend

### 2.1: Update Cargo.toml Dependencies

**Files**: `api/Cargo.toml`

```toml
[dependencies]
# Existing dependencies...
solana-pubsub-client = "1.18"
futures-util = "0.3"
tokio-stream = "0.1"
tokio = { version = "1.0", features = ["full"] }
dashmap = "5.5"
uuid = { version = "1.0", features = ["v4"] }
```

### 2.2: Verify Dependency Compatibility

```bash
cd api
cargo check
cargo test --lib
```

**Validation**: Ensure all dependencies resolve and existing tests pass

## Step 3: Enhanced SubmitTransaction Implementation

**Objective**: Modify existing SubmitTransaction to return enhanced response

### 3.1: Update SubmitTransaction Return Type

**Files**: `api/src/api/transaction/v1/service_impl.rs`

**Changes**:
- Modify `submit_transaction` method signature
- Add submission result classification logic
- Maintain existing validation and signing checks
- Add error categorization for different failure modes

```rust
async fn submit_transaction(
    &self,
    request: Request<SubmitTransactionRequest>,
) -> Result<Response<SubmitTransactionResponse>, Status> {
    // Existing validation logic...
    
    let submission_result = match self.rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(&solana_transaction, commitment) {
        Ok(signature) => SubmissionResult::Submitted,
        Err(e) => classify_submission_error(e),
    };
    
    Ok(Response::new(SubmitTransactionResponse {
        signature: signature.to_string(),
        submission_result: submission_result.into(),
        error_message: if submission_result != SubmissionResult::Submitted {
            Some(error_msg)
        } else {
            None
        },
    }))
}
```

### 3.2: Add Error Classification Function

**Files**: `api/src/api/transaction/v1/service_impl.rs`

```rust
fn classify_submission_error(error: solana_client::client_error::ClientError) -> SubmissionResult {
    // Analyze error types and map to appropriate SubmissionResult
}
```

### 3.3: Integration Testing

**Files**: `tests/go/composable_e2e_test.go`

Add test cases for enhanced SubmitTransaction response validation:
- Successful submission returns SUBMITTED
- Invalid transaction returns appropriate failure code
- Network errors return FAILED_NETWORK_ERROR

```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestComposableE2ESuite/Test_Enhanced_SubmitTransaction"
```

## Step 4: WebSocket Manager Infrastructure

**Objective**: Create WebSocket connection management and subscription handling

### 4.1: Create WebSocket Manager Module

**Files**: `api/src/websocket/mod.rs`, `api/src/websocket/manager.rs`

```rust
use solana_pubsub_client::PubsubClient;
use dashmap::DashMap;
use tokio::sync::mpsc;

pub struct WebSocketManager {
    pubsub_client: Arc<PubsubClient>,
    active_subscriptions: Arc<DashMap<String, SubscriptionHandle>>,
}

struct SubscriptionHandle {
    subscription_id: u64,
    sender: mpsc::UnboundedSender<MonitorTransactionResponse>,
    abort_handle: tokio::task::AbortHandle,
}

impl WebSocketManager {
    pub async fn new(ws_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pubsub_client = PubsubClient::new(ws_url).await?;
        // Implementation...
    }

    pub async fn subscribe_to_signature(
        &self,
        signature: String,
        commitment_level: CommitmentLevel,
        include_logs: bool,
    ) -> Result<mpsc::UnboundedReceiver<MonitorTransactionResponse>, Status> {
        // Subscription implementation...
    }
}
```

### 4.2: Add WebSocket Manager to Service Providers

**Files**: `api/src/service_providers/service_providers.rs`

```rust
pub struct ServiceProviders {
    pub solana_clients: Arc<SolanaClientsServiceProviders>,
    pub websocket_manager: Arc<WebSocketManager>,
    pub config: Config,
}
```

### 4.3: Unit Testing for WebSocket Manager

**Files**: `api/src/websocket/tests.rs`

```bash
cargo test --lib websocket::tests
```

## Step 5: gRPC Streaming Implementation

**Objective**: Implement MonitorTransaction streaming RPC method

### 5.1: Add MonitorTransaction Method

**Files**: `api/src/api/transaction/v1/service_impl.rs`

```rust
type MonitorTransactionStream = ReceiverStream<Result<MonitorTransactionResponse, Status>>;

async fn monitor_transaction(
    &self,
    request: Request<MonitorTransactionRequest>,
) -> Result<Response<Self::MonitorTransactionStream>, Status> {
    let req = request.into_inner();
    
    // Validate request parameters
    validate_signature_format(&req.signature)?;
    
    // Create response stream
    let (tx, rx) = mpsc::channel(100);
    
    // Get WebSocket subscription
    let subscription_rx = self.websocket_manager
        .subscribe_to_signature(
            req.signature.clone(),
            req.commitment_level(),
            req.include_logs,
        ).await?;
    
    // Spawn task to bridge WebSocket to gRPC stream
    let signature = req.signature.clone();
    tokio::spawn(async move {
        bridge_websocket_to_grpc_stream(signature, subscription_rx, tx).await;
    });
    
    Ok(Response::new(ReceiverStream::new(rx)))
}
```

### 5.2: Add Stream Bridging Function

**Files**: `api/src/api/transaction/v1/streaming.rs`

```rust
async fn bridge_websocket_to_grpc_stream(
    signature: String,
    mut subscription_rx: mpsc::UnboundedReceiver<MonitorTransactionResponse>,
    tx: mpsc::Sender<Result<MonitorTransactionResponse, Status>>,
) {
    // Bridge implementation with error handling and cleanup
}
```

### 5.3: Add Stream Type Definition

**Files**: `api/src/api/transaction/v1/service_impl.rs`

```rust
impl TransactionService for TransactionServiceImpl {
    type MonitorTransactionStream = ReceiverStream<Result<MonitorTransactionResponse, Status>>;
    // Other implementations...
}
```

### 5.4: Integration Testing for Streaming

**Files**: `tests/go/streaming_e2e_test.go`

```go
func (suite *StreamingE2ETestSuite) Test_MonitorTransaction_Success() {
    // Create and submit transaction
    submitResp := // ... submit transaction
    
    // Start monitoring stream
    stream, err := suite.transactionService.MonitorTransaction(suite.ctx, &transaction_v1.MonitorTransactionRequest{
        Signature: submitResp.Signature,
        CommitmentLevel: type_v1.CommitmentLevel_COMMITMENT_LEVEL_CONFIRMED,
        IncludeLogs: true,
    })
    
    // Receive stream updates until completion
    for {
        update, err := stream.Recv()
        if err == io.EOF {
            break
        }
        // Assert update progression
    }
}
```

```bash
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestStreamingE2ESuite"
```

## Step 6: Service Integration and Wiring

**Objective**: Wire all components together in the service architecture

### 6.1: Update Service Provider Construction

**Files**: `api/src/service_providers/service_providers.rs`

```rust
impl ServiceProviders {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let solana_clients = Arc::new(SolanaClientsServiceProviders::new(&config)?);
        
        let ws_url = derive_websocket_url_from_rpc(&config.solana_rpc_url)?;
        let websocket_manager = Arc::new(WebSocketManager::new(&ws_url).await?);
        
        Ok(ServiceProviders {
            solana_clients,
            websocket_manager,
            config,
        })
    }
}
```

### 6.2: Update TransactionServiceImpl Constructor

**Files**: `api/src/api/transaction/v1/service_impl.rs`

```rust
impl TransactionServiceImpl {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        websocket_manager: Arc<WebSocketManager>,
    ) -> Self {
        Self { 
            rpc_client,
            websocket_manager,
        }
    }
}
```

### 6.3: Update Service Registration

**Files**: `api/src/api/api.rs`

```rust
pub fn create_transaction_service(
    service_providers: Arc<ServiceProviders>
) -> TransactionServiceImpl {
    TransactionServiceImpl::new(
        service_providers.solana_clients.get_rpc_client(),
        service_providers.websocket_manager.clone(),
    )
}
```

### 6.4: Integration Testing

```bash
cargo run --package protosol-solana-api
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v
```

## Step 7: Resource Management and Cleanup

**Objective**: Implement proper resource cleanup and memory management

### 7.1: Add Subscription Lifecycle Management

**Files**: `api/src/websocket/manager.rs`

```rust
impl WebSocketManager {
    pub async fn cleanup_expired_subscriptions(&self) {
        // Remove expired subscriptions
    }
    
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Graceful shutdown of all subscriptions
    }
}
```

### 7.2: Add Periodic Cleanup Task

**Files**: `api/src/main.rs`

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Existing setup...
    
    // Start cleanup task
    let websocket_manager = service_providers.websocket_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            websocket_manager.cleanup_expired_subscriptions().await;
        }
    });
    
    // Start server...
}
```

### 7.3: Add Graceful Shutdown

**Files**: `api/src/main.rs`

```rust
// Graceful shutdown on SIGTERM/SIGINT
tokio::select! {
    _ = server => {},
    _ = tokio::signal::ctrl_c() => {
        println!("Shutting down gracefully...");
        service_providers.websocket_manager.shutdown().await?;
    }
}
```

### 7.4: Memory Usage Testing

**Files**: `tests/go/resource_test.go`

```go
func (suite *ResourceTestSuite) Test_Memory_Usage_Under_Load() {
    // Test concurrent subscriptions
    // Monitor memory usage
    // Validate cleanup
}
```

## Step 8: Comprehensive Integration Testing

**Objective**: Ensure complete system functionality with comprehensive test coverage

### 8.1: End-to-End Workflow Testing

**Files**: `tests/go/complete_workflow_test.go`

```go
func (suite *WorkflowTestSuite) Test_Submit_And_Monitor_Success() {
    // 1. Submit transaction
    submitResp := // ...
    suite.Assert().Equal(transaction_v1.SubmissionResult_SUBMITTED, submitResp.SubmissionResult)
    
    // 2. Monitor transaction
    stream := // ... start monitoring
    
    // 3. Validate progression through states
    var finalStatus transaction_v1.TransactionStatus
    for update := range stream {
        finalStatus = update.Status
        if update.Status == transaction_v1.TransactionStatus_CONFIRMED {
            break
        }
    }
    
    suite.Assert().Equal(transaction_v1.TransactionStatus_CONFIRMED, finalStatus)
}

func (suite *WorkflowTestSuite) Test_Failed_Submit_No_Monitoring() {
    // Test failed submission doesn't require monitoring
}
```

### 8.2: Concurrent Monitoring Testing

**Files**: `tests/go/concurrency_test.go`

```go
func (suite *ConcurrencyTestSuite) Test_Multiple_Concurrent_Monitors() {
    // Submit multiple transactions concurrently
    // Monitor all simultaneously  
    // Validate all complete correctly
}
```

### 8.3: Error Scenario Testing

**Files**: `tests/go/error_scenarios_test.go`

```go
func (suite *ErrorTestSuite) Test_Network_Interruption_Recovery() {
    // Start monitoring
    // Simulate network interruption
    // Validate recovery behavior
}

func (suite *ErrorTestSuite) Test_Invalid_Signature_Handling() {
    // Test invalid signature format
    // Validate error response
}
```

### 8.4: Performance Testing

**Files**: `tests/go/performance_test.go`

```go
func (suite *PerformanceTestSuite) Test_Response_Time_Requirements() {
    // Measure initial response time < 100ms
    // Measure stream update latency
    // Validate cleanup time < 1 second
}
```

### 8.5: Full Test Suite Execution

```bash
# Run all integration tests
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v ./...

# Run specific test suites
RUN_INTEGRATION_TESTS=1 go test -v -run "TestWorkflowSuite"
RUN_INTEGRATION_TESTS=1 go test -v -run "TestConcurrencySuite" 
RUN_INTEGRATION_TESTS=1 go test -v -run "TestErrorTestSuite"
RUN_INTEGRATION_TESTS=1 go test -v -run "TestPerformanceTestSuite"
```

## Step 9: Build Verification and Deployment Preparation

**Objective**: Ensure complete build success and deployment readiness

### 9.1: Clean Build Verification

```bash
# Clean and regenerate all code
./scripts/code-gen/clean/all.sh
./scripts/code-gen/generate/all.sh

# Verify Rust build
cd api
cargo clean
cargo build --release
cargo test --release

# Verify Go build  
cd ../lib/go
go mod tidy
go build ./...

# Verify TypeScript build
cd ../ts
npm install
npm run build
```

### 9.2: Integration Test Validation

```bash
# Start services
./scripts/tests/start-validator.sh    # Terminal 1
./scripts/tests/start-backend.sh      # Terminal 2

# Full integration test suite
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -timeout 10m ./...
```

### 9.3: Performance Baseline Measurement

```bash
# Measure baseline performance
cd tests/go
RUN_INTEGRATION_TESTS=1 go test -v -run "TestPerformance" -bench=.
```

### 9.4: Documentation Update

**Files**: `CLAUDE.md`

Add new feature documentation:
- MonitorTransaction RPC method usage
- Enhanced SubmitTransaction response handling
- WebSocket integration patterns
- Performance characteristics

### 9.5: Final Validation Checklist

- [ ] All protobuf definitions validated with `buf lint`
- [ ] All generated code compiles without warnings
- [ ] Enhanced SubmitTransaction returns correct response format
- [ ] MonitorTransaction streaming works end-to-end
- [ ] WebSocket subscriptions properly managed and cleaned up
- [ ] Integration tests achieve 100% pass rate
- [ ] Performance requirements met (< 100ms initial response)
- [ ] Memory usage within limits (< 50MB + 1KB per subscription)
- [ ] Graceful shutdown implemented and tested
- [ ] Error scenarios properly handled and tested

## Technical Dependencies and Build Ordering

### Build Dependencies

1. **Proto Changes** → Code Generation → Rust Dependencies
2. **WebSocket Manager** → Service Integration → Streaming Implementation  
3. **Enhanced SubmitTransaction** → MonitorTransaction → Integration Testing
4. **Unit Tests** → Integration Tests → Performance Tests

### Critical Integration Points

- WebSocket URL derivation from RPC configuration
- Proper commitment level mapping between proto and Solana types
- Stream lifecycle management in tonic framework
- Error propagation from WebSocket to gRPC status codes

### Resource Management Requirements

- Subscription cleanup on client disconnect
- Memory bounds on message buffering  
- Connection pooling for WebSocket endpoints
- Timeout handling for abandoned streams

Each step builds incrementally with strong testing validation, ensuring no broken states and maintaining backward compatibility throughout the implementation process.