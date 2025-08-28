use solana_client::rpc_client::RpcClient;
use std::sync::Arc;

/// Service provider container for Solana client instances
pub struct SolanaClientsServiceProviders {
    /// Shared RPC client for Solana blockchain interactions
    pub rpc_client: Arc<RpcClient>,
}

impl SolanaClientsServiceProviders {
    /// Creates a new `SolanaClientsServiceProviders` instance with the specified RPC URL
    pub fn new(rpc_url: &str) -> Self {
        println!("🔗 Initializing Solana RPC client with URL: {rpc_url}");

        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));

        Self { rpc_client }
    }

    /// Returns a cloned reference to the shared RPC client
    pub fn get_rpc_client(&self) -> Arc<RpcClient> {
        Arc::clone(&self.rpc_client)
    }
}
