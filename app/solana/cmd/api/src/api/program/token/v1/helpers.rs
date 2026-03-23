//! Validation and conversion helpers for the Token Program service.

use std::collections::HashSet;
use std::str::FromStr;

use tonic::Status;

use protochain_api::protochain::solana::program::token::v1::{
    token2022_extension, token2022_holding_account_extension, Token2022Extension,
    Token2022HoldingAccountExtension,
};
use spl_token_2022::{
    extension::ExtensionType,
    state::{Account, Mint},
};

use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_metadata_interface::state::TokenMetadata;

use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;

// ---------------------------------------------------------------------------
//  Extension validation
// ---------------------------------------------------------------------------

/// Validates that the given extension list contains no duplicates.
#[allow(clippy::result_large_err)]
pub(crate) fn validate_no_duplicate_extensions(
    extensions: &[Token2022Extension],
) -> Result<(), Status> {
    let mut seen: HashSet<&str> = HashSet::new();
    for ext in extensions {
        let key = match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(_) => "Metadata",
        };
        if !seen.insert(key) {
            return Err(Status::invalid_argument(format!("Duplicate extension: {key}")));
        }
    }
    Ok(())
}

/// Validates that the given holding account extension list contains no duplicates.
#[allow(clippy::result_large_err)]
pub(crate) fn validate_no_duplicate_holding_account_extensions(
    extensions: &[Token2022HoldingAccountExtension],
) -> Result<(), Status> {
    let mut seen: HashSet<&str> = HashSet::new();
    for ext in extensions {
        let key = match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_holding_account_extension::Extension::MemoTransfer(_) => "MemoTransfer",
        };
        if !seen.insert(key) {
            return Err(Status::invalid_argument(format!(
                "Duplicate holding account extension: {key}"
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  Decimal validation
// ---------------------------------------------------------------------------

/// Maximum number of decimal places supported for Solana token mints.
///
/// SOL itself uses 9 decimals; USDC-like tokens typically use 6; NFTs use 0.
/// Values above 9 are not used in practice and can cause precision issues in
/// UIs and off-chain tooling.
const MAX_TOKEN_DECIMALS: u8 = 9;

/// Parses and validates a token decimal value from a proto `uint32` field.
///
/// Returns a `u8` in the range `0..=9` or an `INVALID_ARGUMENT` status.
#[allow(clippy::result_large_err)]
pub(crate) fn validate_decimals(value: u32) -> Result<u8, Status> {
    let decimals =
        u8::try_from(value).map_err(|_| Status::invalid_argument("decimals must fit in a u8"))?;
    if decimals > MAX_TOKEN_DECIMALS {
        return Err(Status::invalid_argument(format!(
            "decimals must be between 0 and {MAX_TOKEN_DECIMALS}, got {decimals}"
        )));
    }
    Ok(decimals)
}

// ---------------------------------------------------------------------------
//  Human-readable amount parsing
// ---------------------------------------------------------------------------

/// Parses a human-readable decimal amount string (e.g. "1.5") into the base-unit
/// `u64` representation using the given number of decimals.
///
/// Uses `rust_decimal` for robust decimal arithmetic — the same approach as
/// shopspring/decimal in the Go ecosystem.
///
/// Examples with `decimals = 6`:
///   "1.0"      → `1_000_000`
///   "0.5"      → `500_000`
///   "1000"     → `1_000_000_000`
///   "0.000001" → 1
///
/// Returns `INVALID_ARGUMENT` if the string is malformed, has too many decimal
/// places, or the resulting value overflows `u64`.
#[allow(clippy::result_large_err)]
pub(crate) fn parse_human_amount(amount_str: &str, decimals: u8) -> Result<u64, Status> {
    use rust_decimal::Decimal;

    let amount = Decimal::from_str(amount_str)
        .map_err(|e| Status::invalid_argument(format!("Invalid amount '{amount_str}': {e}")))?;

    if amount.is_sign_negative() {
        return Err(Status::invalid_argument("amount must not be negative"));
    }

    // Truncate the amount to the mint's decimal precision and verify it is
    // identical to the original.  This catches cases like "1.2345" when the
    // mint only supports 2 decimals — we refuse to silently truncate to "1.23".
    let truncated = amount.round_dp(u32::from(decimals));
    if truncated != amount {
        return Err(Status::invalid_argument(format!(
            "Amount '{amount_str}' has more fractional digits than the mint supports ({decimals} decimals); \
             refusing to silently truncate to {truncated}",
        )));
    }

    // Multiply by 10^decimals to convert to base units.
    let multiplier = Decimal::from(10u64.pow(u32::from(decimals)));
    let base_units = amount.checked_mul(multiplier).ok_or_else(|| {
        Status::invalid_argument("Amount overflows when converting to base units")
    })?;

    // The result must be a whole number at this point (we verified no
    // precision was lost above).
    base_units
        .try_into()
        .map_err(|e| Status::invalid_argument(format!("Amount overflows u64: {e}")))
}

// ---------------------------------------------------------------------------
//  Space / rent calculation
// ---------------------------------------------------------------------------

/// Calculates the space (in bytes) to allocate when creating a Token-2022 mint
/// account via `System::CreateAccount`.
///
/// This includes the base mint layout and fixed-size extension type pods
/// (e.g. `MetadataPointer`), but **not** variable-length content like
/// `TokenMetadata` which the Token-2022 program allocates internally via
/// `realloc` when `initialize_token_metadata` is called.
///
/// Returns `Mint::LEN` when no extensions are provided.
#[allow(clippy::result_large_err)]
pub(crate) fn mint_create_account_space(
    extensions: &[Token2022Extension],
) -> Result<usize, Status> {
    if extensions.is_empty() {
        return Ok(Mint::LEN);
    }

    let mut sdk_extension_types: Vec<ExtensionType> = Vec::new();
    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(_) => {
                sdk_extension_types.push(ExtensionType::MetadataPointer);
            }
        }
    }

    ExtensionType::try_calculate_account_len::<Mint>(&sdk_extension_types).map_err(|e| {
        Status::internal(format!("failed to calculate mint account length for extensions: {e}"))
    })
}

/// Calculates the total space a Token-2022 mint account will occupy after
/// **all** extensions — including variable-length metadata content — have been
/// fully initialised.
///
/// This is used to determine the rent-exempt lamport deposit at account
/// creation. The Token-2022 program resizes the account via `realloc` when
/// metadata is written, so the account must be pre-funded with enough lamports
/// for the final size even though `mint_create_account_space` returns a smaller
/// allocation.
///
/// Returns `Mint::LEN` when no extensions are provided.
#[allow(clippy::result_large_err)]
pub(crate) fn mint_total_space_for_rent(
    extensions: &[Token2022Extension],
) -> Result<usize, Status> {
    if extensions.is_empty() {
        return Ok(Mint::LEN);
    }

    let mut sdk_extension_types: Vec<ExtensionType> = Vec::new();
    let mut extra_variable_len: usize = 0;

    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_extension::Extension::Metadata(meta) => {
                sdk_extension_types.push(ExtensionType::MetadataPointer);

                let token_metadata = TokenMetadata {
                    update_authority: OptionalNonZeroPubkey::default(),
                    mint: Pubkey::default(),
                    name: meta.name.clone(),
                    symbol: meta.symbol.clone(),
                    uri: meta.uri.clone(),
                    additional_metadata: meta
                        .additional_metadata
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                };

                extra_variable_len += token_metadata.tlv_size_of().map_err(|e| {
                    Status::internal(format!("failed to calculate metadata TLV size: {e}"))
                })?;
            }
        }
    }

    let base_space = ExtensionType::try_calculate_account_len::<Mint>(&sdk_extension_types)
        .map_err(|e| {
            Status::internal(format!("failed to calculate mint account length for extensions: {e}"))
        })?;

    Ok(base_space + extra_variable_len)
}

/// Collects the SDK `ExtensionType` variants requested by the holding account
/// extensions, used for calculating account size and building reallocate
/// instructions.
#[allow(clippy::result_large_err)]
pub(crate) fn holding_account_extension_types(
    extensions: &[Token2022HoldingAccountExtension],
) -> Result<Vec<ExtensionType>, Status> {
    let mut types = Vec::with_capacity(extensions.len());
    for ext in extensions {
        match ext
            .extension
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("Extension must have a type set"))?
        {
            token2022_holding_account_extension::Extension::MemoTransfer(_) => {
                types.push(ExtensionType::MemoTransfer);
            }
        }
    }
    Ok(types)
}

/// Calculates the total account size for a Token-2022 holding account with the
/// given extensions.
///
/// Returns `Account::LEN` when no extensions are provided.
#[allow(clippy::result_large_err)]
pub(crate) fn holding_account_total_space(
    extensions: &[Token2022HoldingAccountExtension],
) -> Result<usize, Status> {
    if extensions.is_empty() {
        return Ok(Account::LEN);
    }

    let sdk_types = holding_account_extension_types(extensions)?;
    ExtensionType::try_calculate_account_len::<Account>(&sdk_types).map_err(|e| {
        Status::internal(format!("failed to calculate holding account length for extensions: {e}"))
    })
}
