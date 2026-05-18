//! Token-2022 **Transfer Fee** extension: extraction and instruction building.

use std::str::FromStr;

use tonic::Status;

use solana_sdk::pubkey::Pubkey;
use spl_token_2022::{
    extension::{
        transfer_fee::{instruction::initialize_transfer_fee_config, TransferFeeConfig},
        BaseStateWithExtensions, StateWithExtensions,
    },
    state::Mint,
};

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, Token2022Extension, Token2022ExtensionTransferFee,
};

/// Maximum transfer fee basis points (100%).
const MAX_FEE_BASIS_POINTS: u32 = 10_000;

/// Extracts the Transfer Fee extension from a parsed Token-2022 mint account
/// and returns it as a `Token2022Extension` proto, or `None` if the extension
/// is not present.
pub(crate) fn extract_transfer_fee_extension(
    state: &StateWithExtensions<'_, Mint>,
) -> Option<Token2022Extension> {
    let ext = state.get_extension::<TransferFeeConfig>().ok()?;

    let config_authority: Option<Pubkey> = ext.transfer_fee_config_authority.into();
    let withdraw_authority: Option<Pubkey> = ext.withdraw_withheld_authority.into();

    // Use the newer_transfer_fee as the current configuration.
    let basis_points: u16 = ext.newer_transfer_fee.transfer_fee_basis_points.into();
    let max_fee: u64 = ext.newer_transfer_fee.maximum_fee.into();

    Some(Token2022Extension {
        extension: Some(token2022_extension::Extension::TransferFee(
            Token2022ExtensionTransferFee {
                transfer_fee_config_authority_pub_key: config_authority
                    .map(|k| k.to_string())
                    .unwrap_or_default(),
                withdraw_withheld_authority_pub_key: withdraw_authority
                    .map(|k| k.to_string())
                    .unwrap_or_default(),
                transfer_fee_basis_points: u32::from(basis_points),
                maximum_fee: max_fee,
            },
        )),
    })
}

/// Builds the pre-init instruction for the Transfer Fee extension.
///
/// Returns `(pre_init, post_init)` — only pre-init is populated.
///
/// - **pre-init**: `initialize_transfer_fee_config` (must precede `initialize_mint`)
#[allow(clippy::result_large_err)]
pub(crate) fn build_transfer_fee_instructions(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    config: &Token2022ExtensionTransferFee,
) -> Result<
    (
        Vec<solana_sdk::instruction::Instruction>,
        Vec<solana_sdk::instruction::Instruction>,
    ),
    Status,
> {
    if config.transfer_fee_basis_points > MAX_FEE_BASIS_POINTS {
        return Err(Status::invalid_argument(format!(
            "transfer_fee_basis_points must be <= {MAX_FEE_BASIS_POINTS}, got {}",
            config.transfer_fee_basis_points
        )));
    }

    let basis_points = u16::try_from(config.transfer_fee_basis_points)
        .map_err(|_| Status::invalid_argument("transfer_fee_basis_points must fit in a u16"))?;

    let config_authority = if config.transfer_fee_config_authority_pub_key.is_empty() {
        None
    } else {
        Some(Pubkey::from_str(&config.transfer_fee_config_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid transfer_fee_config_authority_pub_key: {e}"))
        })?)
    };

    let withdraw_authority = if config.withdraw_withheld_authority_pub_key.is_empty() {
        None
    } else {
        Some(Pubkey::from_str(&config.withdraw_withheld_authority_pub_key).map_err(|e| {
            Status::invalid_argument(format!("Invalid withdraw_withheld_authority_pub_key: {e}"))
        })?)
    };

    let ix = initialize_transfer_fee_config(
        token_program_id,
        mint_pubkey,
        config_authority.as_ref(),
        withdraw_authority.as_ref(),
        basis_points,
        config.maximum_fee,
    )
    .map_err(|e| {
        Status::internal(format!(
            "could not create initialize_transfer_fee_config instruction: {e}"
        ))
    })?;

    Ok((vec![ix], Vec::new()))
}
