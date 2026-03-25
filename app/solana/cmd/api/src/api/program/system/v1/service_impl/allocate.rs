use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use solana_system_interface::{instruction as system_instruction, program as system_program};
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;
use protochain_api::protochain::solana::program::system::v1::{
    AllocateRequest, AllocateResponse, AllocateWithSeedRequest, AllocateWithSeedResponse,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::SystemProgramServiceImpl {
    pub(super) fn handle_allocate(
        &self,
        request: Request<AllocateRequest>,
    ) -> Result<Response<AllocateResponse>, Status> {
        let req = request.into_inner();

        if req.account.is_empty() {
            return Err(Status::invalid_argument("Account address is required"));
        }

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let instruction = system_instruction::allocate(&account, req.space);
        Ok(Response::new(AllocateResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    pub(super) fn handle_allocate_with_seed(
        &self,
        request: Request<AllocateWithSeedRequest>,
    ) -> Result<Response<AllocateWithSeedResponse>, Status> {
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

        let account = Pubkey::from_str(&req.account)
            .map_err(|e| Status::invalid_argument(format!("Invalid account address: {e}")))?;

        let base = Pubkey::from_str(&req.base)
            .map_err(|e| Status::invalid_argument(format!("Invalid base address: {e}")))?;

        let instruction = system_instruction::allocate_with_seed(
            &account,
            &base,
            &req.seed,
            req.space,
            &system_program::id(),
        );

        Ok(Response::new(AllocateWithSeedResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
