//! Handler implementation for `ParseMint`.

use std::str::FromStr;

use tonic::{Request, Response, Status};

use mpl_token_metadata::accounts::Metadata as MetaplexMetadataAccount;
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;

use protochain_api::protochain::solana::program::token::v1::{
    metaplex_uses, MetaplexCollection, MetaplexCreator, MetaplexTokenMetadata, MetaplexUses,
    MintInfo, ParseMintRequest, ParseMintResponse,
};

use solana_commitment_config::CommitmentConfig;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::ID as SPL_TOKEN_PROGRAM_ID;
use spl_token_2022::{extension::StateWithExtensions, state::Mint, ID as TOKEN_2022_PROGRAM_ID};

use crate::api::program::token::v1::extensions::extract_token2022_extensions;
use crate::api::program::token::v1::helpers::sdk_token_program_to_proto;

use super::TokenProgramServiceImpl;

/// Trims trailing null bytes and whitespace that Metaplex pads fixed-length
/// string fields with.
fn trim_metaplex_string(s: &str) -> String {
    s.trim_end_matches('\0').trim().to_string()
}

/// Converts a deserialized Metaplex `Metadata` account into the proto
/// `MetaplexTokenMetadata` message.
fn metaplex_metadata_to_proto(metadata: &MetaplexMetadataAccount) -> MetaplexTokenMetadata {
    let creators = metadata
        .creators
        .as_ref()
        .map(|creators| {
            creators
                .iter()
                .map(|c| MetaplexCreator {
                    address: c.address.to_string(),
                    verified: c.verified,
                    share: u32::from(c.share),
                })
                .collect()
        })
        .unwrap_or_default();

    let collection = metadata.collection.as_ref().map(|c| MetaplexCollection {
        verified: c.verified,
        key: c.key.to_string(),
    });

    let uses = metadata.uses.as_ref().map(|u| {
        let use_method = match u.use_method {
            mpl_token_metadata::types::UseMethod::Burn => metaplex_uses::UseMethod::Burn as i32,
            mpl_token_metadata::types::UseMethod::Multiple => {
                metaplex_uses::UseMethod::Multiple as i32
            }
            mpl_token_metadata::types::UseMethod::Single => metaplex_uses::UseMethod::Single as i32,
        };
        MetaplexUses {
            use_method,
            remaining: u.remaining,
            total: u.total,
        }
    });

    MetaplexTokenMetadata {
        name: trim_metaplex_string(&metadata.name),
        symbol: trim_metaplex_string(&metadata.symbol),
        uri: trim_metaplex_string(&metadata.uri),
        seller_fee_basis_points: u32::from(metadata.seller_fee_basis_points),
        creators,
        collection,
        uses,
    }
}

impl TokenProgramServiceImpl {
    /// Parses mint account data into structured format.
    ///
    /// Supports both Legacy SPL Token and Token-2022 mints. The account owner is
    /// checked to ensure it belongs to a known token program, and the appropriate
    /// unpacking strategy is used (Token-2022 mints may contain extension data
    /// beyond the base 82-byte Mint layout).
    ///
    /// For legacy SPL Token mints, the standard Metaplex metadata PDA is derived
    /// and fetched. If it exists, the metadata is returned in the
    /// `metaplex_metadata` field.
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
            .await
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
        let (mint, extensions, metaplex_metadata) = if account.owner == TOKEN_2022_PROGRAM_ID {
            let state = StateWithExtensions::<Mint>::unpack(&account.data).map_err(|e| {
                Status::invalid_argument(format!("Failed to parse Token-2022 mint: {e}"))
            })?;

            let extensions = extract_token2022_extensions(&state, &account_pubkey);
            (state.base, extensions, None)
        } else {
            let mint = Mint::unpack(&account.data).map_err(|e| {
                Status::invalid_argument(format!("Failed to parse mint account: {e}"))
            })?;

            // For SPL mints, attempt to fetch the associated Metaplex metadata PDA.
            let metaplex_metadata = self.fetch_metaplex_metadata(&account_pubkey).await;

            (mint, Vec::new(), metaplex_metadata)
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
            metaplex_metadata,
        }))
    }

    /// Derives the Metaplex metadata PDA for the given mint and attempts to
    /// fetch and deserialize it. Returns `None` if the account does not exist
    /// or cannot be deserialized (i.e. no metadata was ever created).
    async fn fetch_metaplex_metadata(&self, mint_pubkey: &Pubkey) -> Option<MetaplexTokenMetadata> {
        // Derive the metadata PDA using the standard Metaplex seed convention.
        let (metadata_pda, _) = Pubkey::find_program_address(
            &[
                b"metadata",
                METADATA_PROGRAM_ID.as_ref(),
                mint_pubkey.as_ref(),
            ],
            &METADATA_PROGRAM_ID,
        );

        // Attempt to fetch the account — if it doesn't exist, return None.
        let account = self
            .rpc_client
            .get_account_with_commitment(&metadata_pda, CommitmentConfig::confirmed())
            .await
            .ok()?
            .value?;

        // Verify the account is owned by the Metaplex Token Metadata program.
        if account.owner != METADATA_PROGRAM_ID {
            return None;
        }

        // Deserialize the metadata account data.
        let metadata = MetaplexMetadataAccount::safe_deserialize(&account.data).ok()?;

        Some(metaplex_metadata_to_proto(&metadata))
    }
}
