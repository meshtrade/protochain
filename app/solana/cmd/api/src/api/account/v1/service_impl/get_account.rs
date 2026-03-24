//! Handler implementation for `GetAccount`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{
    Account, GetAccountRequest, GetAccountResponse,
};

use solana_sdk::pubkey::Pubkey;

use super::commitment_level_to_config;
use super::AccountServiceImpl;

#[allow(clippy::result_large_err, clippy::unused_self)]
impl AccountServiceImpl {
    /// Fetches account data from the Solana blockchain with configurable commitment level.
    pub(super) fn handle_get_account(
        &self,
        request: Request<GetAccountRequest>,
    ) -> Result<Response<GetAccountResponse>, Status> {
        println!("Received get account request: {request:?}");

        let req = request.into_inner();

        // Validate the address format
        if req.address.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }

        // Parse the address
        let pubkey = Pubkey::from_str(&req.address)
            .map_err(|e| Status::invalid_argument(format!("Invalid address format: {e}")))?;

        // Log account fetch attempt for debugging
        println!("Attempting to fetch account: {pubkey} via RPC client");

        // CRITICAL: Use get_account_with_commitment instead of get_account for timing reliability
        //
        // Reasoning for this design choice:
        // 1. TIMING ISSUES: After FundNative creates an account via request_airdrop(), there's a
        //    timing window where the account exists on-chain but isn't visible via get_account()
        //    due to different commitment levels between airdrop confirmation and account queries.
        //
        // 2. COMMITMENT CONSISTENCY: request_airdrop() + confirm_transaction() uses 'confirmed'
        //    commitment internally, so we need get_account_with_commitment(confirmed) to see
        //    the same blockchain state.
        //
        // 3. LOCAL VALIDATOR BEHAVIOR: Local test validators can have different timing characteristics
        //    than mainnet. The confirmed commitment level provides consistent behavior across
        //    different network conditions.
        //
        // 4. ATOMIC TRANSACTION SUPPORT: Multi-instruction transactions that create and immediately
        //    use accounts require consistent commitment levels across all RPC operations.
        //
        // Alternative approaches considered:
        // - get_account(): Fast but unreliable due to commitment timing mismatches
        // - get_account_with_commitment(processed): Faster but still timing issues
        // - get_account_with_commitment(configurable): Now configurable via request parameter
        let commitment = commitment_level_to_config(req.commitment_level);

        // Fetch account from Solana network using our dependency-injected RPC client
        match self
            .rpc_client
            .get_account_with_commitment(&pubkey, commitment)
        {
            Ok(response) => {
                if let Some(account) = response.value {
                    println!("RPC get_account_with_commitment succeeded for: {pubkey}");
                    println!("Account balance: {} lamports", account.lamports);
                    // Convert Solana account to our Account type
                    let account_proto = Account {
                        address: req.address.clone(),
                        lamports: account.lamports,
                        owner: account.owner.to_string(),
                        executable: account.executable,
                        data: serde_json::to_string(&account.data)
                            .unwrap_or_else(|_| "Failed to serialize account data".to_string()),
                        rent_epoch: account.rent_epoch,
                    };

                    println!("Successfully fetched account: {}", req.address);
                    Ok(Response::new(GetAccountResponse {
                        account: Some(account_proto),
                    }))
                } else {
                    println!("get_account_with_commitment returned None for: {pubkey}");
                    Err(Status::not_found(format!("Account not found: {}", req.address)))
                }
            }
            Err(e) => {
                eprintln!("Error fetching account {}: {}", req.address, e);
                // Check if it's a not found error
                if e.to_string().contains("not found") || e.to_string().contains("AccountNotFound")
                {
                    Err(Status::not_found(format!("Account not found: {}", req.address)))
                } else {
                    Err(Status::internal(format!("Failed to fetch account: {e}")))
                }
            }
        }
    }
}
