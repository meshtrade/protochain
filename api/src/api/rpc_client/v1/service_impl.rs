use std::sync::Arc;
use tonic::{Request, Response, Status};

use protosol_api::protosol::solana::rpc_client::v1::{
    service_server::Service as RpcClientService, GetMinimumBalanceForRentExemptionRequest,
    GetMinimumBalanceForRentExemptionResponse,
};

use solana_client::rpc_client::RpcClient;

/// RPC Client service implementation for wrapping Solana RPC client methods
#[derive(Clone)]
pub struct RpcClientServiceImpl {
    /// Solana RPC client for blockchain interactions
    rpc_client: Arc<RpcClient>,
}

impl RpcClientServiceImpl {
    /// Creates a new `RpcClientServiceImpl` instance with the provided RPC client
    pub const fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }
}

#[tonic::async_trait]
impl RpcClientService for RpcClientServiceImpl {
    /// Gets the minimum balance required for rent exemption for a given data length
    async fn get_minimum_balance_for_rent_exemption(
        &self,
        request: Request<GetMinimumBalanceForRentExemptionRequest>,
    ) -> Result<Response<GetMinimumBalanceForRentExemptionResponse>, Status> {
        let req = request.into_inner();

        // Note: get_minimum_balance_for_rent_exemption doesn't support commitment levels in current Solana client
        // The commitment level parameter is accepted for API consistency but not used

        // Call the underlying Solana RPC client method
        match self.rpc_client.get_minimum_balance_for_rent_exemption(
            usize::try_from(req.data_length)
                .map_err(|e| Status::invalid_argument(format!("Invalid data length: {e}")))?,
        ) {
            Ok(balance) => {
                let response = GetMinimumBalanceForRentExemptionResponse { balance };
                Ok(Response::new(response))
            }
            Err(e) => Err(Status::internal(format!(
                "Failed to get minimum balance for rent exemption: {e}"
            ))),
        }
    }
}
