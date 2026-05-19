//! Token-2022 **Permanent Delegate** extension: extraction and instruction building.

use std::str::FromStr;

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{
    extension::{
        permanent_delegate::PermanentDelegate, BaseStateWithExtensions, StateWithExtensions,
    },
    instruction::initialize_permanent_delegate,
    state::Mint,
};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension, Token2022ExtensionPermanentDelegate,
};

/// Extracts the Permanent Delegate extension from a parsed Token-2022 mint
/// account and returns it as a `Token2022Extension` proto, or `None` if the
/// extension is not present.
pub(crate) fn extract_permanent_delegate_extension(
    state: &StateWithExtensions<'_, Mint>,
) -> Option<Token2022Extension> {
    let ext = state.get_extension::<PermanentDelegate>().ok()?;
    let delegate: Option<Pubkey> = ext.delegate.into();

    Some(Token2022Extension {
        extension: Some(token2022_extension::Extension::PermanentDelegate(
            Token2022ExtensionPermanentDelegate {
                delegate_pub_key: delegate.map(|k| k.to_string()).unwrap_or_default(),
            },
        )),
    })
}

/// Builds the pre-init instruction for the Permanent Delegate extension.
///
/// Returns `(pre_init, post_init)` — only pre-init is populated.
///
/// - **pre-init**: `initialize_permanent_delegate` (must precede `initialize_mint`)
#[allow(clippy::result_large_err)]
pub(crate) fn build_permanent_delegate_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    config: &Token2022ExtensionPermanentDelegate,
) -> Result<
    (
        Vec<solana_sdk::instruction::Instruction>,
        Vec<solana_sdk::instruction::Instruction>,
    ),
    Status,
> {
    if config.delegate_pub_key.is_empty() {
        return Err(Status::invalid_argument("permanent_delegate.delegate_pub_key is required"));
    }

    let delegate = Pubkey::from_str(&config.delegate_pub_key)
        .map_err(|e| Status::invalid_argument(format!("Invalid delegate_pub_key: {e}")))?;

    let ix =
        initialize_permanent_delegate(token_program_id, mint_pubkey, &delegate).map_err(|e| {
            Status::internal(format!(
                "could not create initialize_permanent_delegate instruction: {e}"
            ))
        })?;

    Ok((vec![ix], Vec::new()))
}
