use std::sync::Arc;

use super::TransactionServiceImpl;
use crate::service_providers::ServiceProviders;

pub struct TransactionV1API {
    pub transaction_service: Arc<TransactionServiceImpl>,
}

impl TransactionV1API {
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        // Extract the specific dependency (RPC client) from service providers
        let rpc_client = service_providers.solana_clients.get_rpc_client();
        
        TransactionV1API {
            transaction_service: Arc::new(TransactionServiceImpl::new(rpc_client)),
        }
    }
}