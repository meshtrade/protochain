// Pure validation functions - no external dependencies, fully unit testable

use protosol_api::protosol::solana::transaction::v1::{Transaction, TransactionState};

/// Validates that a state transition is allowed in the transaction lifecycle
pub fn validate_state_transition(
    from: TransactionState,
    to: TransactionState,
) -> Result<(), String> {
    match (from, to) {
        // Valid transitions from DRAFT
        (TransactionState::Draft, TransactionState::Compiled) => Ok(()),

        // Valid transitions from COMPILED
        (TransactionState::Compiled, TransactionState::PartiallySigned) => Ok(()),
        (TransactionState::Compiled, TransactionState::FullySigned) => Ok(()), // Direct to fully signed if all signers provided

        // Valid transitions from PARTIALLY_SIGNED
        (TransactionState::PartiallySigned, TransactionState::FullySigned) => Ok(()),

        // No transitions allowed from FULLY_SIGNED (terminal state)
        (TransactionState::FullySigned, _) => {
            Err("Cannot transition from FULLY_SIGNED state - it is terminal".to_string())
        }

        // No transitions allowed to DRAFT (would break immutability)
        (_, TransactionState::Draft) if from != TransactionState::Draft => {
            Err("Cannot transition back to DRAFT state - would break immutability".to_string())
        }

        // No backward transitions (enforce progression)
        (TransactionState::PartiallySigned, TransactionState::Compiled) => {
            Err("Cannot transition backward from PARTIALLY_SIGNED to COMPILED".to_string())
        }

        // Handle unspecified state
        (TransactionState::Unspecified, _) => {
            Err("Cannot transition from UNSPECIFIED state".to_string())
        }
        (_, TransactionState::Unspecified) => {
            Err("Cannot transition to UNSPECIFIED state".to_string())
        }

        // All other combinations are invalid
        _ => Err(format!("Invalid state transition from {from:?} to {to:?}")),
    }
}

/// Validates that a transaction's fields are consistent with its current state
pub fn validate_transaction_state_consistency(transaction: &Transaction) -> Result<(), String> {
    match transaction.state() {
        TransactionState::Draft => {
            // DRAFT: Must have instructions, no compiled data, no signatures
            if transaction.instructions.is_empty() {
                return Err("DRAFT transaction must have at least one instruction".to_string());
            }
            if !transaction.data.is_empty() {
                return Err("DRAFT transaction must not have compiled data".to_string());
            }
            if !transaction.signatures.is_empty() {
                return Err("DRAFT transaction must not have signatures".to_string());
            }
        }

        TransactionState::Compiled => {
            // COMPILED: Must have instructions and compiled data, no signatures
            if transaction.instructions.is_empty() {
                return Err("COMPILED transaction must have instructions".to_string());
            }
            if transaction.data.is_empty() {
                return Err("COMPILED transaction must have compiled data".to_string());
            }
            if !transaction.signatures.is_empty() {
                return Err("COMPILED transaction must not have signatures yet".to_string());
            }
            if transaction.recent_blockhash.is_empty() {
                return Err("COMPILED transaction must have recent_blockhash".to_string());
            }
            if transaction.fee_payer.is_empty() {
                return Err("COMPILED transaction must have fee_payer".to_string());
            }
        }

        TransactionState::PartiallySigned => {
            // PARTIALLY_SIGNED: Must have compiled data and some signatures
            if transaction.data.is_empty() {
                return Err("PARTIALLY_SIGNED transaction must have compiled data".to_string());
            }
            if transaction.signatures.is_empty() {
                return Err(
                    "PARTIALLY_SIGNED transaction must have at least one signature".to_string()
                );
            }
            if transaction.recent_blockhash.is_empty() {
                return Err("PARTIALLY_SIGNED transaction must have recent_blockhash".to_string());
            }
            if transaction.fee_payer.is_empty() {
                return Err("PARTIALLY_SIGNED transaction must have fee_payer".to_string());
            }
        }

        TransactionState::FullySigned => {
            // FULLY_SIGNED: Must have compiled data and all required signatures
            if transaction.data.is_empty() {
                return Err("FULLY_SIGNED transaction must have compiled data".to_string());
            }
            if transaction.signatures.is_empty() {
                return Err("FULLY_SIGNED transaction must have signatures".to_string());
            }
            if transaction.recent_blockhash.is_empty() {
                return Err("FULLY_SIGNED transaction must have recent_blockhash".to_string());
            }
            if transaction.fee_payer.is_empty() {
                return Err("FULLY_SIGNED transaction must have fee_payer".to_string());
            }
        }

        TransactionState::Unspecified => {
            return Err("Transaction state cannot be UNSPECIFIED".to_string());
        }
    }

    Ok(())
}

/// Validates that a given operation is allowed for the current transaction state
pub fn validate_operation_allowed_for_state(
    state: TransactionState,
    operation: &str,
) -> Result<(), String> {
    match (state, operation) {
        // DRAFT state operations
        (TransactionState::Draft, "compile") => Ok(()),
        (TransactionState::Draft, "add_instruction") => Ok(()), // Hypothetical future operation
        (TransactionState::Draft, "remove_instruction") => Ok(()), // Hypothetical future operation

        // COMPILED state operations
        (TransactionState::Compiled, "sign") => Ok(()),
        (TransactionState::Compiled, "estimate") => Ok(()),
        (TransactionState::Compiled, "simulate") => Ok(()),

        // PARTIALLY_SIGNED state operations
        (TransactionState::PartiallySigned, "sign") => Ok(()), // Add more signatures
        (TransactionState::PartiallySigned, "estimate") => Ok(()),
        (TransactionState::PartiallySigned, "simulate") => Ok(()),

        // FULLY_SIGNED state operations
        (TransactionState::FullySigned, "submit") => Ok(()),
        (TransactionState::FullySigned, "estimate") => Ok(()), // Still valid for fee estimation
        (TransactionState::FullySigned, "simulate") => Ok(()), // Still valid for testing

        // No operations allowed for UNSPECIFIED
        (TransactionState::Unspecified, _) => {
            Err("No operations allowed for UNSPECIFIED state".to_string())
        }

        // Invalid operation for current state
        _ => Err(format!("Operation '{operation}' not allowed for transaction state {state:?}")),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // unwrap is acceptable in tests for cleaner assertions
mod tests {
    use super::*;
    use protosol_api::protosol::solana::transaction::v1::*;

    #[test]
    fn test_valid_state_transitions() {
        // Valid forward transitions
        assert!(
            validate_state_transition(TransactionState::Draft, TransactionState::Compiled).is_ok()
        );
        assert!(validate_state_transition(
            TransactionState::Compiled,
            TransactionState::PartiallySigned
        )
        .is_ok());
        assert!(validate_state_transition(
            TransactionState::Compiled,
            TransactionState::FullySigned
        )
        .is_ok()); // Direct
        assert!(validate_state_transition(
            TransactionState::PartiallySigned,
            TransactionState::FullySigned
        )
        .is_ok());

        // Valid self-transitions (for adding more signatures)
        assert!(validate_state_transition(
            TransactionState::PartiallySigned,
            TransactionState::PartiallySigned
        )
        .is_ok());
    }

    #[test]
    fn test_invalid_state_transitions() {
        // Cannot go backward to DRAFT
        assert!(
            validate_state_transition(TransactionState::Compiled, TransactionState::Draft).is_err()
        );
        assert!(validate_state_transition(
            TransactionState::PartiallySigned,
            TransactionState::Draft
        )
        .is_err());
        assert!(
            validate_state_transition(TransactionState::FullySigned, TransactionState::Draft)
                .is_err()
        );

        // Cannot go backward from PARTIALLY_SIGNED to COMPILED
        assert!(validate_state_transition(
            TransactionState::PartiallySigned,
            TransactionState::Compiled
        )
        .is_err());

        // Cannot transition from FULLY_SIGNED (terminal state)
        assert!(validate_state_transition(
            TransactionState::FullySigned,
            TransactionState::Compiled
        )
        .is_err());
        assert!(validate_state_transition(
            TransactionState::FullySigned,
            TransactionState::PartiallySigned
        )
        .is_err());

        // Cannot transition from/to UNSPECIFIED
        assert!(
            validate_state_transition(TransactionState::Unspecified, TransactionState::Draft)
                .is_err()
        );
        assert!(
            validate_state_transition(TransactionState::Draft, TransactionState::Unspecified)
                .is_err()
        );
    }

    #[test]
    fn test_transaction_state_consistency_draft() {
        // Valid DRAFT transaction
        let valid_draft = Transaction {
            instructions: vec![SolanaInstruction::default()], // Has instructions
            state: TransactionState::Draft.into(),
            config: None,
            data: String::new(), // No data
            fee_payer: String::new(),
            recent_blockhash: String::new(),
            signatures: vec![], // No signatures
            hash: String::new(),
            signature: String::new(),
        };
        assert!(validate_transaction_state_consistency(&valid_draft).is_ok());

        // Invalid DRAFT - no instructions
        let invalid_draft_no_instructions = Transaction {
            instructions: vec![], // Empty instructions - invalid
            state: TransactionState::Draft.into(),
            config: None,
            data: String::new(),
            fee_payer: String::new(),
            recent_blockhash: String::new(),
            signatures: vec![],
            hash: String::new(),
            signature: String::new(),
        };
        assert!(validate_transaction_state_consistency(&invalid_draft_no_instructions).is_err());

        // Invalid DRAFT - has compiled data
        let invalid_draft_has_data = Transaction {
            instructions: vec![SolanaInstruction::default()],
            state: TransactionState::Draft.into(),
            config: None,
            data: "compiled data".to_string(), // Should be empty
            fee_payer: String::new(),
            recent_blockhash: String::new(),
            signatures: vec![],
            hash: String::new(),
            signature: String::new(),
        };
        assert!(validate_transaction_state_consistency(&invalid_draft_has_data).is_err());
    }

    #[test]
    fn test_transaction_state_consistency_compiled() {
        // Valid COMPILED transaction
        let valid_compiled = Transaction {
            instructions: vec![SolanaInstruction::default()],
            state: TransactionState::Compiled.into(),
            config: None,
            data: "compiled transaction data".to_string(),
            fee_payer: "5ByGMvVKHAw2pABUg8jz35hLcFuiqXWkGkqQ9aaC1mQX".to_string(),
            recent_blockhash: "BKxyMTxUBEzajVU5JnGXfpFYuL7GUjHwKN8mQjzPZRHD".to_string(),
            signatures: vec![], // No signatures yet
            hash: String::new(),
            signature: String::new(),
        };
        assert!(validate_transaction_state_consistency(&valid_compiled).is_ok());

        // Invalid COMPILED - no data
        let invalid_compiled_no_data = Transaction {
            instructions: vec![SolanaInstruction::default()],
            state: TransactionState::Compiled.into(),
            config: None,
            data: String::new(), // Should have data
            fee_payer: "5ByGMvVKHAw2pABUg8jz35hLcFuiqXWkGkqQ9aaC1mQX".to_string(),
            recent_blockhash: "BKxyMTxUBEzajVU5JnGXfpFYuL7GUjHwKN8mQjzPZRHD".to_string(),
            signatures: vec![],
            hash: String::new(),
            signature: String::new(),
        };
        assert!(validate_transaction_state_consistency(&invalid_compiled_no_data).is_err());
    }

    #[test]
    fn test_operation_permissions() {
        // DRAFT operations
        assert!(validate_operation_allowed_for_state(TransactionState::Draft, "compile").is_ok());
        assert!(validate_operation_allowed_for_state(TransactionState::Draft, "sign").is_err());
        assert!(validate_operation_allowed_for_state(TransactionState::Draft, "submit").is_err());

        // COMPILED operations
        assert!(validate_operation_allowed_for_state(TransactionState::Compiled, "sign").is_ok());
        assert!(
            validate_operation_allowed_for_state(TransactionState::Compiled, "estimate").is_ok()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::Compiled, "simulate").is_ok()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::Compiled, "compile").is_err()
        );
        assert!(validate_operation_allowed_for_state(TransactionState::Compiled, "submit").is_err());

        // PARTIALLY_SIGNED operations
        assert!(
            validate_operation_allowed_for_state(TransactionState::PartiallySigned, "sign").is_ok()
        ); // Add more sigs
        assert!(validate_operation_allowed_for_state(
            TransactionState::PartiallySigned,
            "estimate"
        )
        .is_ok());
        assert!(
            validate_operation_allowed_for_state(TransactionState::PartiallySigned, "compile")
                .is_err()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::PartiallySigned, "submit")
                .is_err()
        );

        // FULLY_SIGNED operations
        assert!(
            validate_operation_allowed_for_state(TransactionState::FullySigned, "submit").is_ok()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::FullySigned, "estimate").is_ok()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::FullySigned, "simulate").is_ok()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::FullySigned, "compile").is_err()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::FullySigned, "sign").is_err()
        );

        // UNSPECIFIED - no operations allowed
        assert!(
            validate_operation_allowed_for_state(TransactionState::Unspecified, "compile").is_err()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::Unspecified, "sign").is_err()
        );
        assert!(
            validate_operation_allowed_for_state(TransactionState::Unspecified, "submit").is_err()
        );
    }

    #[test]
    fn test_edge_cases() {
        // Invalid operation names
        assert!(
            validate_operation_allowed_for_state(TransactionState::Draft, "invalid_operation")
                .is_err()
        );
        assert!(validate_operation_allowed_for_state(TransactionState::Compiled, "").is_err());

        // Test error messages are descriptive
        let result =
            validate_state_transition(TransactionState::FullySigned, TransactionState::Draft);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("terminal"));

        let result = validate_operation_allowed_for_state(TransactionState::Draft, "submit");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }
}
