//! Token-2022 **Pausable** extension: extraction and instruction building.

use std::str::FromStr;

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::extension::{
    pausable::{instruction::initialize as initialize_pausable, PausableConfig},
    BaseStateWithExtensions, StateWithExtensions,
};
use spl_token_2022::state::Mint;

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension, Token2022ExtensionPausable,
};

/// Extracts the Pausable extension from a parsed Token-2022 mint account and
/// returns it as a `Token2022Extension` proto, or `None` if the extension is
/// not present.
pub(crate) fn extract_pausable_extension(
    state: &StateWithExtensions<'_, Mint>,
) -> Option<Token2022Extension> {
    let ext = state.get_extension::<PausableConfig>().ok()?;
    let authority: Option<Pubkey> = ext.authority.into();

    Some(Token2022Extension {
        extension: Some(token2022_extension::Extension::Pausable(Token2022ExtensionPausable {
            authority_pub_key: authority.map(|k| k.to_string()).unwrap_or_default(),
        })),
    })
}

/// Builds the pre-init instruction for the Pausable extension.
///
/// Returns `(pre_init, post_init)` — only pre-init is populated.
///
/// - **pre-init**: `initialize_pausable` (must precede `initialize_mint`)
#[allow(clippy::result_large_err)]
pub(crate) fn build_pausable_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    config: &Token2022ExtensionPausable,
) -> Result<
    (
        Vec<solana_sdk::instruction::Instruction>,
        Vec<solana_sdk::instruction::Instruction>,
    ),
    Status,
> {
    let authority = if config.authority_pub_key.is_empty() {
        *mint_authority
    } else {
        Pubkey::from_str(&config.authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid pausable authority_pub_key: {e}"))
        })?
    };

    let ix = initialize_pausable(token_program_id, mint_pubkey, &authority).map_err(|e| {
        Status::internal(format!("could not create initialize_pausable instruction: {e}"))
    })?;

    Ok((vec![ix], Vec::new()))
}
