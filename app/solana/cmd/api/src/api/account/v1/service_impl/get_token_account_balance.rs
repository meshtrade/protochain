//! Handler implementation for `GetTokenAccountBalance`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{
    GetTokenAccountBalanceRequest, GetTokenAccountBalanceResponse,
};

use solana_sdk::pubkey::Pubkey;

use super::AccountServiceImpl;

#[allow(clippy::result_large_err)]
impl AccountServiceImpl {
    /// Retrieves the token balance for a given SPL token account.
    pub(super) async fn handle_get_token_account_balance(
        &self,
        request: Request<GetTokenAccountBalanceRequest>,
    ) -> Result<Response<GetTokenAccountBalanceResponse>, Status> {
        let req = request.into_inner();

        // parse the address
        let pubkey = Pubkey::from_str(&req.address)
            .map_err(|e| Status::invalid_argument(format!("Invalid address format: {e}")))?;

        // get the balance on the given token account
        let balance = self
            .rpc_client
            .get_token_account_balance(&pubkey)
            .await
            .map_err(|e| {
                Status::internal(format!("Could not retrieve token account balance: {e}"))
            })?;

        Ok(Response::new(GetTokenAccountBalanceResponse {
            amount: balance.amount,
            decimals: balance.decimals.into(), // just using unwrap here since we are casting u8 to u32 so panic should not happen
        }))
    }
}
