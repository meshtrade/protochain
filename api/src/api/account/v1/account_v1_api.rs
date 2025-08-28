use std::sync::Arc;

use super::AccountServiceImpl;
use crate::service_providers::ServiceProviders;

pub struct AccountV1API {
    pub account_service: Arc<AccountServiceImpl>,
}

impl AccountV1API {
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        // Extract the specific dependency (RPC client) from service providers
        let rpc_client = service_providers.solana_clients.get_rpc_client();

        Self {
            account_service: Arc::new(AccountServiceImpl::new(rpc_client)),
        }
    }
}
