use std::sync::Arc;

use super::system::System;
use crate::service_providers::ServiceProviders;

pub struct Program {
    pub system: Arc<System>,
}

impl Program {
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Program {
            system: Arc::new(System::new(service_providers)),
        }
    }
}