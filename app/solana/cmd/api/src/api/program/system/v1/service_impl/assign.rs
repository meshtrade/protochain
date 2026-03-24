use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction as system_instruction;
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;
use protochain_api::protochain::solana::program::system::v1::{
    AssignRequest, AssignResponse, AssignWithSeedRequest, AssignWithSeedResponse,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::SystemProgramServiceImpl {
    pub(super) fn handle_assign(
        &self,
        request: Request<AssignRequest>,
    ) -> Result<Response<AssignResponse>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }
        if req.owner_program.is_empty() {
            return Err(Status::invalid_argument("Owner program is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let owner_program = Pubkey::from_str(&req.owner_program)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner program: {e}")))?;

        let instruction = system_instruction::assign(&account, &owner_program);
        Ok(Response::new(AssignResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    pub(super) fn handle_assign_with_seed(
        &self,
        request: Request<AssignWithSeedRequest>,
    ) -> Result<Response<AssignWithSeedResponse>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }
        if req.base.is_empty() {
            return Err(Status::invalid_argument("Base address is required"));
        }
        if req.seed.is_empty() {
            return Err(Status::invalid_argument("Seed is required"));
        }
        if req.owner_program.is_empty() {
            return Err(Status::invalid_argument("Owner program is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let base = Pubkey::from_str(&req.base)
            .map_err(|e| Status::invalid_argument(format!("Invalid base address: {e}")))?;

        let owner_program = Pubkey::from_str(&req.owner_program)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner program: {e}")))?;

        let instruction =
            system_instruction::assign_with_seed(&account, &base, &req.seed, &owner_program);

        Ok(Response::new(AssignWithSeedResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
