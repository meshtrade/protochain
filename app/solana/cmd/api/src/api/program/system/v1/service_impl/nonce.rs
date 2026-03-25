use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;
use protochain_api::protochain::solana::program::system::v1::{
    AdvanceNonceAccountRequest, AdvanceNonceAccountResponse, AuthorizeNonceAccountRequest,
    AuthorizeNonceAccountResponse, InitializeNonceAccountRequest, InitializeNonceAccountResponse,
    UpgradeNonceAccountRequest, UpgradeNonceAccountResponse, WithdrawNonceAccountRequest,
    WithdrawNonceAccountResponse,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::SystemProgramServiceImpl {
    pub(super) fn handle_initialize_nonce_account(
        &self,
        request: Request<InitializeNonceAccountRequest>,
    ) -> Result<Response<InitializeNonceAccountResponse>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.authority.is_empty() {
            return Err(Status::invalid_argument("Authority address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let authority = Pubkey::from_str(&req.authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority address: {e}")))?;

        // Note: initialize_nonce_account might not be available in this solana-sdk version
        // Using create_nonce_account which returns Vec<Instruction>, take the second one (initialize)
        let instructions = system_instruction::create_nonce_account(
            &authority,     // payer
            &nonce_account, // nonce account
            &authority,     // authority
            1_000_000,      // minimum balance for nonce account
        );
        // Take the initialize instruction (second one) - first is create_account
        let instruction = instructions
            .into_iter()
            .nth(1)
            .ok_or_else(|| Status::internal("Failed to create initialize nonce instruction"))?;

        Ok(Response::new(InitializeNonceAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    pub(super) fn handle_authorize_nonce_account(
        &self,
        request: Request<AuthorizeNonceAccountRequest>,
    ) -> Result<Response<AuthorizeNonceAccountResponse>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.current_authority.is_empty() {
            return Err(Status::invalid_argument("Current authority address is required"));
        }
        if req.new_authority.is_empty() {
            return Err(Status::invalid_argument("New authority address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let current_authority = Pubkey::from_str(&req.current_authority).map_err(|e| {
            Status::invalid_argument(format!("Invalid current authority address: {e}"))
        })?;

        let new_authority = Pubkey::from_str(&req.new_authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid new authority address: {e}")))?;

        let instruction = system_instruction::authorize_nonce_account(
            &nonce_account,
            &current_authority,
            &new_authority,
        );

        Ok(Response::new(AuthorizeNonceAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    pub(super) fn handle_withdraw_nonce_account(
        &self,
        request: Request<WithdrawNonceAccountRequest>,
    ) -> Result<Response<WithdrawNonceAccountResponse>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.authority.is_empty() {
            return Err(Status::invalid_argument("Authority address is required"));
        }
        if req.to.is_empty() {
            return Err(Status::invalid_argument("To address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let authority = Pubkey::from_str(&req.authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority address: {e}")))?;

        let to = Pubkey::from_str(&req.to)
            .map_err(|e| Status::invalid_argument(format!("Invalid to address: {e}")))?;

        let instruction = system_instruction::withdraw_nonce_account(
            &nonce_account,
            &authority,
            &to,
            req.lamports,
        );

        Ok(Response::new(WithdrawNonceAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    pub(super) fn handle_advance_nonce_account(
        &self,
        request: Request<AdvanceNonceAccountRequest>,
    ) -> Result<Response<AdvanceNonceAccountResponse>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }
        if req.authority.is_empty() {
            return Err(Status::invalid_argument("Authority address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let authority = Pubkey::from_str(&req.authority)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority address: {e}")))?;

        let instruction = system_instruction::advance_nonce_account(&nonce_account, &authority);

        Ok(Response::new(AdvanceNonceAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    pub(super) fn handle_upgrade_nonce_account(
        &self,
        request: Request<UpgradeNonceAccountRequest>,
    ) -> Result<Response<UpgradeNonceAccountResponse>, Status> {
        let req = request.into_inner();

        if req.nonce_account.is_empty() {
            return Err(Status::invalid_argument("Nonce account address is required"));
        }

        let nonce_account = Pubkey::from_str(&req.nonce_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid nonce account address: {e}")))?;

        let instruction = system_instruction::upgrade_nonce_account(nonce_account);

        Ok(Response::new(UpgradeNonceAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
