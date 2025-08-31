use std::sync::Arc;

use super::service_impl::TokenProgramServiceImpl;
use crate::service_providers::ServiceProviders;

/// Token Program API v1 wrapper
pub struct TokenV1API {
    /// The Token Program service implementation
    pub token_program_service: Arc<TokenProgramServiceImpl>,
}

impl TokenV1API {
    /// Creates a new Token V1 API instance
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        Self {
            token_program_service: Arc::new(TokenProgramServiceImpl::new(Arc::clone(
                &service_providers.solana_clients.rpc_client,
            ))),
        }
    }
}
