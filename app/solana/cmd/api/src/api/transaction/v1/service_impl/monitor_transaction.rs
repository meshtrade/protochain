use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

use protochain_api::protochain::solana::r#type::v1::CommitmentLevel;
use protochain_api::protochain::solana::transaction::v1::{
    MonitorTransactionRequest, MonitorTransactionResponse, TransactionStatus,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::TransactionServiceImpl {
    /// Monitors a transaction for real-time status changes via WebSocket streaming
    ///
    /// This method establishes a persistent gRPC server streaming connection that pushes
    /// transaction status updates from the Solana blockchain in real-time. It bridges
    /// WebSocket pubsub notifications to gRPC streaming protocol.
    ///
    /// Networking Architecture:
    /// 1. Validates input parameters and signature format
    /// 2. Creates unbounded WebSocket subscription via `WebSocketManager`
    /// 3. Establishes bounded gRPC stream channel (capacity: 100)
    /// 4. Spawns async bridge task for protocol translation
    /// 5. Returns `ReceiverStream` for client consumption
    ///
    /// Resource Management:
    /// - WebSocket subscription auto-cleanup on client disconnect
    /// - Bridge task terminates on terminal status or client disconnect
    /// - Bounded channel prevents memory exhaustion from fast updates
    ///
    /// Error Handling:
    /// - Input validation prevents malformed signature attacks
    /// - Timeout bounds prevent resource exhaustion (5-300 seconds)
    /// - Channel failures trigger automatic cleanup
    pub(super) fn handle_monitor_transaction(
        &self,
        request: Request<MonitorTransactionRequest>,
    ) -> Result<Response<ReceiverStream<Result<MonitorTransactionResponse, Status>>>, Status> {
        let req = request.into_inner();

        // Validate signature format
        if req.signature.is_empty() {
            error!("MonitorTransaction called with empty signature");
            return Err(Status::invalid_argument("Transaction signature is required"));
        }

        // Parse signature to validate format
        req.signature
            .parse::<solana_sdk::signature::Signature>()
            .map_err(|_| {
                error!(
                    signature = %req.signature,
                    "Invalid signature format provided to MonitorTransaction"
                );
                Status::invalid_argument("Invalid signature format")
            })?;

        // Validate commitment level
        let commitment_level = CommitmentLevel::try_from(req.commitment_level).map_err(|_| {
            error!(
                commitment_level = req.commitment_level,
                signature = %req.signature,
                "Invalid commitment level provided to MonitorTransaction"
            );
            Status::invalid_argument("Invalid commitment level")
        })?;

        // Validate timeout (if provided)
        let timeout_seconds = if req.timeout_seconds == 0 {
            60
        } else {
            req.timeout_seconds
        };
        if !(5..=300).contains(&timeout_seconds) {
            error!(
                timeout_seconds = timeout_seconds,
                signature = %req.signature,
                "Invalid timeout value provided to MonitorTransaction"
            );
            return Err(Status::invalid_argument("Timeout must be between 5 and 300 seconds"));
        }

        info!(
            signature = %req.signature,
            commitment_level = ?commitment_level,
            timeout_seconds = timeout_seconds,
            include_logs = req.include_logs,
            "Starting transaction monitoring"
        );

        // Create response stream channel with bounded capacity
        // Buffer size 100 provides good balance between memory usage and throughput
        // This prevents unbounded memory growth if client consumes slowly
        let (tx, rx) = mpsc::channel(100);

        // Subscribe to signature updates via WebSocket manager
        let websocket_rx = match self.websocket_manager.subscribe_to_signature(
            &req.signature,
            commitment_level,
            req.include_logs,
            Some(timeout_seconds),
        ) {
            Ok(rx) => rx,
            Err(e) => {
                return Err(*e);
            }
        };

        // Spawn task to bridge WebSocket updates to gRPC stream
        // This task handles protocol translation between WebSocket pubsub and gRPC streaming
        let signature_for_task = req.signature.clone();
        tokio::spawn(async move {
            bridge_websocket_to_grpc_stream(signature_for_task, websocket_rx, tx, timeout_seconds)
                .await;
        });

        info!(
            signature = %req.signature,
            commitment_level = ?commitment_level,
            "Transaction monitoring stream established"
        );

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

/// Helper function to check if a transaction status is terminal
const fn is_terminal_status(status: TransactionStatus) -> bool {
    matches!(
        status,
        TransactionStatus::Confirmed
            | TransactionStatus::Finalized
            | TransactionStatus::Failed
            | TransactionStatus::Dropped
            | TransactionStatus::Timeout
    )
}

/// Helper function to send timeout notification to gRPC client
async fn send_timeout_notification(
    grpc_tx: &mpsc::Sender<Result<MonitorTransactionResponse, Status>>,
    signature: &str,
) {
    let timeout_response = MonitorTransactionResponse {
        signature: signature.to_string(),
        status: TransactionStatus::Timeout.into(),
        slot: 0,
        error_message: "Stream monitoring timeout reached".to_string(),
        logs: vec![],
        compute_units_consumed: 0,
        current_commitment: CommitmentLevel::Unspecified.into(),
    };

    // Best effort - ignore if client already disconnected
    if grpc_tx.send(Ok(timeout_response)).await.is_err() {
        debug!(
            signature = %signature,
            "Client disconnected before timeout notification could be sent"
        );
    }
}

/// Bridges WebSocket subscription updates to gRPC streaming response
///
/// This function performs critical protocol translation between Solana WebSocket pubsub
/// and gRPC server streaming. It handles proper resource cleanup and prevents memory leaks.
///
/// Architecture:
/// - Receives updates from unbounded WebSocket channel (real-time blockchain events)
/// - Translates to bounded gRPC stream channel (client consumption rate-limited)
/// - Implements timeout-based cleanup to prevent zombie tasks
/// - Detects client disconnections for immediate resource cleanup
///
/// Resource Management:
/// - Uses timeout to prevent indefinite hanging on stalled WebSocket
/// - Detects gRPC channel closure (client disconnect) for immediate cleanup
/// - Terminates on terminal transaction states to free resources
/// - No explicit drop needed - channels auto-cleanup when task ends
///
/// Memory Safety:
/// - No heap allocations in hot path (only stack-based message passing)
/// - Clone operations are minimal (only for logging)
/// - Task automatically terminates preventing memory leaks
async fn bridge_websocket_to_grpc_stream(
    signature: String,
    mut websocket_rx: tokio::sync::mpsc::UnboundedReceiver<MonitorTransactionResponse>,
    grpc_tx: mpsc::Sender<Result<MonitorTransactionResponse, Status>>,
    timeout_seconds: u32,
) {
    debug!(
        signature = %signature,
        timeout_seconds = timeout_seconds,
        "Starting stream bridge"
    );

    let bridge_timeout = Duration::from_secs(u64::from(timeout_seconds) + 5); // Add 5s buffer

    // Use timeout to prevent indefinite hanging if WebSocket stops responding
    let bridge_result = timeout(bridge_timeout, async {
        while let Some(response) = websocket_rx.recv().await {
            debug!(
                signature = %signature,
                status = ?response.status(),
                slot = response.slot,
                "Received WebSocket update"
            );

            // Try to send to gRPC client - if this fails, client has disconnected
            if matches!(grpc_tx.send(Ok(response.clone())).await, Ok(())) {
                // Successfully sent to client
            } else {
                info!(
                    signature = %signature,
                    "Client disconnected (gRPC channel closed)"
                );
                return; // Early return - no need to continue processing
            }

            // Check if this is a terminal status that should end the stream
            if is_terminal_status(response.status()) {
                info!(
                    signature = %signature,
                    status = ?response.status(),
                    slot = response.slot,
                    "Terminal status reached"
                );
                return; // End stream on terminal status
            }
        }

        // WebSocket channel closed (sender dropped)
        debug!(
            signature = %signature,
            "WebSocket stream ended (sender closed)"
        );
    })
    .await;

    if bridge_result == Ok(()) {
        debug!(
            signature = %signature,
            "Stream bridge completed normally"
        );
    } else {
        warn!(
            signature = %signature,
            timeout_seconds = timeout_seconds + 5,
            "Stream bridge timed out"
        );
        // Send timeout notification to client if channel is still open
        send_timeout_notification(&grpc_tx, &signature).await;
    }
}
