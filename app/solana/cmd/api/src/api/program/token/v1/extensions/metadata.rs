//! Token-2022 **Metadata** extension: extraction and instruction building.

use std::str::FromStr;

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{
    extension::{
        metadata_pointer::{
            instruction::initialize as initialize_metadata_pointer, MetadataPointer,
        },
        BaseStateWithExtensions, StateWithExtensions,
    },
    state::Mint,
};

use spl_token_metadata_interface::{
    instruction::{initialize as initialize_token_metadata, update_field},
    state::{Field, TokenMetadata},
};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension, Token2022ExtensionMetadata,
};

/// Extracts the Metadata extension (`MetadataPointer` + `TokenMetadata`) from a
/// parsed Token-2022 mint account and returns it as a `Token2022Extension`
/// proto, or `None` if the extension is not present.
pub(crate) fn extract_metadata_extension(
    state: &StateWithExtensions<'_, Mint>,
    account_pubkey: &Pubkey,
) -> Option<Token2022Extension> {
    let metadata_pointer = state.get_extension::<MetadataPointer>().ok()?;
    let metadata_address: Option<Pubkey> = metadata_pointer.metadata_address.into();
    let metadata_addr = metadata_address?;

    // Only read the variable-length TokenMetadata if it is stored on
    // this mint account itself (self-referencing metadata).
    if metadata_addr != *account_pubkey {
        return None;
    }

    let token_metadata = state.get_variable_len_extension::<TokenMetadata>().ok()?;
    let update_authority: Option<Pubkey> = token_metadata.update_authority.into();

    Some(Token2022Extension {
        extension: Some(token2022_extension::Extension::Metadata(Token2022ExtensionMetadata {
            metadata_address: metadata_addr.to_string(),
            update_authority_pub_key: update_authority.map(|k| k.to_string()).unwrap_or_default(),
            name: token_metadata.name,
            symbol: token_metadata.symbol,
            uri: token_metadata.uri,
            additional_metadata: token_metadata.additional_metadata.into_iter().collect(),
        })),
    })
}

/// Builds the pre-init and post-init instructions for the Metadata extension.
///
/// Returns `(pre_init, post_init)` instruction vectors.
///
/// - **pre-init**: `initialize_metadata_pointer` (must precede `initialize_mint`)
/// - **post-init**: `initialize_token_metadata` + `update_field` × N (must follow `initialize_mint`)
#[allow(clippy::result_large_err)]
pub(crate) fn build_metadata_mint_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    meta: &Token2022ExtensionMetadata,
) -> Result<
    (
        Vec<solana_sdk::instruction::Instruction>,
        Vec<solana_sdk::instruction::Instruction>,
    ),
    Status,
> {
    let mut pre_init = Vec::new();
    let mut post_init = Vec::new();

    // Resolve metadata_address: default to mint itself (self-referencing metadata)
    let metadata_address = if meta.metadata_address.is_empty() {
        *mint_pubkey
    } else {
        Pubkey::from_str(&meta.metadata_address)
            .map_err(|e| Status::invalid_argument(format!("Invalid metadata_address: {e}")))?
    };

    // Resolve update_authority: default to mint_authority
    let update_authority = if meta.update_authority_pub_key.is_empty() {
        *mint_authority
    } else {
        Pubkey::from_str(&meta.update_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid metadata update_authority_pub_key: {e}"))
        })?
    };

    // Pre-init: metadata pointer must be initialised before the mint
    pre_init.push(
        initialize_metadata_pointer(
            token_program_id,
            mint_pubkey,
            Some(update_authority),
            Some(metadata_address),
        )
        .map_err(|e| {
            Status::internal(format!(
                "could not create initialize_metadata_pointer instruction: {e}"
            ))
        })?,
    );

    // Post-init: token metadata must be initialised after the mint
    post_init.push(initialize_token_metadata(
        token_program_id,
        &metadata_address,
        &update_authority,
        mint_pubkey,
        mint_authority,
        meta.name.clone(),
        meta.symbol.clone(),
        meta.uri.clone(),
    ));

    // Post-init: additional metadata fields
    for (key, value) in &meta.additional_metadata {
        post_init.push(update_field(
            token_program_id,
            &metadata_address,
            &update_authority,
            Field::Key(key.clone()),
            value.clone(),
        ));
    }

    Ok((pre_init, post_init))
}
