//! Token program conversion utilities
//!
//! This module provides conversion utilities for mapping between protobuf token program
//! definitions and Solana SDK token program IDs.

use protochain_api::protochain::solana::r#type::v1::TokenProgram;
use solana_sdk::pubkey::Pubkey;
use spl_token::id as spl_token_id;
use spl_token_2022::id as spl_token_2022_id;

/// Converts a protobuf `TokenProgram` enum to the corresponding Solana SDK program ID
///
/// Maps the protobuf token program definition to the actual Solana blockchain
/// program ID that should be used for token operations.
///
/// # Arguments
/// * `token_program` - The protobuf `TokenProgram` enum value
///
/// # Returns
/// * `Ok(Pubkey)` - The corresponding Solana token program ID
/// * `Err(String)` - Error if the token program is UNSPECIFIED or unknown
///
/// # Token Programs
/// - **Legacy**: SPL Token Program (Token-v1)
/// - **2022**: Token Extensions Program (Token-v2022)
pub fn proto_token_program_to_sdk(token_program: TokenProgram) -> Result<Pubkey, String> {
    match token_program {
        TokenProgram::Unspecified => {
            Err("TokenProgram must be specified (cannot be UNSPECIFIED)".to_string())
        }
        TokenProgram::Legacy => Ok(spl_token_id()),
        TokenProgram::TokenProgram2022 => Ok(spl_token_2022_id()),
    }
}

/// Converts a Solana SDK token program ID to the corresponding protobuf `TokenProgram`
///
/// Maps a Solana token program ID back to its protobuf representation.
///
/// # Arguments
/// * `program_id` - The Solana token program ID
///
/// # Returns
/// `TokenProgram::Unspecified` if the program ID doesn't match any known token programs
pub fn sdk_token_program_to_proto(program_id: &Pubkey) -> TokenProgram {
    if program_id == &spl_token_id() {
        TokenProgram::Legacy
    } else if program_id == &spl_token_2022_id() {
        TokenProgram::TokenProgram2022
    } else {
        TokenProgram::Unspecified
    }
}

/// Gets the Solana SDK token program ID for a protobuf `TokenProgram` enum value
///
/// A convenience wrapper around `proto_token_program_to_sdk` that unwraps the result.
/// This is useful when you need to directly obtain the program ID without error handling.
///
/// # Arguments
/// * `token_program` - The protobuf `TokenProgram` enum value
///
/// # Returns
/// The corresponding Solana token program ID
///
/// # Panics
/// Panics if the token program is UNSPECIFIED
pub fn get_token_program_id(token_program: TokenProgram) -> Pubkey {
    proto_token_program_to_sdk(token_program)
        .unwrap_or_else(|e| panic!("Failed to get token program ID: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_to_sdk_legacy() {
        let result = proto_token_program_to_sdk(TokenProgram::Legacy);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), spl_token_id());
    }

    #[test]
    fn test_proto_to_sdk_token_2022() {
        let result = proto_token_program_to_sdk(TokenProgram::TokenProgram2022);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), spl_token_2022_id());
    }

    #[test]
    fn test_proto_to_sdk_unspecified() {
        let result = proto_token_program_to_sdk(TokenProgram::Unspecified);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("TokenProgram must be specified"));
    }

    #[test]
    fn test_sdk_to_proto_legacy() {
        let result = sdk_token_program_to_proto(&spl_token_id());
        assert_eq!(result, TokenProgram::Legacy);
    }

    #[test]
    fn test_sdk_to_proto_token_2022() {
        let result = sdk_token_program_to_proto(&spl_token_2022_id());
        assert_eq!(result, TokenProgram::TokenProgram2022);
    }

    #[test]
    fn test_sdk_to_proto_unknown() {
        let program_id = Pubkey::new_unique();
        let result = sdk_token_program_to_proto(&program_id);
        assert_eq!(result, TokenProgram::Unspecified);
    }

    #[test]
    fn test_roundtrip_legacy() {
        let original = TokenProgram::Legacy;
        let program_id = proto_token_program_to_sdk(original).unwrap();
        let converted = sdk_token_program_to_proto(&program_id);
        assert_eq!(original, converted);
    }

    #[test]
    fn test_roundtrip_token_2022() {
        let original = TokenProgram::TokenProgram2022;
        let program_id = proto_token_program_to_sdk(original).unwrap();
        let converted = sdk_token_program_to_proto(&program_id);
        assert_eq!(original, converted);
    }

    #[test]
    fn test_get_token_program_id_legacy() {
        let program_id = get_token_program_id(TokenProgram::Legacy);
        assert_eq!(program_id, spl_token_id());
    }

    #[test]
    fn test_get_token_program_id_token_2022() {
        let program_id = get_token_program_id(TokenProgram::TokenProgram2022);
        assert_eq!(program_id, spl_token_2022_id());
    }

    #[test]
    #[should_panic(expected = "Failed to get token program ID")]
    fn test_get_token_program_id_unspecified_panics() {
        let _ = get_token_program_id(TokenProgram::Unspecified);
    }
}
