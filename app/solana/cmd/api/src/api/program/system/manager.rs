use std::sync::Arc;

use super::v1::SystemProgramV1API;
use crate::service_providers::ServiceProviders;

/// System program service aggregator providing versioned API access
pub struct System {
    /// Version 1 of the System Program API
    pub v1: Arc<SystemProgramV1API>,
}

impl System {
    /// Creates a new System instance with the provided service providers
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            v1: Arc::new(SystemProgramV1API::new(service_providers)),
        }
    }
}
