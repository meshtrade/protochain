use std::sync::Arc;

use super::AccountServiceImpl;
use crate::service_providers::ServiceProviders;

/// gRPC service wrapper for account management operations
pub struct AccountV1API {
    /// Core account service implementation
    pub account_service: Arc<AccountServiceImpl>,
}

impl AccountV1API {
    /// Creates a new `AccountV1API` instance with the provided service providers
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        // Extract the specific dependency (RPC client) from service providers
        let rpc_client = service_providers.solana_clients.get_rpc_client();

        Self {
            account_service: Arc::new(AccountServiceImpl::new(rpc_client)),
        }
    }
}
