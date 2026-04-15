use std::str::FromStr;

use protochain_api::protochain::solana::program::system::v1::{
    BuildMemoRequest, BuildMemoResponse,
};
use solana_sdk::pubkey::Pubkey;
use spl_memo_interface::{instruction::build_memo, v3::ID as MEMO_V3_PROGRAM_ID};
use tonic::{Request, Response, Status};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

#[allow(clippy::result_large_err, clippy::unused_self)]
impl super::SystemProgramServiceImpl {
    pub(super) fn handle_build_memo(
        &self,
        request: Request<BuildMemoRequest>,
    ) -> Result<Response<BuildMemoResponse>, Status> {
        let req = request.into_inner();

        // Validation
        if req.memo.is_empty() {
            return Err(Status::invalid_argument("Memo message is required"));
        }

        let pubkeys: Vec<Pubkey> = req
            .signers
            .iter()
            .map(|signer| {
                Pubkey::from_str(signer)
                    .map_err(|e| Status::invalid_argument(format!("Invalid signer address: {e}")))
            })
            .collect::<Result<_, _>>()?;

        let pubkey_refs: Vec<&Pubkey> = pubkeys.iter().collect();

        let instruction = build_memo(&MEMO_V3_PROGRAM_ID, req.memo.as_bytes(), &pubkey_refs);

        Ok(Response::new(BuildMemoResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
