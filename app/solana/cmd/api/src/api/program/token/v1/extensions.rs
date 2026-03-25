//! Token-2022 extension building and extraction.
//!
//! Each supported extension lives in its own sub-module. The top-level helpers
//! here orchestrate across all extensions.

pub(crate) mod helpers;
mod metadata;

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{extension::StateWithExtensions, instruction::initialize_mint2, state::Mint};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension,
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

    if let Some(ext) = metadata::extract_metadata_extension(state, account_pubkey) {
        extensions.push(ext);
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
                let (mut pre, mut post) = metadata::build_metadata_mint_instructions(
                    token_program_id,
                    mint_pubkey,
                    mint_authority,
                    meta,
                )?;
                pre_init_instructions.append(&mut pre);
                post_init_instructions.append(&mut post);
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
