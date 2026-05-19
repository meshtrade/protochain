//! Token-2022 **Immutable Owner** holding-account extension: instruction building.
//!
//! Note: ATAs created via the ATA program under Token-2022 automatically have
//! `ImmutableOwner` enabled. This module supports explicitly requesting the
//! extension when creating holding accounts.

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::instruction::initialize_immutable_owner;

/// Builds the post-ATA-creation instructions for the Immutable Owner extension.
///
/// Returns `(pre_init, post_init)` — only post-init is populated.
///
/// - **post-init**: `initialize_immutable_owner` (runs after ATA creation and
///   reallocate)
#[allow(clippy::result_large_err)]
pub(crate) fn build_immutable_owner_holding_account_instructions(
    token_program_id: &Pubkey,
    token_account: &Pubkey,
) -> Result<Vec<solana_sdk::instruction::Instruction>, Status> {
    let ix = initialize_immutable_owner(token_program_id, token_account).map_err(|e| {
        Status::internal(format!("could not create initialize_immutable_owner instruction: {e}"))
    })?;

    Ok(vec![ix])
}
