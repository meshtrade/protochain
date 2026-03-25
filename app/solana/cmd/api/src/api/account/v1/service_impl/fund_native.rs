//! Handler implementation for `FundNative`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{FundNativeRequest, FundNativeResponse};

use solana_sdk::pubkey::Pubkey;

use crate::api::common::transaction_monitoring::wait_for_transaction_success_by_string;

use super::commitment_level_to_config;
use super::AccountServiceImpl;

#[allow(clippy::result_large_err, clippy::unused_self)]
impl AccountServiceImpl {
    /// Requests an airdrop of SOL to the given address (devnet/testnet only).
    pub(super) async fn handle_fund_native(
        &self,
        request: Request<FundNativeRequest>,
    ) -> Result<Response<FundNativeResponse>, Status> {
        // Validate minimum funding amount to prevent common failures
        const MIN_FUNDING_AMOUNT: u64 = 1_000_000_000; // 1 SOL for rent exemption

        println!("Received fund native request: {request:?}");

        let req = request.into_inner();

        // Basic input validation
        if req.address.is_empty() {
            return Err(Status::invalid_argument("Address is required"));
        }

        if req.amount.is_empty() {
            return Err(Status::invalid_argument("Amount is required"));
        }

        // Parse and validate address
        let address = Pubkey::from_str(&req.address)
            .map_err(|e| Status::invalid_argument(format!("Invalid address: {e}")))?;

        // Parse and validate amount
        let amount = req
            .amount
            .parse::<u64>()
            .map_err(|e| Status::invalid_argument(format!("Invalid amount: {e}")))?;

        if amount == 0 {
            return Err(Status::invalid_argument("Amount must be greater than 0"));
        }

        if amount < MIN_FUNDING_AMOUNT {
            return Err(Status::invalid_argument(
                format!(
                    "Funding amount too small. Minimum: {MIN_FUNDING_AMOUNT} lamports (1 SOL) required for rent exemption. Provided: {amount} lamports"
                )
            ));
        }

        // Request airdrop
        println!("Requesting airdrop of {amount} lamports to {address}");
        // RPC client ready for airdrop request
        let signature = self
            .rpc_client
            .request_airdrop(&address, amount)
            .await
            .map_err(|e| Status::internal(format!("Airdrop request failed: {e}")))?;

        // Wait for transaction success validation (not just confirmation)
        println!("Waiting for airdrop success validation: {signature}");
        let commitment = commitment_level_to_config(req.commitment_level);
        wait_for_transaction_success_by_string(
            self.rpc_client.clone(),
            &signature.to_string(),
            commitment,
            Some(60),
        )
        .await?;

        println!("Airdrop completed successfully: {signature}");

        Ok(Response::new(FundNativeResponse {
            signature: signature.to_string(),
        }))
    }
}
