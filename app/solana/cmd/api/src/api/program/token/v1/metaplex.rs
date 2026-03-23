//! Metaplex Token Metadata conversion and instruction building.

use std::str::FromStr;

use tonic::Status;

use mpl_token_metadata::instructions::CreateMetadataAccountV3Builder;
use mpl_token_metadata::types::{
    Collection as MplCollection, Creator as MplCreator, DataV2, UseMethod as MplUseMethod,
    Uses as MplUses,
};
use mpl_token_metadata::ID as METADATA_PROGRAM_ID;
use solana_sdk::pubkey::Pubkey;

use protochain_api::protochain::solana::program::token::v1::{
    metaplex_uses, MetaplexTokenMetadata,
};

/// Converts proto `MetaplexTokenMetadata` into the Metaplex SDK `DataV2` type.
#[allow(clippy::result_large_err)]
pub(crate) fn proto_metadata_to_data_v2(
    metadata: &MetaplexTokenMetadata,
) -> Result<DataV2, Status> {
    let creators = if metadata.creators.is_empty() {
        None
    } else {
        let mut creators = Vec::with_capacity(metadata.creators.len());
        for c in &metadata.creators {
            creators.push(MplCreator {
                address: Pubkey::from_str(&c.address).map_err(|e| {
                    Status::invalid_argument(format!("Invalid creator address: {e}"))
                })?,
                verified: c.verified,
                share: u8::try_from(c.share).map_err(|_| {
                    Status::invalid_argument("creator share must be between 0 and 100")
                })?,
            });
        }
        Some(creators)
    };

    let collection = metadata
        .collection
        .as_ref()
        .map(|c| {
            Ok::<_, Status>(MplCollection {
                verified: c.verified,
                key: Pubkey::from_str(&c.key).map_err(|e| {
                    Status::invalid_argument(format!("Invalid collection key: {e}"))
                })?,
            })
        })
        .transpose()?;

    let uses = metadata
        .uses
        .as_ref()
        .map(|u| {
            let use_method = match metaplex_uses::UseMethod::try_from(u.use_method) {
                Ok(metaplex_uses::UseMethod::Burn) => MplUseMethod::Burn,
                Ok(metaplex_uses::UseMethod::Multiple) => MplUseMethod::Multiple,
                Ok(metaplex_uses::UseMethod::Single) => MplUseMethod::Single,
                _ => {
                    return Err(Status::invalid_argument(
                        "use_method must be BURN, MULTIPLE, or SINGLE",
                    ))
                }
            };
            Ok::<_, Status>(MplUses {
                use_method,
                remaining: u.remaining,
                total: u.total,
            })
        })
        .transpose()?;

    Ok(DataV2 {
        name: metadata.name.clone(),
        symbol: metadata.symbol.clone(),
        uri: metadata.uri.clone(),
        seller_fee_basis_points: u16::try_from(metadata.seller_fee_basis_points).map_err(|_| {
            Status::invalid_argument("seller_fee_basis_points must fit in u16 (0–65535)")
        })?,
        creators,
        collection,
        uses,
    })
}

/// Builds a `CreateMetadataAccountV3` instruction for the Metaplex Token Metadata
/// program, which creates the on-chain metadata PDA for an SPL Token mint.
#[allow(clippy::result_large_err)]
pub(crate) fn build_create_metaplex_metadata_instruction(
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    metadata: &MetaplexTokenMetadata,
) -> Result<solana_sdk::instruction::Instruction, Status> {
    let (metadata_pda, _) = Pubkey::find_program_address(
        &[
            b"metadata",
            METADATA_PROGRAM_ID.as_ref(),
            mint_pubkey.as_ref(),
        ],
        &METADATA_PROGRAM_ID,
    );

    let data = proto_metadata_to_data_v2(metadata)?;

    Ok(CreateMetadataAccountV3Builder::new()
        .metadata(metadata_pda)
        .mint(*mint_pubkey)
        .mint_authority(*mint_authority)
        .payer(*payer)
        .update_authority(*mint_authority, true)
        .data(data)
        .is_mutable(true)
        .instruction())
}
