//! Token-2022 extension building and extraction.

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
    instruction::initialize_mint2,
    state::Mint,
};

use spl_token_metadata_interface::{
    instruction::{initialize as initialize_token_metadata, update_field},
    state::{Field, TokenMetadata},
};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension, Token2022ExtensionMetadata,
};

/// Extracts Token-2022 extensions from a parsed mint account and converts them
/// to proto `Token2022Extension` messages.
///
/// Currently supports:
///   - **Metadata**: reads `MetadataPointer` + `TokenMetadata` TLV data and
///     returns a `Token2022ExtensionMetadata` proto.
///
/// Extensions that are not present on the account are silently skipped.
pub(crate) fn extract_token2022_extensions(
    state: &StateWithExtensions<'_, Mint>,
    account_pubkey: &Pubkey,
) -> Vec<Token2022Extension> {
    let mut extensions = Vec::new();

    // Try to extract the Metadata extension (MetadataPointer + TokenMetadata).
    if let Ok(metadata_pointer) = state.get_extension::<MetadataPointer>() {
        let metadata_address: Option<Pubkey> = metadata_pointer.metadata_address.into();
        if let Some(metadata_addr) = metadata_address {
            // Only read the variable-length TokenMetadata if it is stored on
            // this mint account itself (self-referencing metadata).
            if metadata_addr == *account_pubkey {
                if let Ok(token_metadata) = state.get_variable_len_extension::<TokenMetadata>() {
                    let update_authority: Option<Pubkey> = token_metadata.update_authority.into();

                    extensions.push(Token2022Extension {
                        extension: Some(token2022_extension::Extension::Metadata(
                            Token2022ExtensionMetadata {
                                metadata_address: metadata_addr.to_string(),
                                update_authority_pub_key: update_authority
                                    .map(|k| k.to_string())
                                    .unwrap_or_default(),
                                name: token_metadata.name,
                                symbol: token_metadata.symbol,
                                uri: token_metadata.uri,
                                additional_metadata: token_metadata
                                    .additional_metadata
                                    .into_iter()
                                    .collect(),
                            },
                        )),
                    });
                }
            }
        }
    }

    extensions
}

/// Builds the ordered list of SDK instructions needed to initialise a Token-2022
/// mint with the requested extensions.
///
/// The instruction sequence for a mint with the Metadata extension is:
///   1. `initialize_metadata_pointer`  – must precede `initialize_mint`
///   2. `initialize_mint`
///   3. `initialize_token_metadata`    – must follow `initialize_mint`
///   4. `update_field` × N             – one per additional-metadata entry
///
/// For a plain mint (no extensions) only step 2 is emitted.
///
/// New extension types can be supported by adding arms to the pre/post match
/// blocks and collecting the relevant instructions.
#[allow(clippy::result_large_err)]
pub(crate) fn build_token2022_mint_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    extensions: &[Token2022Extension],
) -> Result<Vec<solana_sdk::instruction::Instruction>, Status> {
    // --- Phase 1: instructions that MUST run before initialize_mint ---
    let mut pre_init_instructions = Vec::new();
    // --- Phase 3: instructions that MUST run after initialize_mint ---
    let mut post_init_instructions = Vec::new();

    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(meta) => {
                // Resolve metadata_address: default to mint itself (self-referencing metadata)
                let metadata_address = if meta.metadata_address.is_empty() {
                    *mint_pubkey
                } else {
                    Pubkey::from_str(&meta.metadata_address).map_err(|e| {
                        Status::invalid_argument(format!("Invalid metadata_address: {e}"))
                    })?
                };

                // Resolve update_authority: default to mint_authority
                let update_authority = if meta.update_authority_pub_key.is_empty() {
                    *mint_authority
                } else {
                    Pubkey::from_str(&meta.update_authority_pub_key).map_err(|e| {
                        Status::invalid_argument(format!(
                            "Invalid metadata update_authority_pub_key: {e}"
                        ))
                    })?
                };

                // Pre-init: metadata pointer must be initialised before the mint
                pre_init_instructions.push(
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
                post_init_instructions.push(initialize_token_metadata(
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
                    post_init_instructions.push(update_field(
                        token_program_id,
                        &metadata_address,
                        &update_authority,
                        Field::Key(key.clone()),
                        value.clone(),
                    ));
                }
            }
        }
    }

    // --- Phase 2: initialize_mint itself ---
    let init_mint_instruction =
        initialize_mint2(token_program_id, mint_pubkey, mint_authority, freeze_authority, decimals)
            .map_err(|e| {
                Status::internal(format!("could not create initialise mint token instruction: {e}"))
            })?;

    // Assemble: pre-init → initialize_mint → post-init
    let mut instructions =
        Vec::with_capacity(pre_init_instructions.len() + 1 + post_init_instructions.len());
    instructions.append(&mut pre_init_instructions);
    instructions.push(init_mint_instruction);
    instructions.append(&mut post_init_instructions);

    Ok(instructions)
}
