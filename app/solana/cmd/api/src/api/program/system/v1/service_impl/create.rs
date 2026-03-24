use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use solana_system_interface::{instruction as system_instruction, program as system_program};
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;
use protochain_api::protochain::solana::program::system::v1::{
    CreateRequest, CreateResponse, CreateWithSeedRequest, CreateWithSeedResponse,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::SystemProgramServiceImpl {
    pub(super) fn handle_create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<CreateResponse>, Status> {
        let req = request.into_inner();

        // Validation
        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.new_account.is_empty() {
            return Err(Status::invalid_argument("New account address is required"));
        }

        let payer = Pubkey::from_str(&req.payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer address: {e}")))?;

        let new_account = Pubkey::from_str(&req.new_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid new account address: {e}")))?;

        // Parse owner program (default to system program if empty)
        let owner = if req.owner.is_empty() {
            system_program::id()
        } else {
            Pubkey::from_str(&req.owner).map_err(|e| {
                Status::invalid_argument(format!("Invalid owner program address: {e}"))
            })?
        };

        // Build instruction using SDK
        let instruction = system_instruction::create_account(
            &payer,
            &new_account,
            req.lamports,
            req.space,
            &owner,
        );

        // Convert to proto format
        let mut proto_instruction = sdk_instruction_to_proto(instruction);

        // Add descriptive information for composable transactions
        let owner_display = if req.owner.is_empty() {
            "system program (default)".to_string()
        } else {
            req.owner.clone()
        };
        proto_instruction.description = format!(
            "Create account: {} (payer: {}, owner: {}, lamports: {}, space: {}, owner: {})",
            req.new_account, req.payer, owner_display, req.lamports, req.space, req.owner
        );

        Ok(Response::new(CreateResponse {
            instruction: Some(proto_instruction),
        }))
    }

    pub(super) fn handle_create_with_seed(
        &self,
        request: Request<CreateWithSeedRequest>,
    ) -> Result<Response<CreateWithSeedResponse>, Status> {
        let req = request.into_inner();

        if req.payer.is_empty() {
            return Err(Status::invalid_argument("Payer address is required"));
        }
        if req.new_account.is_empty() {
            return Err(Status::invalid_argument("New account address is required"));
        }
        if req.base.is_empty() {
            return Err(Status::invalid_argument("Base address is required"));
        }
        if req.seed.is_empty() {
            return Err(Status::invalid_argument("Seed is required"));
        }

        let payer = Pubkey::from_str(&req.payer)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer address: {e}")))?;

        let new_account = Pubkey::from_str(&req.new_account)
            .map_err(|e| Status::invalid_argument(format!("Invalid new account address: {e}")))?;

        let base = Pubkey::from_str(&req.base)
            .map_err(|e| Status::invalid_argument(format!("Invalid base address: {e}")))?;

        let instruction = system_instruction::create_account_with_seed(
            &payer,
            &new_account,
            &base,
            &req.seed,
            req.lamports,
            req.space,
            &system_program::id(),
        );

        Ok(Response::new(CreateWithSeedResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
