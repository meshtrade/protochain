use std::sync::Arc;
use anyhow::Result;

use super::solana_clients::SolanaClientsServiceProviders;
use crate::config::Config;

pub struct ServiceProviders {
    pub solana_clients: Arc<SolanaClientsServiceProviders>,
    config: Config, // Store config for network info and other services
}

impl ServiceProviders {
    pub fn new() -> Result<Self> {
        // Fallback constructor using environment variable
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        
        println!("🌐 Initializing Solana service providers with RPC URL: {}", rpc_url);
        
        let solana_clients = Arc::new(
            SolanaClientsServiceProviders::new(&rpc_url)?
        );
        
        // Create a minimal config for the fallback constructor
        let mut default_config = Config::default();
        default_config.solana.rpc_url = rpc_url;
        
        Ok(ServiceProviders {
            solana_clients,
            config: default_config,
        })
    }
    
    /// New constructor that uses the provided configuration
    pub fn new_with_config(config: Config) -> Result<Self> {
        println!("🌐 Initializing Solana service providers with configured RPC URL: {}", config.solana.rpc_url);
        
        let solana_clients = Arc::new(
            SolanaClientsServiceProviders::new(&config.solana.rpc_url)?
        );
        
        Ok(ServiceProviders {
            solana_clients,
            config,
        })
    }
    
    pub fn get_network_info(&self) -> String {
        self.config.solana.rpc_url.clone()
    }
}