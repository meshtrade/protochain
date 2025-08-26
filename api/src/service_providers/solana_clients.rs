use std::sync::Arc;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;

pub struct SolanaClientsServiceProviders {
    pub rpc_client: Arc<RpcClient>,
}

impl SolanaClientsServiceProviders {
    pub fn new(rpc_url: &str) -> Result<Self> {
        println!("🔗 Initializing Solana RPC client with URL: {}", rpc_url);
        
        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));
        
        Ok(SolanaClientsServiceProviders {
            rpc_client,
        })
    }
    
    pub fn get_rpc_client(&self) -> Arc<RpcClient> {
        Arc::clone(&self.rpc_client)
    }
}