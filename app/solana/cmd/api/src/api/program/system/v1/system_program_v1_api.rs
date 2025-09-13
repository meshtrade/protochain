use std::sync::Arc;

use super::SystemProgramServiceImpl;
use crate::service_providers::ServiceProviders;

/// gRPC service wrapper for System Program v1 operations
pub struct SystemProgramV1API {
    /// Core System Program service implementation
    pub system_program_service: Arc<SystemProgramServiceImpl>,
}

impl SystemProgramV1API {
    /// Creates a new `SystemProgramV1API` instance with the provided service providers
    pub fn new(_service_providers: Arc<ServiceProviders>) -> Self {
        // No RPC client needed for instruction-based system program service

        Self {
            system_program_service: Arc::new(SystemProgramServiceImpl::new()),
        }
    }
}
