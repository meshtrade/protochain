//! Handler implementation for `CloseTokenAccount`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::token::v1::{
    CloseTokenAccountRequest, CloseTokenAccountResponse,
};

use solana_commitment_config::CommitmentConfig;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{
    extension::StateWithExtensions, instruction::close_account, state::Account as TokenAccount,
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

use super::TokenProgramServiceImpl;

impl TokenProgramServiceImpl {
    /// Creates a `CloseAccount` instruction for a token account.
    ///
    /// Reads the token account on-chain to determine the owning token program
    /// (SPL Token or Token-2022). The token account must have a zero token
    /// balance before it can be closed.
    ///
    /// Multi-sig authorities are not yet supported.
    pub(crate) async fn handle_close_token_account(
        &self,
        request: Request<CloseTokenAccountRequest>,
    ) -> Result<Response<CloseTokenAccountResponse>, Status> {
        let req = request.into_inner();

        // --- validate inputs ------------------------------------------------
        if req.token_account_pub_key.is_empty() {
            return Err(Status::invalid_argument("token_account_pub_key is required"));
        }
        if req.destination_pub_key.is_empty() {
            return Err(Status::invalid_argument("destination_pub_key is required"));
        }
        if req.authority_pub_key.is_empty() {
            return Err(Status::invalid_argument("authority_pub_key is required"));
        }

        let token_account_pubkey = Pubkey::from_str(&req.token_account_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid token_account_pub_key: {e}")))?;
        let destination_pubkey = Pubkey::from_str(&req.destination_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid destination_pub_key: {e}")))?;
        let authority_pubkey = Pubkey::from_str(&req.authority_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid authority_pub_key: {e}")))?;

        // --- fetch account to determine the token program -------------------
        let account = self
            .rpc_client
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

        let token_program_id = account.owner;

        // --- check token balance is zero ------------------------------------
        let token_balance = if token_program_id == TOKEN_2022_PROGRAM_ID {
            StateWithExtensions::<TokenAccount>::unpack(&account.data)
                .map_err(|e| {
                    Status::internal(format!("Failed to parse Token-2022 account data: {e}"))
                })?
                .base
                .amount
        } else {
            TokenAccount::unpack(&account.data)
                .map_err(|e| Status::internal(format!("Failed to parse token account data: {e}")))?
                .amount
        };

        if token_balance > 0 {
            return Err(Status::failed_precondition(format!(
                "Token account still holds {token_balance} tokens; \
                 transfer or burn all tokens before closing the account"
            )));
        }

        // --- build instruction ----------------------------------------------
        let instruction = close_account(
            &token_program_id,
            &token_account_pubkey,
            &destination_pubkey,
            &authority_pubkey,
            &[], // no additional signers — multi-sig not yet supported
        )
        .map_err(|e| {
            Status::invalid_argument(format!("Failed to create CloseAccount instruction: {e}"))
        })?;

        Ok(Response::new(CloseTokenAccountResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
