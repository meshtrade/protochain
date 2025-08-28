use std::sync::Arc;

use super::TransactionServiceImpl;
use crate::service_providers::ServiceProviders;

/// gRPC service wrapper for transaction lifecycle operations
pub struct TransactionV1API {
    /// Core transaction service implementation
    pub transaction_service: Arc<TransactionServiceImpl>,
}

impl TransactionV1API {
    /// Creates a new `TransactionV1API` instance with the provided service providers
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        // Extract the specific dependencies (RPC client and WebSocket manager) from service providers
        let rpc_client = service_providers.solana_clients.get_rpc_client();
        let websocket_manager = service_providers.websocket_manager.clone();

        Self {
            transaction_service: Arc::new(TransactionServiceImpl::new(
                rpc_client,
                websocket_manager,
            )),
        }
    }
}
