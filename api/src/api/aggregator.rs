use std::sync::Arc;

use super::account::v1::AccountV1API;
use super::program::Program;
use super::transaction::v1::TransactionV1API;
use crate::service_providers::ServiceProviders;

/// Main API aggregator that combines all service implementations
pub struct Api {
    /// Account management API v1
    pub account_v1: Arc<AccountV1API>,
    /// Transaction lifecycle API v1
    pub transaction_v1: Arc<TransactionV1API>,
    /// Program services (system, etc.)
    pub program: Arc<Program>,
}

impl Api {
    /// Creates a new API instance with the provided service providers
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            account_v1: Arc::new(AccountV1API::new(&service_providers)),
            transaction_v1: Arc::new(TransactionV1API::new(&service_providers)),
            program: Arc::new(Program::new(service_providers)),
        }
    }
}
