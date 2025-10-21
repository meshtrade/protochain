use anyhow::Result;
use std::sync::Arc;

use super::solana_clients::SolanaClientsServiceProviders;
use crate::config::Config;
use crate::websocket::{derive_websocket_url_from_rpc, WebSocketManager};

/// Main service provider container that manages all service dependencies
pub struct ServiceProviders {
    /// Solana RPC client providers
    pub solana_clients: Arc<SolanaClientsServiceProviders>,
    /// WebSocket manager for real-time monitoring
    pub websocket_manager: Arc<WebSocketManager>,
    config: Config, // Store config for network info and other services
}

impl ServiceProviders {
    /// New constructor that uses the provided configuration
    pub async fn new_with_config(config: Config) -> Result<Self> {
        println!(
            "🌐 Initializing Solana service providers with configured RPC URL: {}",
            config.solana.rpc_url
        );

        let solana_clients = Arc::new(SolanaClientsServiceProviders::new(&config.solana.rpc_url));

        // Use configured WebSocket URL if available, otherwise derive from RPC URL
        let ws_url = if let Some(ws_url) = config.solana.websocket_url.as_ref() {
            ws_url.clone()
        } else {
            derive_websocket_url_from_rpc(&config.solana.rpc_url)
                .map_err(|e| anyhow::anyhow!("Failed to derive WebSocket URL: {}", e))?
        };

        // Create WebSocket manager with simulation mode
        println!("🔌 Initializing WebSocket manager...");

        // The WebSocket manager provides realistic transaction monitoring simulation
        let websocket_manager = Arc::new(
            WebSocketManager::new(&ws_url, &config.solana.rpc_url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create WebSocket manager: {}", e))?,
        );

        Ok(Self {
            solana_clients,
            websocket_manager,
            config,
        })
    }

    /// Returns network information string for logging/debugging
    pub fn get_network_info(&self) -> String {
        self.config.solana.rpc_url.clone()
    }
}
