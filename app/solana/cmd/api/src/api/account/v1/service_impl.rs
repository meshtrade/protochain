//! Account gRPC service implementation.
//!
//! The `AccountService` trait impl lives here, with each method delegating
//! to a handler defined in a sub-module under `service_impl/`.
//!
//! Sub-modules:
//! - [`fund_native`]                   -- `FundNative`
//! - [`generate_new_key_pair`]         -- `GenerateNewKeyPair`
//! - [`get_account`]                   -- `GetAccount`
//! - [`get_associated_token_address`]  -- `GetAssociatedTokenAddress`
//! - [`get_token_account_balance`]     -- `GetTokenAccountBalance`

mod fund_native;
mod generate_new_key_pair;
mod get_account;
mod get_associated_token_address;
mod get_token_account_balance;

use std::sync::Arc;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::account::v1::{
    service_server::Service as AccountService, FundNativeRequest, FundNativeResponse,
    GenerateNewKeyPairRequest, GenerateNewKeyPairResponse, GetAccountRequest, GetAccountResponse,
    GetAssociatedTokenAddressRequest, GetAssociatedTokenAddressResponse,
    GetTokenAccountBalanceRequest, GetTokenAccountBalanceResponse,
};
use protochain_api::protochain::solana::r#type::v1::CommitmentLevel;

use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;

// ---------------------------------------------------------------------------
//  Struct definition
// ---------------------------------------------------------------------------

/// Core business logic implementation for account management operations
#[derive(Clone)]
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

// ---------------------------------------------------------------------------
//  Shared helpers
// ---------------------------------------------------------------------------

/// Helper function to convert proto `CommitmentLevel` to Solana `CommitmentConfig`
/// Provides sensible defaults when commitment level is not specified
pub(crate) fn commitment_level_to_config(commitment_level: i32) -> CommitmentConfig {
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

// ---------------------------------------------------------------------------
//  Trait implementation -- each method delegates to a handler sub-module
// ---------------------------------------------------------------------------

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
    async fn get_account(
        &self,
        request: Request<GetAccountRequest>,
    ) -> Result<Response<GetAccountResponse>, Status> {
        self.handle_get_account(request)
    }

    async fn generate_new_key_pair(
        &self,
        request: Request<GenerateNewKeyPairRequest>,
    ) -> Result<Response<GenerateNewKeyPairResponse>, Status> {
        self.handle_generate_new_key_pair(request)
    }

    async fn fund_native(
        &self,
        request: Request<FundNativeRequest>,
    ) -> Result<Response<FundNativeResponse>, Status> {
        self.handle_fund_native(request).await
    }

    async fn get_token_account_balance(
        &self,
        request: Request<GetTokenAccountBalanceRequest>,
    ) -> Result<Response<GetTokenAccountBalanceResponse>, Status> {
        self.handle_get_token_account_balance(request)
    }

    async fn get_associated_token_address(
        &self,
        request: Request<GetAssociatedTokenAddressRequest>,
    ) -> Result<Response<GetAssociatedTokenAddressResponse>, Status> {
        self.handle_get_associated_token_address(request)
    }
}
