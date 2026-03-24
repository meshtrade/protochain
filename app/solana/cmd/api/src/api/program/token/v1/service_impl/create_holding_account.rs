//! Handler implementations for `CreateToken2022HoldingAccount` and
//! `CreateSPLTokenHoldingAccount`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_holding_account_extension, CreateSplTokenHoldingAccountRequest,
    CreateSplTokenHoldingAccountResponse, CreateToken2022HoldingAccountRequest,
    CreateToken2022HoldingAccountResponse,
};

use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{
    extension::{memo_transfer::instruction::enable_required_transfer_memos, ExtensionType},
    instruction::reallocate,
    state::Account,
    ID as TOKEN_2022_PROGRAM_ID,
};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

use super::super::extensions::helpers::{
    holding_account_total_space, validate_no_duplicate_holding_account_extensions,
};
use super::TokenProgramServiceImpl;

impl TokenProgramServiceImpl {
    /// Creates a Token-2022 holding account (ATA) with optional extensions.
    ///
    /// Combines ATA creation, rent calculation, and extension-init instructions
    /// into a single response. For each requested extension, appends the
    /// necessary reallocate + extension-init instructions after the ATA creation.
    pub(crate) async fn handle_create_token2022_holding_account(
        &self,
        request: Request<CreateToken2022HoldingAccountRequest>,
    ) -> Result<Response<CreateToken2022HoldingAccountResponse>, Status> {
        let req = request.into_inner();

        if req.payer_pub_key.is_empty() {
            return Err(Status::invalid_argument("payer_pub_key is required"));
        }
        if req.owner_pub_key.is_empty() {
            return Err(Status::invalid_argument("owner_pub_key is required"));
        }
        if req.mint_pub_key.is_empty() {
            return Err(Status::invalid_argument("mint_pub_key is required"));
        }

        let payer_pubkey = Pubkey::from_str(&req.payer_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer_pub_key: {e}")))?;
        let owner_pubkey = Pubkey::from_str(&req.owner_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner_pub_key: {e}")))?;
        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;

        validate_no_duplicate_holding_account_extensions(&req.extensions)?;

        // Calculate rent for the final account size (base + all extensions)
        let total_space = holding_account_total_space(&req.extensions)?;
        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(total_space)
            .await
            .map_err(|e| {
                Status::internal(format!("failed to get minimum balance for holding account: {e}"))
            })?;

        let mut instructions: Vec<protochain_api::SolanaInstruction> = Vec::new();

        // 1. Create Associated Token Account
        let create_ata_ix = create_associated_token_account(
            &payer_pubkey,
            &owner_pubkey,
            &mint_pubkey,
            &TOKEN_2022_PROGRAM_ID,
        );
        instructions.push(sdk_instruction_to_proto(create_ata_ix));

        // 2. For each extension: reallocate + init
        if !req.extensions.is_empty() {
            let ata_address = get_associated_token_address_with_program_id(
                &owner_pubkey,
                &mint_pubkey,
                &TOKEN_2022_PROGRAM_ID,
            );

            for ext in &req.extensions {
                match ext
                    .extension
                    .as_ref()
                    .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
                {
                    token2022_holding_account_extension::Extension::MemoTransfer(cfg) => {
                        // Reallocate account to include MemoTransfer extension
                        let reallocate_ix = reallocate(
                            &TOKEN_2022_PROGRAM_ID,
                            &ata_address,
                            &payer_pubkey,
                            &owner_pubkey,
                            &[&owner_pubkey],
                            &[ExtensionType::MemoTransfer],
                        )
                        .map_err(|e| {
                            Status::internal(format!(
                                "could not create reallocation instruction for memo extension: {e}"
                            ))
                        })?;
                        instructions.push(sdk_instruction_to_proto(reallocate_ix));

                        // Enable required transfer memos if configured
                        if cfg.require_incoming_memo {
                            let enable_memo_ix = enable_required_transfer_memos(
                                &TOKEN_2022_PROGRAM_ID,
                                &ata_address,
                                &owner_pubkey,
                                &[&payer_pubkey],
                            )
                            .map_err(|e| {
                                Status::internal(format!(
                                    "could not create enable_required_transfer_memos instruction: {e}"
                                ))
                            })?;
                            instructions.push(sdk_instruction_to_proto(enable_memo_ix));
                        }
                    }
                }
            }
        }

        Ok(Response::new(CreateToken2022HoldingAccountResponse {
            instructions,
            lamports,
        }))
    }

    /// Creates a legacy SPL Token holding account (ATA) in one call.
    ///
    /// Returns a single ATA creation instruction and the rent-exempt lamport cost.
    pub(crate) async fn handle_create_spl_token_holding_account(
        &self,
        request: Request<CreateSplTokenHoldingAccountRequest>,
    ) -> Result<Response<CreateSplTokenHoldingAccountResponse>, Status> {
        let req = request.into_inner();

        if req.payer_pub_key.is_empty() {
            return Err(Status::invalid_argument("payer_pub_key is required"));
        }
        if req.owner_pub_key.is_empty() {
            return Err(Status::invalid_argument("owner_pub_key is required"));
        }
        if req.mint_pub_key.is_empty() {
            return Err(Status::invalid_argument("mint_pub_key is required"));
        }

        let payer_pubkey = Pubkey::from_str(&req.payer_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid payer_pub_key: {e}")))?;
        let owner_pubkey = Pubkey::from_str(&req.owner_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid owner_pub_key: {e}")))?;
        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;

        // SPL Token accounts are always Account::LEN (165 bytes)
        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Account::LEN)
            .await
            .map_err(|e| {
                Status::internal(format!("failed to get minimum balance for holding account: {e}"))
            })?;

        let create_ata_ix = create_associated_token_account(
            &payer_pubkey,
            &owner_pubkey,
            &mint_pubkey,
            &SPL_TOKEN_PROGRAM_ID,
        );

        Ok(Response::new(CreateSplTokenHoldingAccountResponse {
            instructions: vec![sdk_instruction_to_proto(create_ata_ix)],
            lamports,
        }))
    }
}
