use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{
    service_server::Service as AccountService, Account, FundNativeRequest, FundNativeResponse,
    GenerateNewKeyPairRequest, GenerateNewKeyPairResponse, GetAccountRequest, GetAccountResponse,
};
use protochain_api::protochain::solana::r#type::v1::{CommitmentLevel, KeyPair};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, SeedDerivable, Signer},
};

use crate::api::common::transaction_monitoring::wait_for_transaction_success_by_string;

#[derive(Clone)]
/// Core business logic implementation for account management operations
pub struct AccountServiceImpl {
    /// Solana RPC client for blockchain interactions
    rpc_client: Arc<RpcClient>,
}

impl AccountServiceImpl {
    /// Creates a new `AccountServiceImpl` instance with the provided RPC client
    pub const fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }
}

/// Helper function to convert proto `CommitmentLevel` to Solana `CommitmentConfig`
/// Provides sensible defaults when commitment level is not specified
fn commitment_level_to_config(commitment_level: i32) -> CommitmentConfig {
    match CommitmentLevel::try_from(commitment_level) {
        Ok(CommitmentLevel::Processed) => CommitmentConfig::processed(),
        Ok(CommitmentLevel::Confirmed) => CommitmentConfig::confirmed(),
        Ok(CommitmentLevel::Finalized) => CommitmentConfig::finalized(),
        Ok(CommitmentLevel::Unspecified) | Err(_) => {
            // Default to confirmed for reliability - matches our previous fix
            CommitmentConfig::confirmed()
        }
    }
}

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
    async fn get_account(
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
        println!("🔍 Attempting to fetch account: {pubkey} via RPC client");

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
                    println!("✅ RPC get_account_with_commitment succeeded for: {pubkey}");
                    println!("💰 Account balance: {} lamports", account.lamports);
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
                    println!("⚠️ get_account_with_commitment returned None for: {pubkey}");
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

    async fn generate_new_key_pair(
        &self,
        request: Request<GenerateNewKeyPairRequest>,
    ) -> Result<Response<GenerateNewKeyPairResponse>, Status> {
        println!("Received generate keypair request: {request:?}");

        let req = request.into_inner();

        // Generate keypair (random or from seed)
        let keypair = if req.seed.is_empty() {
            // Random generation
            Keypair::new()
        } else {
            // Deterministic generation from seed
            let seed_bytes = hex::decode(&req.seed)
                .map_err(|e| Status::invalid_argument(format!("Invalid hex seed: {e}")))?;

            if seed_bytes.len() != 32 {
                return Err(Status::invalid_argument("Seed must be exactly 32 bytes"));
            }

            let mut seed_array = [0u8; 32];
            seed_array.copy_from_slice(&seed_bytes);
            Keypair::from_seed(&seed_array).map_err(|e| {
                Status::internal(format!("Failed to generate keypair from seed: {e}"))
            })?
        };

        // Create protobuf KeyPair with proper field names
        let key_pair = KeyPair {
            public_key: keypair.pubkey().to_string(), // Base58 encoded
            private_key: bs58::encode(keypair.to_bytes()).into_string(), // Base58 encoded full keypair
        };

        println!("Generated keypair with public key: {}", key_pair.public_key);

        Ok(Response::new(GenerateNewKeyPairResponse {
            key_pair: Some(key_pair),
        }))
    }

    async fn fund_native(
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
