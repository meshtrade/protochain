# Transaction Monitoring Streaming Feature - Technical Requirements

## Protocol Buffer Message Definitions

### Enhanced SubmitTransactionResponse

```protobuf
message SubmitTransactionResponse {
  string signature = 1;
  SubmissionResult submission_result = 2;
  optional string error_message = 3;
}

enum SubmissionResult {
  SUBMISSION_RESULT_UNSPECIFIED = 0;
  SUBMITTED = 1;                  // Transaction successfully submitted to network
  FAILED_VALIDATION = 2;          // Transaction failed pre-submission validation
  FAILED_NETWORK_ERROR = 3;       // Network/RPC error prevented submission
  FAILED_INSUFFICIENT_FUNDS = 4;  // Fee payer has insufficient balance
  FAILED_INVALID_SIGNATURE = 5;   // Transaction signature validation failed
}
```

### MonitorTransaction Request/Response Messages

```protobuf
message MonitorTransactionRequest {
  string signature = 1;                                               // Transaction signature to monitor
  protosol.solana.type.v1.CommitmentLevel commitment_level = 2;       // Target commitment level
  bool include_logs = 3;                                              // Include program execution logs
  optional uint32 timeout_seconds = 4;                               // Monitor timeout (default: 60)
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
  RECEIVED = 1;           // Transaction received by validator
  PROCESSED = 2;          // Transaction processed (commitment: processed)
  CONFIRMED = 3;          // Transaction confirmed (commitment: confirmed)
  FINALIZED = 4;          // Transaction finalized (commitment: finalized)
  FAILED = 5;             // Transaction failed during execution
  DROPPED = 6;            // Transaction dropped from network
  TIMEOUT = 7;            // Monitoring timeout reached
}
```

## gRPC Service Method Signatures

### Service Definition Extension

```protobuf
service Service {
  // Existing methods...
  rpc CompileTransaction(CompileTransactionRequest) returns (CompileTransactionResponse);
  rpc SubmitTransaction(SubmitTransactionRequest) returns (SubmitTransactionResponse);
  
  // New streaming method
  rpc MonitorTransaction(MonitorTransactionRequest) returns (stream MonitorTransactionResponse);
}
```

### Streaming Pattern Specifications

- **Stream Type**: Server-side streaming (unidirectional)
- **Stream Duration**: Until target commitment level reached or timeout
- **Message Frequency**: Real-time updates as transaction status changes
- **Termination Conditions**:
  - Target commitment level achieved
  - Transaction failure detected
  - Timeout exceeded
  - Client cancellation
  - WebSocket connection error

## WebSocket Integration Architecture

### PubsubClient Integration

```rust
use solana_client::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcTransactionLogsFilter;

struct WebSocketManager {
    pubsub_client: Arc<PubsubClient>,
    signature_subscriptions: Arc<Mutex<HashMap<String, SubscriptionHandle>>>,
}

struct SubscriptionHandle {
    subscription_id: u64,
    sender: mpsc::UnboundedSender<MonitorTransactionResponse>,
    abort_handle: AbortHandle,
}
```

### WebSocket Connection Parameters

- **Endpoint URL**: Derived from RPC client configuration
- **Connection Timeout**: 30 seconds
- **Reconnection Strategy**: Exponential backoff (max 3 retries)
- **Subscription Cleanup**: Automatic unsubscribe on stream termination
- **Error Propagation**: WebSocket errors converted to gRPC Status codes

## Error Handling Requirements

### Status Code Mapping

| Error Condition | gRPC Status Code | Description |
|-----------------|------------------|-------------|
| Invalid signature format | INVALID_ARGUMENT | Malformed signature string |
| WebSocket connection failed | UNAVAILABLE | Network connectivity issues |
| Subscription timeout | DEADLINE_EXCEEDED | No response within timeout |
| Solana network error | INTERNAL | RPC/blockchain errors |
| Rate limiting | RESOURCE_EXHAUSTED | Too many active subscriptions |
| Service shutdown | CANCELLED | Server shutdown requested |

### Error Recovery Patterns

- **WebSocket Reconnection**: Automatic retry with exponential backoff
- **Subscription Recreation**: Re-establish signature monitoring on connection recovery
- **Timeout Handling**: Graceful stream termination with timeout status
- **Resource Cleanup**: Ensure WebSocket subscriptions are properly closed

## Performance Criteria

### Resource Constraints

- **Maximum Concurrent Subscriptions**: 1000 per server instance
- **Memory Usage**: 50MB baseline + 1KB per active subscription
- **WebSocket Connections**: Single shared connection per Solana endpoint
- **Message Buffering**: 100 message buffer per stream to handle bursts

### Response Time Requirements

- **Initial Response**: < 100ms from request to first message
- **Status Updates**: Real-time propagation from WebSocket events
- **Stream Termination**: < 500ms from condition to final message
- **Resource Cleanup**: < 1 second after stream completion

## Integration Testing Requirements

### Test Coverage Areas

1. **Enhanced SubmitTransaction Flow**
   - Validate submission result enumeration
   - Test error condition handling
   - Verify immediate response feedback

2. **MonitorTransaction Streaming**
   - Full transaction lifecycle monitoring
   - Commitment level progression tracking
   - Timeout and cancellation handling
   - Multiple concurrent subscriptions

3. **WebSocket Integration**
   - Connection establishment and recovery
   - Subscription lifecycle management
   - Error propagation from WebSocket to gRPC

4. **End-to-End Workflow**
   - Submit transaction → Monitor → Receive confirmation
   - Failed submission → No monitoring required
   - Network interruption → Stream recovery

### Validation Criteria

- **Functional**: All transaction statuses correctly reported
- **Performance**: Response times within specified limits  
- **Reliability**: Stream recovery after network failures
- **Resource**: No memory leaks from abandoned subscriptions
- **Concurrency**: Multiple clients monitoring simultaneously

## Field Requirements Specification

### Required Fields

All message fields marked as required must be populated:
- `MonitorTransactionRequest.signature`: Transaction signature string
- `MonitorTransactionRequest.commitment_level`: Target commitment level
- `MonitorTransactionResponse.signature`: Monitored signature
- `MonitorTransactionResponse.status`: Current transaction status

### Optional Fields

Optional fields provide enhanced monitoring capabilities:
- `MonitorTransactionRequest.include_logs`: Program execution logs
- `MonitorTransactionRequest.timeout_seconds`: Custom timeout duration
- `MonitorTransactionResponse.logs`: Execution logs (if requested)
- `MonitorTransactionResponse.compute_units_consumed`: Resource usage

### Field Validation Rules

- **Signature**: Must be valid Base58-encoded 64-byte string
- **Commitment Level**: Must be valid enum value (PROCESSED, CONFIRMED, FINALIZED)
- **Timeout**: Range 5-300 seconds (default: 60)
- **Logs**: Maximum 10MB combined log size per transaction

## Resource Management Requirements

### WebSocket Connection Management

- **Connection Pooling**: Single connection per Solana RPC endpoint
- **Connection Health Monitoring**: Periodic ping/pong for liveness detection
- **Graceful Shutdown**: Close all subscriptions before connection termination
- **Connection Recovery**: Automatic reconnection with subscription restoration

### Memory Management

- **Subscription Tracking**: Efficient HashMap for signature-to-subscription mapping
- **Message Buffering**: Bounded channels to prevent memory exhaustion
- **Cleanup Triggers**: Automatic cleanup on timeout, cancellation, or completion
- **Resource Limits**: Configurable limits on concurrent subscriptions

### Concurrency Control

- **Thread Safety**: All subscription management operations thread-safe
- **Subscription Isolation**: Independent failure handling per subscription
- **Backpressure Handling**: Client stream backpressure does not affect other streams
- **Race Condition Prevention**: Atomic operations for subscription lifecycle