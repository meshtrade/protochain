use std::sync::Arc;

use super::service_impl::RpcClientServiceImpl;
use crate::service_providers::ServiceProviders;

/// RPC Client API v1 wrapper
pub struct RpcClientV1API {
    /// The RPC Client service implementation
    pub rpc_client_service: Arc<RpcClientServiceImpl>,
}

impl RpcClientV1API {
    /// Creates a new RPC Client V1 API instance
    pub fn new(service_providers: &Arc<ServiceProviders>) -> Self {
        Self {
            rpc_client_service: Arc::new(RpcClientServiceImpl::new(Arc::clone(
                &service_providers.solana_clients.rpc_client,
            ))),
        }
    }
}
