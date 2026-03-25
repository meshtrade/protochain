use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use solana_system_interface::{instruction as system_instruction, program as system_program};
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;
use protochain_api::protochain::solana::program::system::v1::{
    TransferRequest, TransferResponse, TransferWithSeedRequest, TransferWithSeedResponse,
};

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::SystemProgramServiceImpl {
    pub(super) fn handle_transfer(
        &self,
        request: Request<TransferRequest>,
    ) -> Result<Response<TransferResponse>, Status> {
        let req = request.into_inner();

        if req.from.is_empty() {
            return Err(Status::invalid_argument("From address is required"));
        }
        if req.to.is_empty() {
            return Err(Status::invalid_argument("To address is required"));
        }

        let from = Pubkey::from_str(&req.from)
            .map_err(|e| Status::invalid_argument(format!("Invalid from address: {e}")))?;

        let to = Pubkey::from_str(&req.to)
            .map_err(|e| Status::invalid_argument(format!("Invalid to address: {e}")))?;

        let instruction = system_instruction::transfer(&from, &to, req.lamports);

        // Convert to proto format and add description
        let mut proto_instruction = sdk_instruction_to_proto(instruction);
        proto_instruction.description =
            format!("Transfer {} lamports from {} to {}", req.lamports, req.from, req.to);

        Ok(Response::new(TransferResponse {
            instruction: Some(proto_instruction),
        }))
    }

    pub(super) fn handle_transfer_with_seed(
        &self,
        request: Request<TransferWithSeedRequest>,
    ) -> Result<Response<TransferWithSeedResponse>, Status> {
        let req = request.into_inner();

        if req.from.is_empty() {
            return Err(Status::invalid_argument("From address is required"));
        }
        if req.from_base.is_empty() {
            return Err(Status::invalid_argument("From base address is required"));
        }
        if req.from_seed.is_empty() {
            return Err(Status::invalid_argument("From seed is required"));
        }
        if req.to.is_empty() {
            return Err(Status::invalid_argument("To address is required"));
        }

        let from = Pubkey::from_str(&req.from)
            .map_err(|e| Status::invalid_argument(format!("Invalid from address: {e}")))?;

        let from_base = Pubkey::from_str(&req.from_base)
            .map_err(|e| Status::invalid_argument(format!("Invalid from base address: {e}")))?;

        let to = Pubkey::from_str(&req.to)
            .map_err(|e| Status::invalid_argument(format!("Invalid to address: {e}")))?;

        let instruction = system_instruction::transfer_with_seed(
            &from,
            &from_base,
            req.from_seed.clone(),
            &system_program::id(),
            &to,
            req.lamports,
        );

        Ok(Response::new(TransferWithSeedResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
