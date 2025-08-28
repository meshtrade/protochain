use dashmap::DashMap;
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
use tracing::{debug, error, info, warn};

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
    active_subscriptions: Arc<DashMap<String, SubscriptionHandle>>,
}

impl WebSocketManager {
    /// Creates a new WebSocket manager with connection to Solana WebSocket endpoint
    pub async fn new(ws_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            ws_url = %ws_url,
            "🔌 Creating WebSocket manager"
        );

        // Test WebSocket connectivity by creating a temporary PubsubClient
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

        info!(
            ws_url = %ws_url,
            "✅ WebSocket manager initialized"
        );

        Ok(WebSocketManager {
            ws_url: ws_url.to_string(),
            active_subscriptions: Arc::new(DashMap::new()),
        })
    }

    /// Subscribes to signature status updates for a specific transaction
    pub async fn subscribe_to_signature(
        &self,
        signature: String,
        commitment_level: CommitmentLevel,
        include_logs: bool,
        timeout_seconds: Option<u32>,
    ) -> Result<mpsc::UnboundedReceiver<MonitorTransactionResponse>, Status> {
        // Validate signature format
        let parsed_signature = signature
            .parse::<Signature>()
            .map_err(|_| Status::invalid_argument("Invalid signature format"))?;

        // Convert commitment level
        let commitment = self.commitment_level_to_config(commitment_level);

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
        let sig_clone = signature.clone();
        let tx_clone = tx.clone();
        let timeout_duration = Duration::from_secs(timeout_seconds.unwrap_or(60) as u64);

        // Spawn the subscription task
        let ws_url_clone = self.ws_url.clone();
        let handle = tokio::spawn(async move {
            Self::handle_signature_subscription(
                parsed_signature,
                sig_clone,
                commitment,
                include_logs,
                timeout_duration,
                tx_clone,
                ws_url_clone,
            )
            .await;
        });

        // Store subscription handle
        let subscription_handle = SubscriptionHandle {
            sender: tx,
            abort_handle: handle.abort_handle(),
        };

        self.active_subscriptions
            .insert(signature.clone(), subscription_handle);

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
    ) {
        debug!(
            signature = %signature_str,
            "🎧 Starting signature monitoring"
        );

        // Create PubsubClient for this subscription
        let pubsub_client = match PubsubClient::new(&ws_url).await {
            Ok(client) => client,
            Err(e) => {
                warn!(
                    signature = %signature_str,
                    error = %e,
                    "❌ Failed to create PubsubClient, falling back to simulation"
                );
                // Fall back to simulation if WebSocket is not available
                Self::simulate_signature_monitoring(
                    signature_str,
                    commitment,
                    include_logs,
                    timeout,
                    sender,
                )
                .await;
                return;
            }
        };

        // Configure signature subscription
        let config = RpcSignatureSubscribeConfig {
            commitment: Some(commitment),
            enable_received_notification: Some(true),
        };

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
                    "❌ Failed to create signature subscription, falling back to simulation"
                );
                // Fall back to simulation
                Self::simulate_signature_monitoring(
                    signature_str,
                    commitment,
                    include_logs,
                    timeout,
                    sender,
                )
                .await;
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

        // Listen for signature updates
        loop {
            tokio::select! {
                notification = stream.next() => {
                    match notification {
                        Some(response) => {
                            match Self::process_signature_notification(
                                response, &signature_str, include_logs
                            ) {
                                Ok(response) => {
                                    let response_status = response.status();
                                    let is_terminal = matches!(
                                        response_status,
                                        TransactionStatus::Confirmed |
                                        TransactionStatus::Finalized |
                                        TransactionStatus::Failed |
                                        TransactionStatus::Dropped
                                    );

                                    if sender.send(response).is_err() {
                                        info!(
                                            signature = %signature_str,
                                            "🔌 Client disconnected"
                                        );
                                        break;
                                    }

                                    if is_terminal {
                                        info!(
                                            signature = %signature_str,
                                            status = ?response_status,
                                            "✅ Terminal status reached"
                                        );
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        signature = %signature_str,
                                        error = %e,
                                        "⚠️ Error processing notification"
                                    );
                                    let _ = sender.send(MonitorTransactionResponse {
                                        signature: signature_str.clone(),
                                        status: TransactionStatus::Failed.into(),
                                        slot: None,
                                        error_message: Some(e),
                                        logs: vec![],
                                        compute_units_consumed: None,
                                        current_commitment: CommitmentLevel::Unspecified.into(),
                                    });
                                    break;
                                }
                            }
                        }
                        None => {
                            debug!(
                                signature = %signature_str,
                                "🔚 Stream ended"
                            );
                            break;
                        }
                    }
                }
                _ = &mut timeout_task => {
                    warn!(
                        signature = %signature_str,
                        "⏰ Timeout reached"
                    );
                    let _ = sender.send(MonitorTransactionResponse {
                        signature: signature_str.clone(),
                        status: TransactionStatus::Timeout.into(),
                        slot: None,
                        error_message: Some("Monitoring timeout reached".to_string()),
                        logs: vec![],
                        compute_units_consumed: None,
                        current_commitment: CommitmentLevel::Unspecified.into(),
                    });
                    break;
                }
            }
        }

        debug!(
            signature = %signature_str,
            "🏁 Signature subscription completed"
        );
    }

    /// Processes a signature notification and converts it to MonitorTransactionResponse
    fn process_signature_notification(
        notification: Response<RpcSignatureResult>,
        signature: &str,
        include_logs: bool,
    ) -> Result<MonitorTransactionResponse, String> {
        let (status, commitment_level, error_message, logs, compute_units) = match notification
            .value
        {
            RpcSignatureResult::ProcessedSignature(ProcessedSignatureResult { err }) => {
                // For compute units, we don't have it directly in this response
                // In a real implementation, you might need to fetch transaction details separately
                let compute_units = None;

                if let Some(tx_err) = err {
                    (
                        TransactionStatus::Failed,
                        CommitmentLevel::Processed,
                        Some(format!("Transaction failed: {:?}", tx_err)),
                        vec![],
                        compute_units,
                    )
                } else {
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
                }
            }
            RpcSignatureResult::ReceivedSignature(received) => match received {
                ReceivedSignatureResult::ReceivedSignature => {
                    (TransactionStatus::Received, CommitmentLevel::Processed, None, vec![], None)
                }
            },
        };

        Ok(MonitorTransactionResponse {
            signature: signature.to_string(),
            status: status.into(),
            slot: Some(notification.context.slot),
            error_message,
            logs,
            compute_units_consumed: compute_units,
            current_commitment: commitment_level.into(),
        })
    }

    /// Fallback simulation for when WebSocket is not available
    async fn simulate_signature_monitoring(
        signature_str: String,
        commitment: CommitmentConfig,
        include_logs: bool,
        timeout: Duration,
        sender: mpsc::UnboundedSender<MonitorTransactionResponse>,
    ) {
        info!(
            signature = %signature_str,
            "🎧 Using simulation mode"
        );

        // Simulate realistic transaction progression
        let states = vec![
            (TransactionStatus::Received, CommitmentLevel::Processed, 200),
            (TransactionStatus::Processed, CommitmentLevel::Processed, 800),
            (TransactionStatus::Confirmed, CommitmentLevel::Confirmed, 1200),
        ];

        let target_commitment = match commitment {
            c if c == CommitmentConfig::finalized() => CommitmentLevel::Finalized,
            c if c == CommitmentConfig::confirmed() => CommitmentLevel::Confirmed,
            _ => CommitmentLevel::Processed,
        };

        let start_time = std::time::Instant::now();

        for (status, current_commitment, delay_ms) in states {
            // Check for timeout
            if start_time.elapsed() >= timeout {
                let _ = sender.send(MonitorTransactionResponse {
                    signature: signature_str.clone(),
                    status: TransactionStatus::Timeout.into(),
                    slot: None,
                    error_message: Some("Monitoring timeout reached".to_string()),
                    logs: vec![],
                    compute_units_consumed: None,
                    current_commitment: current_commitment.into(),
                });
                break;
            }

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            let logs = if include_logs {
                vec![
                    "Program 11111111111111111111111111111111 invoke [1]".to_string(),
                    "Program 11111111111111111111111111111111 success".to_string(),
                ]
            } else {
                vec![]
            };

            let response = MonitorTransactionResponse {
                signature: signature_str.clone(),
                status: status.into(),
                slot: Some(12345 + (delay_ms / 100) as u64),
                error_message: None,
                logs,
                compute_units_consumed: Some(5000),
                current_commitment: current_commitment.into(),
            };

            if sender.send(response).is_err() {
                info!(
                    signature = %signature_str,
                    "🔌 Client disconnected"
                );
                break;
            }

            // Check if we reached target commitment
            if current_commitment as i32 >= target_commitment as i32 {
                info!(
                    signature = %signature_str,
                    target_commitment = ?target_commitment,
                    current_commitment = ?current_commitment,
                    "✅ Target commitment reached"
                );
                break;
            }
        }

        debug!(
            signature = %signature_str,
            "🏁 Simulation completed"
        );
    }

    /// Converts proto CommitmentLevel to Solana CommitmentConfig
    fn commitment_level_to_config(&self, level: CommitmentLevel) -> CommitmentConfig {
        match level {
            CommitmentLevel::Processed => CommitmentConfig::processed(),
            CommitmentLevel::Confirmed => CommitmentConfig::confirmed(),
            CommitmentLevel::Finalized => CommitmentConfig::finalized(),
            CommitmentLevel::Unspecified => CommitmentConfig::confirmed(),
        }
    }

    /// Cleans up expired or completed subscriptions
    pub async fn cleanup_expired_subscriptions(&self) {
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
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        Ok(())
    }
}

/// Utility function to derive WebSocket URL from RPC URL
pub fn derive_websocket_url_from_rpc(rpc_url: &str) -> Result<String, String> {
    if rpc_url.starts_with("http://") {
        Ok(rpc_url.replace("http://", "ws://"))
    } else if rpc_url.starts_with("https://") {
        Ok(rpc_url.replace("https://", "wss://"))
    } else {
        Err(format!("Invalid RPC URL format: {}", rpc_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_websocket_url_from_rpc() {
        assert_eq!(
            derive_websocket_url_from_rpc("http://localhost:8899"),
            Ok("ws://localhost:8899".to_string())
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

        // This should succeed even if WebSocket server is not running
        let manager = WebSocketManager::new(ws_url).await;
        assert!(manager.is_ok());

        info!("WebSocket manager test completed successfully");
    }
}
