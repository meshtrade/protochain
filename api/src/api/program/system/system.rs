use std::sync::Arc;

use super::v1::SystemProgramV1API;
use crate::service_providers::ServiceProviders;

pub struct System {
    pub v1: Arc<SystemProgramV1API>,
}

impl System {
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        System {
            v1: Arc::new(SystemProgramV1API::new(service_providers)),
        }
    }
}
