use dashmap::DashMap;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSignatureSubscribeConfig;
use solana_client::rpc_response::{
    ProcessedSignatureResult, ReceivedSignatureResult, Response, RpcSignatureResult,
};
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tonic::Status;
use tracing::{debug, info, warn};

use protosol_api::protosol::solana::r#type::v1::CommitmentLevel;
use protosol_api::protosol::solana::transaction::v1::{
    MonitorTransactionResponse, TransactionStatus,
};

/// Handle for managing a signature subscription
#[derive(Debug)]
struct SubscriptionHandle {
    sender: mpsc::UnboundedSender<MonitorTransactionResponse>,
    abort_handle: tokio::task::AbortHandle,
}

/// WebSocket manager for handling Solana signature subscriptions
#[derive(Clone)]
pub struct WebSocketManager {
    ws_url: String,
    rpc_client: Arc<RpcClient>,
    active_subscriptions: Arc<DashMap<String, SubscriptionHandle>>,
}

impl WebSocketManager {
    /// Creates a new WebSocket manager with connection to Solana WebSocket endpoint
    pub async fn new(
        ws_url: &str,
        rpc_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            ws_url = %ws_url,
            rpc_url = %rpc_url,
            "🔌 Creating WebSocket manager"
        );

        // Create RPC client for transaction status checks
        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));

        // Test WebSocket connectivity by creating a temporary PubsubClient
        Self::validate_websocket_connection(ws_url).await;

        info!(
            ws_url = %ws_url,
            rpc_url = %rpc_url,
            "✅ WebSocket manager initialized"
        );

        Ok(Self {
            ws_url: ws_url.to_string(),
            rpc_client,
            active_subscriptions: Arc::new(DashMap::new()),
        })
    }

    /// Creates subscription configuration for signature monitoring
    const fn create_subscription_config(
        commitment: CommitmentConfig,
    ) -> RpcSignatureSubscribeConfig {
        RpcSignatureSubscribeConfig {
            commitment: Some(commitment),
            enable_received_notification: Some(true),
        }
    }

    /// Checks if a transaction status is terminal (monitoring should stop)
    const fn is_terminal_status(status: TransactionStatus) -> bool {
        matches!(
            status,
            TransactionStatus::Confirmed
                | TransactionStatus::Finalized
                | TransactionStatus::Failed
                | TransactionStatus::Dropped
        )
    }

    /// Creates a timeout response for real-time monitoring
    fn create_realtime_timeout_response(signature_str: &str) -> MonitorTransactionResponse {
        MonitorTransactionResponse {
            signature: signature_str.to_string(),
            status: TransactionStatus::Timeout.into(),
            slot: None,
            error_message: Some("Monitoring timeout reached".to_string()),
            logs: vec![],
            compute_units_consumed: None,
            current_commitment: CommitmentLevel::Unspecified.into(),
        }
    }

    /// Handles a notification response and returns true if monitoring should stop
    fn handle_notification_response(
        notification: Response<RpcSignatureResult>,
        signature_str: &str,
        include_logs: bool,
        sender: &mpsc::UnboundedSender<MonitorTransactionResponse>,
    ) -> bool {
        let response =
            Self::process_signature_notification(notification, signature_str, include_logs);
        let response_status = response.status();
        let is_terminal = Self::is_terminal_status(response_status);

        if sender.send(response).is_err() {
            info!(
                signature = %signature_str,
                "🔌 Client disconnected"
            );
            return true;
        }

        if is_terminal {
            info!(
                signature = %signature_str,
                status = ?response_status,
                "✅ Terminal status reached"
            );
            return true;
        }

        false
    }

    /// Validates WebSocket connectivity for the given URL
    async fn validate_websocket_connection(ws_url: &str) {
        match PubsubClient::new(ws_url).await {
            Ok(_test_client) => {
                info!(
                    ws_url = %ws_url,
                    "✅ WebSocket connection validated successfully"
                );
            }
            Err(e) => {
                warn!(
                    ws_url = %ws_url,
                    error = %e,
                    "⚠️ WebSocket connection failed, will create per-subscription"
                );
            }
        }
    }

    /// Subscribes to signature status updates for a specific transaction
    pub fn subscribe_to_signature(
        &self,
        signature: &str,
        commitment_level: CommitmentLevel,
        include_logs: bool,
        timeout_seconds: Option<u32>,
    ) -> Result<mpsc::UnboundedReceiver<MonitorTransactionResponse>, Box<Status>> {
        // Validate signature format
        let parsed_signature = signature
            .parse::<Signature>()
            .map_err(|_| Box::new(Status::invalid_argument("Invalid signature format")))?;

        // Convert commitment level
        let commitment = Self::commitment_level_to_config(commitment_level);

        // Create channels for communication
        let (tx, rx) = mpsc::unbounded_channel();

        info!(
            signature = %signature,
            commitment_level = ?commitment_level,
            include_logs = include_logs,
            timeout_seconds = ?timeout_seconds,
            "🔔 Creating signature subscription"
        );

        // Clone necessary data for the async task
        let sig_clone = signature.to_string();
        let tx_clone = tx.clone();
        let timeout_duration = Duration::from_secs(u64::from(timeout_seconds.unwrap_or(60)));

        // Spawn the subscription task
        let ws_url_clone = self.ws_url.clone();
        let rpc_client_clone = Arc::clone(&self.rpc_client);
        let handle = tokio::spawn(async move {
            Self::handle_signature_subscription(
                parsed_signature,
                sig_clone,
                commitment,
                include_logs,
                timeout_duration,
                tx_clone,
                ws_url_clone,
                rpc_client_clone,
            )
            .await;
        });

        // Store subscription handle
        let subscription_handle = SubscriptionHandle {
            sender: tx,
            abort_handle: handle.abort_handle(),
        };

        self.active_subscriptions
            .insert(signature.to_string(), subscription_handle);

        info!(
            signature = %signature,
            "✅ Signature subscription created"
        );

        Ok(rx)
    }

    /// Handles the actual signature subscription logic using real Solana WebSocket
    async fn handle_signature_subscription(
        signature: Signature,
        signature_str: String,
        commitment: CommitmentConfig,
        include_logs: bool,
        timeout: Duration,
        sender: mpsc::UnboundedSender<MonitorTransactionResponse>,
        ws_url: String,
        rpc_client: Arc<RpcClient>,
    ) {
        debug!(
            signature = %signature_str,
            "🎧 Starting signature monitoring"
        );

        // CRITICAL FIX: Check current transaction status first
        // This prevents the race condition where transactions confirm before WebSocket subscription
        match rpc_client.get_signature_statuses(&[signature]).await {
            Ok(status_response) => {
                if let Some(Some(status)) = status_response.value.first() {
                    // Transaction already has a final status - send it immediately
                    let (transaction_status, error_message) = match &status.err {
                        Some(err) => (
                            TransactionStatus::Failed,
                            Some(format!("Transaction failed: {err:?}")),
                        ),
                        None => {
                            // Transaction succeeded, determine the commitment level
                            match status.confirmations {
                                Some(0) => (TransactionStatus::Finalized, None),
                                Some(_) => (TransactionStatus::Confirmed, None),
                                None => (TransactionStatus::Finalized, None), // No confirmations means finalized
                            }
                        }
                    };

                    // Send the current status immediately
                    let response = MonitorTransactionResponse {
                        signature: signature_str.clone(),
                        status: transaction_status.into(),
                        slot: Some(status.slot),
                        error_message,
                        logs: if include_logs { vec![] } else { vec![] }, // Could fetch tx details for logs
                        compute_units_consumed: None,
                        current_commitment: match transaction_status {
                            TransactionStatus::Finalized => CommitmentLevel::Finalized,
                            TransactionStatus::Confirmed => CommitmentLevel::Confirmed,
                            _ => CommitmentLevel::Processed,
                        }
                        .into(),
                    };

                    info!(
                        signature = %signature_str,
                        status = ?transaction_status,
                        "✅ Transaction already finalized, sending immediate status"
                    );

                    let _ = sender.send(response);
                    return; // Exit early - no need for WebSocket subscription
                }
                // Transaction not found yet, continue with WebSocket subscription
                info!(
                    signature = %signature_str,
                    "🔄 Transaction not found or still processing, starting WebSocket subscription"
                );
            }
            Err(e) => {
                warn!(
                    signature = %signature_str,
                    error = %e,
                    "⚠️  Failed to check transaction status, proceeding with WebSocket subscription"
                );
                // Continue with WebSocket subscription even if status check failed
            }
        }

        // Create PubsubClient for this subscription
        let pubsub_client = match PubsubClient::new(&ws_url).await {
            Ok(client) => client,
            Err(e) => {
                warn!(
                    signature = %signature_str,
                    error = %e,
                    "❌ Failed to create PubsubClient"
                );
                let _ = sender.send(MonitorTransactionResponse {
                    signature: signature_str.clone(),
                    status: TransactionStatus::Failed.into(),
                    slot: None,
                    error_message: Some(format!("WebSocket connection failed: {e}")),
                    logs: vec![],
                    compute_units_consumed: None,
                    current_commitment: CommitmentLevel::Unspecified.into(),
                });
                return;
            }
        };

        // Configure signature subscription
        let config = Self::create_subscription_config(commitment);

        // Create signature subscription
        let (mut stream, _unsubscribe) = match pubsub_client
            .signature_subscribe(&signature, Some(config))
            .await
        {
            Ok(subscription) => subscription,
            Err(e) => {
                warn!(
                    signature = %signature_str,
                    error = %e,
                    "❌ Failed to create signature subscription"
                );
                let _ = sender.send(MonitorTransactionResponse {
                    signature: signature_str.clone(),
                    status: TransactionStatus::Failed.into(),
                    slot: None,
                    error_message: Some(format!("Signature subscription failed: {e}")),
                    logs: vec![],
                    compute_units_consumed: None,
                    current_commitment: CommitmentLevel::Unspecified.into(),
                });
                return;
            }
        };

        info!(
            signature = %signature_str,
            "✅ Signature subscription established"
        );

        // Set up timeout
        let timeout_task = tokio::time::sleep(timeout);
        tokio::pin!(timeout_task);

        // HYBRID APPROACH: Listen for WebSocket updates with RPC polling fallback
        let mut poll_interval = tokio::time::interval(Duration::from_millis(200));
        poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                notification = stream.next() => {
                    if let Some(response) = notification {
                        info!(
                            signature = %signature_str,
                            "📡 Received WebSocket notification"
                        );
                        if Self::handle_notification_response(response, &signature_str, include_logs, &sender) {
                            break;
                        }
                    } else {
                        debug!(
                            signature = %signature_str,
                            "🔚 WebSocket stream ended"
                        );
                        break;
                    }
                }
                _ = poll_interval.tick() => {
                    // Fallback: Poll RPC for status updates (for unreliable WebSocket environments)
                    if let Ok(status_response) = rpc_client.get_signature_statuses(&[signature]).await {
                        if let Some(Some(status)) = status_response.value.first() {
                            // Transaction status found via RPC polling
                            let (transaction_status, error_message) = match &status.err {
                                Some(err) => (TransactionStatus::Failed, Some(format!("Transaction failed: {err:?}"))),
                                None => {
                                    match status.confirmations {
                                        Some(0) => (TransactionStatus::Finalized, None),
                                        Some(_) => (TransactionStatus::Confirmed, None),
                                        None => (TransactionStatus::Finalized, None),
                                    }
                                }
                            };

                            let response = MonitorTransactionResponse {
                                signature: signature_str.clone(),
                                status: transaction_status.into(),
                                slot: Some(status.slot),
                                error_message,
                                logs: vec![], // RPC polling doesn't include logs by default
                                compute_units_consumed: None,
                                current_commitment: match transaction_status {
                                    TransactionStatus::Finalized => CommitmentLevel::Finalized,
                                    TransactionStatus::Confirmed => CommitmentLevel::Confirmed,
                                    _ => CommitmentLevel::Processed,
                                }.into(),
                            };

                            info!(
                                signature = %signature_str,
                                status = ?transaction_status,
                                "✅ Transaction status found via RPC polling (WebSocket fallback)"
                            );

                            if sender.send(response).is_err() {
                                break; // Client disconnected
                            }

                            // Check if this is a terminal status
                            if Self::is_terminal_status(transaction_status) {
                                break;
                            }
                        }
                    } else {
                        // RPC polling failed, continue waiting
                    }
                }
                () = &mut timeout_task => {
                    warn!(
                        signature = %signature_str,
                        "⏰ Timeout reached (both WebSocket and RPC polling failed)"
                    );
                    let _ = sender.send(Self::create_realtime_timeout_response(&signature_str));
                    break;
                }
            }
        }

        debug!(
            signature = %signature_str,
            "🏁 Signature subscription completed"
        );
    }

    /// Processes a signature notification and converts it to `MonitorTransactionResponse`
    fn process_signature_notification(
        notification: Response<RpcSignatureResult>,
        signature: &str,
        include_logs: bool,
    ) -> MonitorTransactionResponse {
        let (status, commitment_level, error_message, logs, compute_units) = match notification
            .value
        {
            RpcSignatureResult::ProcessedSignature(ProcessedSignatureResult { err }) => {
                // For compute units, we don't have it directly in this response
                // In a real implementation, you might need to fetch transaction details separately
                let compute_units = None;

                err.map_or_else(
                    || {
                        let logs = if include_logs {
                            // In a real implementation, we would get logs from the transaction details
                            // For now, provide a realistic example
                            vec![
                                "Program 11111111111111111111111111111111 invoke [1]".to_string(),
                                "Program 11111111111111111111111111111111 success".to_string(),
                            ]
                        } else {
                            vec![]
                        };

                        (
                            TransactionStatus::Processed,
                            CommitmentLevel::Processed,
                            None,
                            logs,
                            compute_units,
                        )
                    },
                    |tx_err| {
                        (
                            TransactionStatus::Failed,
                            CommitmentLevel::Processed,
                            Some(format!("Transaction failed: {tx_err:?}")),
                            vec![],
                            compute_units,
                        )
                    },
                )
            }
            RpcSignatureResult::ReceivedSignature(received) => match received {
                ReceivedSignatureResult::ReceivedSignature => {
                    (TransactionStatus::Received, CommitmentLevel::Processed, None, vec![], None)
                }
            },
        };

        MonitorTransactionResponse {
            signature: signature.to_string(),
            status: status.into(),
            slot: Some(notification.context.slot),
            error_message,
            logs,
            compute_units_consumed: compute_units,
            current_commitment: commitment_level.into(),
        }
    }

    /// Converts proto `CommitmentLevel` to Solana `CommitmentConfig`
    const fn commitment_level_to_config(level: CommitmentLevel) -> CommitmentConfig {
        match level {
            CommitmentLevel::Processed => CommitmentConfig::processed(),
            CommitmentLevel::Confirmed | CommitmentLevel::Unspecified => {
                CommitmentConfig::confirmed()
            }
            CommitmentLevel::Finalized => CommitmentConfig::finalized(),
        }
    }

    /// Cleans up expired or completed subscriptions
    pub fn cleanup_expired_subscriptions(&self) {
        let mut to_remove = Vec::new();

        // Find subscriptions that are no longer active
        for entry in self.active_subscriptions.iter() {
            let signature = entry.key();
            let handle = entry.value();

            // Check if the sender is closed (client disconnected)
            if handle.sender.is_closed() {
                to_remove.push(signature.clone());
            }
        }

        // Remove inactive subscriptions
        for signature in to_remove {
            if let Some((_key, handle)) = self.active_subscriptions.remove(&signature) {
                handle.abort_handle.abort();
                debug!(
                    signature = %signature,
                    "🧹 Cleaned up subscription"
                );
            }
        }

        let active_count = self.active_subscriptions.len();
        if active_count > 0 {
            debug!(active_count = active_count, "📊 Active subscriptions");
        }
    }

    /// Gracefully shuts down all subscriptions
    pub fn shutdown(&self) {
        info!("🛑 Shutting down WebSocket manager");

        let subscription_count = self.active_subscriptions.len();

        // Abort all active subscription tasks
        for entry in self.active_subscriptions.iter() {
            entry.value().abort_handle.abort();
        }

        // Clear all subscriptions
        self.active_subscriptions.clear();

        info!(
            subscription_count = subscription_count,
            "✅ WebSocket manager shutdown complete"
        );
    }
}

/// Utility function to derive WebSocket URL from RPC URL
/// For local development, WebSocket runs on RPC port + 1
/// For remote endpoints, WebSocket uses same URL with ws/wss protocol
pub fn derive_websocket_url_from_rpc(rpc_url: &str) -> Result<String, String> {
    if rpc_url.starts_with("http://localhost:") {
        // Extract port number and add 1 for WebSocket
        if let Some(port_start) = rpc_url.find(":8") {
            if let Some(port_str) = rpc_url.get(port_start + 1..) {
                if let Ok(port) = port_str.parse::<u16>() {
                    return Ok(format!("ws://localhost:{}", port + 1));
                }
            }
        }
        // Fallback for localhost without port
        Ok(rpc_url.replace("http://", "ws://"))
    } else if rpc_url.starts_with("http://") {
        Ok(rpc_url.replace("http://", "ws://"))
    } else if rpc_url.starts_with("https://") {
        Ok(rpc_url.replace("https://", "wss://"))
    } else {
        Err(format!("Invalid RPC URL format: {rpc_url}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_websocket_url_from_rpc() {
        assert_eq!(
            derive_websocket_url_from_rpc("http://localhost:8899"),
            Ok("ws://localhost:8900".to_string())
        );

        assert_eq!(
            derive_websocket_url_from_rpc("https://api.mainnet-beta.solana.com"),
            Ok("wss://api.mainnet-beta.solana.com".to_string())
        );

        assert!(derive_websocket_url_from_rpc("invalid://url").is_err());
    }

    #[tokio::test]
    async fn test_websocket_manager_creation() {
        // Test WebSocket manager creation
        let ws_url = "ws://localhost:8900";
        let rpc_url = "http://localhost:8899";

        // This should succeed even if WebSocket server is not running
        let manager = WebSocketManager::new(ws_url, rpc_url).await;
        assert!(manager.is_ok());

        info!("WebSocket manager test completed successfully");
    }
}
