use std::sync::Arc;

use super::account::v1::AccountV1API;
use super::program::Program;
use super::transaction::v1::TransactionV1API;
use crate::service_providers::ServiceProviders;

pub struct API {
    pub account_v1: Arc<AccountV1API>,
    pub transaction_v1: Arc<TransactionV1API>,
    pub program: Arc<Program>,
}

impl API {
    pub fn new(service_providers: Arc<ServiceProviders>) -> Self {
        Self {
            account_v1: Arc::new(AccountV1API::new(&service_providers)),
            transaction_v1: Arc::new(TransactionV1API::new(&service_providers)),
            program: Arc::new(Program::new(service_providers)),
        }
    }
}
