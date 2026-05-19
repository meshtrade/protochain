//! Token-2022 **Mint Close Authority** extension: extraction and instruction building.

use std::str::FromStr;

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{
    extension::{
        mint_close_authority::MintCloseAuthority, BaseStateWithExtensions, StateWithExtensions,
    },
    instruction::initialize_mint_close_authority,
    state::Mint,
};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension, Token2022ExtensionMintCloseAuthority,
};

/// Extracts the Mint Close Authority extension from a parsed Token-2022 mint
/// account and returns it as a `Token2022Extension` proto, or `None` if the
/// extension is not present.
pub(crate) fn extract_mint_close_authority_extension(
    state: &StateWithExtensions<'_, Mint>,
) -> Option<Token2022Extension> {
    let ext = state.get_extension::<MintCloseAuthority>().ok()?;
    let close_authority: Option<Pubkey> = ext.close_authority.into();

    Some(Token2022Extension {
        extension: Some(token2022_extension::Extension::MintCloseAuthority(
            Token2022ExtensionMintCloseAuthority {
                close_authority_pub_key: close_authority.map(|k| k.to_string()).unwrap_or_default(),
            },
        )),
    })
}

/// Builds the pre-init instruction for the Mint Close Authority extension.
///
/// Returns `(pre_init, post_init)` — only pre-init is populated.
///
/// - **pre-init**: `initialize_mint_close_authority` (must precede `initialize_mint`)
#[allow(clippy::result_large_err)]
pub(crate) fn build_mint_close_authority_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    config: &Token2022ExtensionMintCloseAuthority,
) -> Result<
    (
        Vec<solana_sdk::instruction::Instruction>,
        Vec<solana_sdk::instruction::Instruction>,
    ),
    Status,
> {
    let close_authority = if config.close_authority_pub_key.is_empty() {
        *mint_authority
    } else {
        Pubkey::from_str(&config.close_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid close_authority_pub_key: {e}"))
        })?
    };

    let ix = initialize_mint_close_authority(token_program_id, mint_pubkey, Some(&close_authority))
        .map_err(|e| {
            Status::internal(format!(
                "could not create initialize_mint_close_authority instruction: {e}"
            ))
        })?;

    Ok((vec![ix], Vec::new()))
}
