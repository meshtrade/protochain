use std::sync::Arc;

use super::system::System;
use crate::service_providers::ServiceProviders;

/// Program services aggregator that provides access to all Solana program interfaces
pub struct Program {
    /// System program service interface
    pub system: Arc<System>,
}

impl Program {
    /// Creates a new Program instance with the provided service providers
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            system: Arc::new(System::new(service_providers)),
        }
    }
}
