//! Handler implementations for `CreateToken2022Mint` and `CreateSPLTokenMint`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::system::v1::{
    service_server::Service as SystemProgramService, CreateRequest,
};
use protochain_api::protochain::solana::program::token::v1::{
    CreateSplTokenMintRequest, CreateSplTokenMintResponse, CreateToken2022MintRequest,
    CreateToken2022MintResponse,
};

use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{instruction::initialize_mint2, state::Mint, ID as TOKEN_2022_PROGRAM_ID};

use crate::api::common::solana_conversions::sdk_instruction_to_proto;

use super::super::extensions::build_token2022_mint_instructions;
use super::super::extensions::helpers::validate_no_duplicate_extensions;
use super::super::helpers::{
    mint_create_account_space, mint_total_space_for_rent, validate_decimals,
};
use super::super::metaplex::build_create_metaplex_metadata_instruction;
use super::TokenProgramServiceImpl;

impl TokenProgramServiceImpl {
    /// Creates a fully initialised Token-2022 mint account in one call.
    ///
    /// Combines rent calculation, `System::CreateAccount`, and all Token-2022
    /// initialisation instructions into a single response. The instruction
    /// order is:
    ///   1. `System::CreateAccount` (via injected system program service)
    ///   2. Extension pre-init (e.g. metadata pointer)
    ///   3. `initialize_mint`
    ///   4. Extension post-init (e.g. token metadata init, `update_field` × N)
    pub(crate) async fn handle_create_token2022_mint(
        &self,
        request: Request<CreateToken2022MintRequest>,
    ) -> Result<Response<CreateToken2022MintResponse>, Status> {
        let req = request.into_inner();

        if req.payer_pub_key.is_empty() {
            return Err(Status::invalid_argument("payer_pub_key is required"));
        }

        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let mint_authority = Pubkey::from_str(&req.mint_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid mint_authority_pub_key: {e}"))
        })?;

        let freeze_authority = if req.freeze_authority_pub_key.is_empty() {
            None
        } else {
            Some(Pubkey::from_str(&req.freeze_authority_pub_key).map_err(|e| {
                Status::invalid_argument(format!("Invalid freeze_authority_pub_key: {e}"))
            })?)
        };

        let decimals = validate_decimals(req.decimals)?;

        validate_no_duplicate_extensions(&req.extensions)?;

        // Calculate space and rent
        let space = mint_create_account_space(&req.extensions)?;
        let rent_space = mint_total_space_for_rent(&req.extensions)?;
        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(rent_space)
            .await
            .map_err(|e| {
                Status::internal(format!("failed to get minimum balance for mint account: {e}"))
            })?;

        // 1. Build System::CreateAccount instruction via the system program service
        let create_account_resp = self
            .system_program_service
            .create(Request::new(CreateRequest {
                payer: req.payer_pub_key.clone(),
                new_account: req.mint_pub_key.clone(),
                owner: TOKEN_2022_PROGRAM_ID.to_string(),
                lamports,
                space: space as u64,
            }))
            .await?;

        let create_account_instruction = create_account_resp
            .into_inner()
            .instruction
            .ok_or_else(|| Status::internal("System program did not return an instruction"))?;

        // 2. Build Token-2022 initialisation instructions
        let sdk_instructions = build_token2022_mint_instructions(
            &TOKEN_2022_PROGRAM_ID,
            &mint_pubkey,
            &mint_authority,
            freeze_authority.as_ref(),
            decimals,
            &req.extensions,
        )?;

        // Assemble: system create + all token init instructions
        let mut instructions = Vec::with_capacity(1 + sdk_instructions.len());
        instructions.push(create_account_instruction);
        for ix in sdk_instructions {
            instructions.push(sdk_instruction_to_proto(ix));
        }

        Ok(Response::new(CreateToken2022MintResponse {
            instructions,
            lamports,
            space: space as u64,
        }))
    }

    /// Creates a fully initialised legacy SPL Token mint account in one call.
    ///
    /// Combines rent calculation, `System::CreateAccount`, `initialize_mint`, and
    /// optionally a Metaplex `CreateMetadataAccountV3` instruction.
    pub(crate) async fn handle_create_spl_token_mint(
        &self,
        request: Request<CreateSplTokenMintRequest>,
    ) -> Result<Response<CreateSplTokenMintResponse>, Status> {
        let req = request.into_inner();

        if req.payer_pub_key.is_empty() {
            return Err(Status::invalid_argument("payer_pub_key is required"));
        }

        let mint_pubkey = Pubkey::from_str(&req.mint_pub_key)
            .map_err(|e| Status::invalid_argument(format!("Invalid mint_pub_key: {e}")))?;
        let mint_authority = Pubkey::from_str(&req.mint_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid mint_authority_pub_key: {e}"))
        })?;

        let freeze_authority = if req.freeze_authority_pub_key.is_empty() {
            None
        } else {
            Some(Pubkey::from_str(&req.freeze_authority_pub_key).map_err(|e| {
                Status::invalid_argument(format!("Invalid freeze_authority_pub_key: {e}"))
            })?)
        };

        let decimals = validate_decimals(req.decimals)?;

        // SPL Token mints are always exactly Mint::LEN (82 bytes)
        let space = Mint::LEN;
        let lamports = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(space)
            .await
            .map_err(|e| {
                Status::internal(format!("failed to get minimum balance for mint account: {e}"))
            })?;

        let mut instructions = Vec::new();

        // 1. System::CreateAccount via the system program service
        let create_account_resp = self
            .system_program_service
            .create(Request::new(CreateRequest {
                payer: req.payer_pub_key.clone(),
                new_account: req.mint_pub_key.clone(),
                owner: SPL_TOKEN_PROGRAM_ID.to_string(),
                lamports,
                space: space as u64,
            }))
            .await?;

        let create_account_instruction = create_account_resp
            .into_inner()
            .instruction
            .ok_or_else(|| Status::internal("System program did not return an instruction"))?;
        instructions.push(create_account_instruction);

        // 2. initialize_mint instruction
        let init_mint_ix = initialize_mint2(
            &SPL_TOKEN_PROGRAM_ID,
            &mint_pubkey,
            &mint_authority,
            freeze_authority.as_ref(),
            decimals,
        )
        .map_err(|e| {
            Status::internal(format!("could not create initialise mint token instruction: {e}"))
        })?;
        instructions.push(sdk_instruction_to_proto(init_mint_ix));

        // 3. Optional Metaplex metadata instruction
        if let Some(ref metadata) = req.metadata {
            let payer = Pubkey::from_str(&req.payer_pub_key)
                .map_err(|e| Status::invalid_argument(format!("Invalid payer_pub_key: {e}")))?;

            let create_metadata_ix = build_create_metaplex_metadata_instruction(
                &mint_pubkey,
                &mint_authority,
                &payer,
                metadata,
            )?;
            instructions.push(sdk_instruction_to_proto(create_metadata_ix));
        }

        Ok(Response::new(CreateSplTokenMintResponse {
            instructions,
            lamports,
            space: space as u64,
        }))
    }
}
