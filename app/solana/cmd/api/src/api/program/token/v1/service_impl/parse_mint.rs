//! Handler implementation for `ParseMint`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use protochain_api::protochain::solana::program::token::v1::{
    MintInfo, ParseMintRequest, ParseMintResponse,
};

use solana_commitment_config::CommitmentConfig;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{extension::StateWithExtensions, state::Mint, ID as TOKEN_2022_PROGRAM_ID};

use crate::api::program::token::v1::extensions::extract_token2022_extensions;
use crate::api::program::token::v1::token_program::sdk_token_program_to_proto;

use super::TokenProgramServiceImpl;

impl TokenProgramServiceImpl {
    /// Parses mint account data into structured format.
    ///
    /// Supports both Legacy SPL Token and Token-2022 mints. The account owner is
    /// checked to ensure it belongs to a known token program, and the appropriate
    /// unpacking strategy is used (Token-2022 mints may contain extension data
    /// beyond the base 82-byte Mint layout).
    #[allow(clippy::unused_async)]
    pub(crate) async fn handle_parse_mint(
        &self,
        request: Request<ParseMintRequest>,
    ) -> Result<Response<ParseMintResponse>, Status> {
        let req = request.into_inner();

        // Parse the account address
        let account_pubkey = Pubkey::from_str(&req.account_address)
            .map_err(|e| Status::invalid_argument(format!("Invalid account_address: {e}")))?;

        // Get the account data
        let account = self
            .rpc_client
            .get_account_with_commitment(&account_pubkey, CommitmentConfig::confirmed())
            .map_err(|e| Status::internal(format!("Failed to get account: {e}")))?
            .value
            .ok_or_else(|| Status::not_found("Account not found"))?;

        // Validate account is owned by a known token program
        if account.owner != SPL_TOKEN_PROGRAM_ID && account.owner != TOKEN_2022_PROGRAM_ID {
            return Err(Status::invalid_argument(format!(
                "Account owner {} is not a known token program",
                account.owner,
            )));
        }

        // Determine which token program owns this mint.
        let token_program = sdk_token_program_to_proto(&account.owner);

        // Unpack the mint account data and extract extensions.
        // Use StateWithExtensions for Token-2022 accounts which may have extension
        // data beyond the base 82-byte Mint layout; use Mint::unpack for legacy
        // SPL accounts which are always exactly 82 bytes.
        let (mint, extensions) = if account.owner == TOKEN_2022_PROGRAM_ID {
            let state = StateWithExtensions::<Mint>::unpack(&account.data).map_err(|e| {
                Status::invalid_argument(format!("Failed to parse Token-2022 mint: {e}"))
            })?;

            let extensions = extract_token2022_extensions(&state, &account_pubkey);
            (state.base, extensions)
        } else {
            let mint = Mint::unpack(&account.data).map_err(|e| {
                Status::invalid_argument(format!("Failed to parse mint account: {e}"))
            })?;
            (mint, Vec::new())
        };

        Ok(Response::new(ParseMintResponse {
            mint: Some(MintInfo {
                mint_authority_pub_key: mint
                    .mint_authority
                    .map(|key| key.to_string())
                    .unwrap_or_default(),
                freeze_authority_pub_key: mint
                    .freeze_authority
                    .map(|key| key.to_string())
                    .unwrap_or_default(),
                decimals: u32::from(mint.decimals),
                supply: mint.supply.to_string(),
                is_initialized: mint.is_initialized,
            }),
            token_program: token_program.into(),
            extensions,
        }))
    }
}
