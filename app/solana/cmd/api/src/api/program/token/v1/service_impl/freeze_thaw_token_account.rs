//! Handler implementations for `FreezeTokenAccount` and `ThawTokenAccount`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::token::v1::{
    FreezeTokenAccountRequest, FreezeTokenAccountResponse, ThawTokenAccountRequest,
    ThawTokenAccountResponse,
};

use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{
    instruction::{freeze_account, thaw_account},
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

use super::TokenProgramServiceImpl;

// ---------------------------------------------------------------------------
//  Shared validation
// ---------------------------------------------------------------------------

/// Parsed and validated inputs shared by freeze and thaw handlers.
struct FreezeThawInputs {
    token_account_pubkey: Pubkey,
    mint_pubkey: Pubkey,
    freeze_authority_pubkey: Pubkey,
    token_program_id: Pubkey,
}

/// Validates request fields and fetches the token account to determine the
/// owning token program. Shared between freeze and thaw to avoid duplication.
async fn validate_and_resolve(
    rpc_client: &solana_client::nonblocking::rpc_client::RpcClient,
    token_account_pub_key: &str,
    mint_pub_key: &str,
    freeze_authority_pub_key: &str,
) -> Result<FreezeThawInputs, Status> {
    if token_account_pub_key.is_empty() {
        return Err(Status::invalid_argument("token_account_pub_key is required"));
    }
    if mint_pub_key.is_empty() {
        return Err(Status::invalid_argument("mint_pub_key is required"));
    }
    if freeze_authority_pub_key.is_empty() {
        return Err(Status::invalid_argument("freeze_authority_pub_key is required"));
    }

    let token_account_pubkey = Pubkey::from_str(token_account_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid token_account_pub_key: {e}")))?;
    let mint_pubkey = Pubkey::from_str(mint_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
    let freeze_authority_pubkey = Pubkey::from_str(freeze_authority_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid freeze_authority_pub_key: {e}")))?;

    let account = rpc_client
        .get_account_with_commitment(&token_account_pubkey, CommitmentConfig::confirmed())
        .await
        .map_err(|e| Status::internal(format!("Failed to get token account: {e}")))?
        .value
        .ok_or_else(|| Status::not_found("Token account not found"))?;

    if account.owner != SPL_TOKEN_PROGRAM_ID && account.owner != TOKEN_2022_PROGRAM_ID {
        return Err(Status::invalid_argument(format!(
            "Account owner {} is not a known token program",
            account.owner,
        )));
    }

    Ok(FreezeThawInputs {
        token_account_pubkey,
        mint_pubkey,
        freeze_authority_pubkey,
        token_program_id: account.owner,
    })
}

// ---------------------------------------------------------------------------
//  Handlers
// ---------------------------------------------------------------------------

impl TokenProgramServiceImpl {
    /// Creates a `FreezeAccount` instruction for a token account.
    ///
    /// Reads the token account on-chain to determine the owning token program.
    /// The instruction must be signed by the mint's freeze authority.
    pub(crate) async fn handle_freeze_token_account(
        &self,
        request: Request<FreezeTokenAccountRequest>,
    ) -> Result<Response<FreezeTokenAccountResponse>, Status> {
        let req = request.into_inner();

        let inputs = validate_and_resolve(
            &self.rpc_client,
            &req.token_account_pub_key,
            &req.mint_pub_key,
            &req.freeze_authority_pub_key,
        )
        .await?;

        let instruction = freeze_account(
            &inputs.token_program_id,
            &inputs.token_account_pubkey,
            &inputs.mint_pubkey,
            &inputs.freeze_authority_pubkey,
            &[], // no additional signers — multi-sig not yet supported
        )
        .map_err(|e| {
            Status::invalid_argument(format!("Failed to create FreezeAccount instruction: {e}"))
        })?;

        Ok(Response::new(FreezeTokenAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }

    /// Creates a `ThawAccount` instruction for a frozen token account.
    ///
    /// Reads the token account on-chain to determine the owning token program.
    /// The instruction must be signed by the mint's freeze authority.
    pub(crate) async fn handle_thaw_token_account(
        &self,
        request: Request<ThawTokenAccountRequest>,
    ) -> Result<Response<ThawTokenAccountResponse>, Status> {
        let req = request.into_inner();

        let inputs = validate_and_resolve(
            &self.rpc_client,
            &req.token_account_pub_key,
            &req.mint_pub_key,
            &req.freeze_authority_pub_key,
        )
        .await?;

        let instruction = thaw_account(
            &inputs.token_program_id,
            &inputs.token_account_pubkey,
            &inputs.mint_pubkey,
            &inputs.freeze_authority_pubkey,
            &[], // no additional signers — multi-sig not yet supported
        )
        .map_err(|e| {
            Status::invalid_argument(format!("Failed to create ThawAccount instruction: {e}"))
        })?;

        Ok(Response::new(ThawTokenAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
