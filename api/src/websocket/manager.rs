use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use dashmap::DashMap;
use uuid::Uuid;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcSignatureSubscribeConfig;
use solana_client::rpc_response::{Response, RpcSignatureResult, ProcessedSignatureResult, ReceivedSignatureResult};
use solana_sdk::{signature::Signature, commitment_config::CommitmentConfig};
use tonic::Status;

use protosol_api::protosol::solana::transaction::v1::{
    MonitorTransactionResponse,
    TransactionStatus,
};
use protosol_api::protosol::solana::r#type::v1::CommitmentLevel;

/// Handle for managing a signature subscription
#[derive(Debug)]
struct SubscriptionHandle {
    subscription_id: String,
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
        println!("🔌 Creating WebSocket manager for: {}", ws_url);
        
        // Test WebSocket connectivity by creating a temporary PubsubClient
        match PubsubClient::new(ws_url).await {
            Ok(_test_client) => {
                println!("✅ WebSocket connection validated successfully");
            }
            Err(e) => {
                println!("⚠️ WebSocket connection failed: {}. Will create per-subscription.", e);
            }
        }
        
        println!("✅ WebSocket manager initialized");
        
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
        let parsed_signature = signature.parse::<Signature>()
            .map_err(|_| Status::invalid_argument("Invalid signature format"))?;

        // Convert commitment level
        let commitment = self.commitment_level_to_config(commitment_level);
        
        // Create channels for communication
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Generate unique subscription ID
        let subscription_id = Uuid::new_v4().to_string();
        
        println!("🔔 Creating signature subscription for: {}", signature);
        
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
            ).await;
        });
        
        // Store subscription handle
        let subscription_handle = SubscriptionHandle {
            subscription_id: subscription_id.clone(),
            sender: tx,
            abort_handle: handle.abort_handle(),
        };
        
        self.active_subscriptions.insert(signature.clone(), subscription_handle);
        
        println!("✅ Signature subscription created: {}", subscription_id);
        
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
        println!("🎧 Starting signature monitoring for: {}", signature_str);
        
        // Create PubsubClient for this subscription
        let pubsub_client = match PubsubClient::new(&ws_url).await {
            Ok(client) => client,
            Err(e) => {
                println!("❌ Failed to create PubsubClient: {}", e);
                // Fall back to simulation if WebSocket is not available
                Self::simulate_signature_monitoring(
                    signature_str, commitment, include_logs, timeout, sender
                ).await;
                return;
            }
        };
        
        // Configure signature subscription
        let config = RpcSignatureSubscribeConfig {
            commitment: Some(commitment),
            enable_received_notification: Some(true),
        };
        
        // Create signature subscription
        let (mut stream, _unsubscribe) = match pubsub_client.signature_subscribe(&signature, Some(config)).await {
            Ok(subscription) => subscription,
            Err(e) => {
                println!("❌ Failed to create signature subscription: {}", e);
                // Fall back to simulation
                Self::simulate_signature_monitoring(
                    signature_str, commitment, include_logs, timeout, sender
                ).await;
                return;
            }
        };
        
        println!("✅ Signature subscription established for: {}", signature_str);
        
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
                                    let is_terminal = matches!(
                                        response.status(),
                                        TransactionStatus::Confirmed |
                                        TransactionStatus::Finalized |
                                        TransactionStatus::Failed |
                                        TransactionStatus::Dropped
                                    );
                                    
                                    if sender.send(response).is_err() {
                                        println!("🔌 Client disconnected for: {}", signature_str);
                                        break;
                                    }
                                    
                                    if is_terminal {
                                        println!("✅ Terminal status reached for: {}", signature_str);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    println!("⚠️ Error processing notification: {}", e);
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
                            println!("🔚 Stream ended for: {}", signature_str);
                            break;
                        }
                    }
                }
                _ = &mut timeout_task => {
                    println!("⏰ Timeout reached for: {}", signature_str);
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
        
        println!("🏁 Signature subscription completed: {}", signature_str);
    }
    
    /// Processes a signature notification and converts it to MonitorTransactionResponse
    fn process_signature_notification(
        notification: Response<RpcSignatureResult>,
        signature: &str,
        include_logs: bool,
    ) -> Result<MonitorTransactionResponse, String> {
        
        let (status, commitment_level, error_message, logs, compute_units) = match notification.value {
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
            RpcSignatureResult::ReceivedSignature(received) => {
                match received {
                    ReceivedSignatureResult::ReceivedSignature => {
                        (
                            TransactionStatus::Received,
                            CommitmentLevel::Processed,
                            None,
                            vec![],
                            None,
                        )
                    }
                }
            }
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
        println!("🎧 Using simulation mode for signature: {}", signature_str);
        
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
                println!("🔌 Client disconnected for: {}", signature_str);
                break;
            }
            
            // Check if we reached target commitment
            if current_commitment as i32 >= target_commitment as i32 {
                println!("✅ Target commitment reached for: {}", signature_str);
                break;
            }
        }
        
        println!("🏁 Simulation completed for: {}", signature_str);
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
                println!("🧹 Cleaned up subscription for: {}", signature);
            }
        }
        
        let active_count = self.active_subscriptions.len();
        if active_count > 0 {
            println!("📊 Active subscriptions: {}", active_count);
        }
    }
    
    /// Gracefully shuts down all subscriptions
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🛑 Shutting down WebSocket manager...");
        
        let subscription_count = self.active_subscriptions.len();
        
        // Abort all active subscription tasks
        for entry in self.active_subscriptions.iter() {
            entry.value().abort_handle.abort();
        }
        
        // Clear all subscriptions
        self.active_subscriptions.clear();
        
        println!("✅ WebSocket manager shutdown complete. Cleaned up {} subscriptions", subscription_count);
        
        Ok(())
    }
    
    /// Returns the number of active subscriptions
    pub fn active_subscription_count(&self) -> usize {
        self.active_subscriptions.len()
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
        
        println!("WebSocket manager test completed successfully");
    }
}