//! Handler implementation for `TransferToken` (`TransferChecked`).

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::token::v1::{
    TransferTokenRequest, TransferTokenResponse,
};

use solana_commitment_config::CommitmentConfig;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{
    extension::StateWithExtensions, instruction::transfer_checked, state::Mint,
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

use super::super::helpers::parse_human_amount;
use super::TokenProgramServiceImpl;

impl TokenProgramServiceImpl {
    /// Creates a `TransferChecked` instruction.
    ///
    /// Reads the mint account on-chain to resolve the token program and
    /// decimal precision. The source and destination ATAs are derived from
    /// the supplied owner public keys and the mint.
    ///
    /// Multi-sig authorities are not yet supported.
    pub(crate) async fn handle_transfer_token(
        &self,
        request: Request<TransferTokenRequest>,
    ) -> Result<Response<TransferTokenResponse>, Status> {
        let req = request.into_inner();

        // --- validate & parse inputs ----------------------------------------
        if req.mint_pub_key.is_empty() {
            return Err(Status::invalid_argument("mint_pub_key is required"));
        }
        if req.source_owner_pub_key.is_empty() {
            return Err(Status::invalid_argument("source_owner_pub_key is required"));
        }
        if req.destination_owner_pub_key.is_empty() {
            return Err(Status::invalid_argument("destination_owner_pub_key is required"));
        }
        if req.amount.is_empty() {
            return Err(Status::invalid_argument("amount is required"));
        }

        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let source_owner_pubkey = Pubkey::from_str(&req.source_owner_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid source_owner_pub_key: {e}")))?;
        let destination_owner_pubkey =
            Pubkey::from_str(&req.destination_owner_pub_key).map_err(|e| {
                Status::invalid_argument(format!("Invalid destination_owner_pub_key: {e}"))
            })?;

        // --- fetch & parse mint account on-chain ----------------------------
        let account = self
            .rpc_client
            .get_account_with_commitment(&mint_pubkey, CommitmentConfig::confirmed())
            .await
            .map_err(|e| Status::internal(format!("Failed to get mint account: {e}")))?
            .value
            .ok_or_else(|| Status::not_found("Mint account not found"))?;

        if account.owner != SPL_TOKEN_PROGRAM_ID && account.owner != TOKEN_2022_PROGRAM_ID {
            return Err(Status::invalid_argument(format!(
                "Account owner {} is not a known token program",
                account.owner,
            )));
        }

        let token_program_id = account.owner;

        // Unpack the mint to read decimals.
        let mint = if token_program_id == TOKEN_2022_PROGRAM_ID {
            StateWithExtensions::<Mint>::unpack(&account.data)
                .map_err(|e| Status::internal(format!("Failed to parse Token-2022 mint: {e}")))?
                .base
        } else {
            Mint::unpack(&account.data)
                .map_err(|e| Status::internal(format!("Failed to parse mint account: {e}")))?
        };

        let decimals = mint.decimals;

        // --- convert human-readable amount to base units --------------------
        let amount = parse_human_amount(&req.amount, decimals)?;

        // --- derive the source and destination ATAs -------------------------
        let source_ata = get_associated_token_address_with_program_id(
            &source_owner_pubkey,
            &mint_pubkey,
            &token_program_id,
        );
        let destination_ata = get_associated_token_address_with_program_id(
            &destination_owner_pubkey,
            &mint_pubkey,
            &token_program_id,
        );

        // --- build instruction ----------------------------------------------
        let instruction = transfer_checked(
            &token_program_id,
            &source_ata,
            &mint_pubkey,
            &destination_ata,
            &source_owner_pubkey,
            &[], // no additional signers — multi-sig not yet supported
            amount,
            decimals,
        )
        .map_err(|e| {
            Status::invalid_argument(format!("Failed to create TransferChecked instruction: {e}"))
        })?;

        Ok(Response::new(TransferTokenResponse {
            instruction: Some(sdk_instruction_to_proto(instruction)),
        }))
    }
}
