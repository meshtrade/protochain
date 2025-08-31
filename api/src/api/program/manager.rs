use std::sync::Arc;

use super::system::System;
use super::token::TokenV1API;
use crate::service_providers::ServiceProviders;

/// Program services aggregator that provides access to all Solana program interfaces
pub struct Program {
    /// System program service interface
    pub system: Arc<System>,
    /// Token program service interface
    pub token: Arc<TokenV1API>,
}

impl Program {
    /// Creates a new Program instance with the provided service providers
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            system: Arc::new(System::new(Arc::clone(&service_providers))),
            token: Arc::new(TokenV1API::new(&service_providers)),
        }
    }
}
