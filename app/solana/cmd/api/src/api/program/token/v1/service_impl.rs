//! Token Program gRPC service implementation.
//!
//! The `TokenProgramService` trait impl lives here, with each method delegating
//! to a handler defined in a sub-module under `service_impl/`.
//!
//! Sub-modules:
//! - [`create_mint`]            — `CreateToken2022Mint`, `CreateSPLTokenMint`
//! - [`parse_mint`]             — `ParseMint`
//! - [`create_holding_account`] — `CreateToken2022HoldingAccount`, `CreateSPLTokenHoldingAccount`
//! - [`mint`]                   — `Mint` (MintToChecked)
//! - [`transfer`]               — `TransferToken` (TransferChecked)
//! - [`burn`]                   — `BurnToken` (BurnChecked)
//! - [`close_token_account`]    — `CloseTokenAccount`
//! - [`freeze_thaw_token_account`] — `FreezeTokenAccount`, `ThawTokenAccount`

mod burn;
mod close_token_account;
mod create_holding_account;
mod create_mint;
mod freeze_thaw_token_account;
mod mint;
mod parse_mint;
mod transfer;

use std::sync::Arc;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::token::v1::{
    service_server::Service as TokenProgramService, BurnTokenRequest, BurnTokenResponse,
    CloseTokenAccountRequest, CloseTokenAccountResponse, CreateSplTokenHoldingAccountRequest,
    CreateSplTokenHoldingAccountResponse, CreateSplTokenMintRequest, CreateSplTokenMintResponse,
    CreateToken2022HoldingAccountRequest, CreateToken2022HoldingAccountResponse,
    CreateToken2022MintRequest, CreateToken2022MintResponse, FreezeTokenAccountRequest,
    FreezeTokenAccountResponse, MintRequest, MintResponse, ParseMintRequest, ParseMintResponse,
    ThawTokenAccountRequest, ThawTokenAccountResponse, TransferTokenRequest, TransferTokenResponse,
};

use solana_client::nonblocking::rpc_client::RpcClient;

use crate::api::program::system::v1::SystemProgramServiceImpl;

// ---------------------------------------------------------------------------
//  Struct definition
// ---------------------------------------------------------------------------

/// Token Program service implementation for Token 2022 and SPL Token operations.
///
/// Depends on the System Program service for building `System::CreateAccount`
/// instructions — ensuring a single source of truth for account creation logic
/// across all services.
#[derive(Clone)]
pub struct TokenProgramServiceImpl {
    /// Solana RPC client for blockchain interactions
    pub(crate) rpc_client: Arc<RpcClient>,
    /// System Program service for creating account instructions
    pub(crate) system_program_service: Arc<SystemProgramServiceImpl>,
}

impl TokenProgramServiceImpl {
    /// Creates a new `TokenProgramServiceImpl` instance with the provided dependencies
    pub const fn new(
        rpc_client: Arc<RpcClient>,
        system_program_service: Arc<SystemProgramServiceImpl>,
    ) -> Self {
        Self {
            rpc_client,
            system_program_service,
        }
    }
}

// ---------------------------------------------------------------------------
//  Trait implementation — each method delegates to a handler sub-module
// ---------------------------------------------------------------------------

#[tonic::async_trait]
impl TokenProgramService for TokenProgramServiceImpl {
    async fn create_token2022_mint(
        &self,
        request: Request<CreateToken2022MintRequest>,
    ) -> Result<Response<CreateToken2022MintResponse>, Status> {
        self.handle_create_token2022_mint(request).await
    }

    async fn create_spl_token_mint(
        &self,
        request: Request<CreateSplTokenMintRequest>,
    ) -> Result<Response<CreateSplTokenMintResponse>, Status> {
        self.handle_create_spl_token_mint(request).await
    }

    async fn parse_mint(
        &self,
        request: Request<ParseMintRequest>,
    ) -> Result<Response<ParseMintResponse>, Status> {
        self.handle_parse_mint(request).await
    }

    async fn create_token2022_holding_account(
        &self,
        request: Request<CreateToken2022HoldingAccountRequest>,
    ) -> Result<Response<CreateToken2022HoldingAccountResponse>, Status> {
        self.handle_create_token2022_holding_account(request).await
    }

    async fn create_spl_token_holding_account(
        &self,
        request: Request<CreateSplTokenHoldingAccountRequest>,
    ) -> Result<Response<CreateSplTokenHoldingAccountResponse>, Status> {
        self.handle_create_spl_token_holding_account(request).await
    }

    async fn mint(&self, request: Request<MintRequest>) -> Result<Response<MintResponse>, Status> {
        self.handle_mint(request).await
    }

    async fn freeze_token_account(
        &self,
        request: Request<FreezeTokenAccountRequest>,
    ) -> Result<Response<FreezeTokenAccountResponse>, Status> {
        self.handle_freeze_token_account(request).await
    }

    async fn thaw_token_account(
        &self,
        request: Request<ThawTokenAccountRequest>,
    ) -> Result<Response<ThawTokenAccountResponse>, Status> {
        self.handle_thaw_token_account(request).await
    }

    async fn burn_token(
        &self,
        request: Request<BurnTokenRequest>,
    ) -> Result<Response<BurnTokenResponse>, Status> {
        self.handle_burn_token(request).await
    }

    async fn transfer_token(
        &self,
        request: Request<TransferTokenRequest>,
    ) -> Result<Response<TransferTokenResponse>, Status> {
        self.handle_transfer_token(request).await
    }

    async fn close_token_account(
        &self,
        request: Request<CloseTokenAccountRequest>,
    ) -> Result<Response<CloseTokenAccountResponse>, Status> {
        self.handle_close_token_account(request).await
    }
}

#[cfg(test)]
mod tests;
