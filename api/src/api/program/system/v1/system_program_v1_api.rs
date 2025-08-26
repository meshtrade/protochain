use std::sync::Arc;

use super::SystemProgramServiceImpl;
use crate::service_providers::ServiceProviders;

pub struct SystemProgramV1API {
    pub system_program_service: Arc<SystemProgramServiceImpl>,
}

impl SystemProgramV1API {
    pub fn new(_service_providers: Arc<ServiceProviders>) -> Self {
        // No RPC client needed for instruction-based system program service
        
        SystemProgramV1API {
            system_program_service: Arc::new(SystemProgramServiceImpl::new()),
        }
    }
}