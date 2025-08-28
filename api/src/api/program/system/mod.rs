//! Solana System Program interface
//!
//! This module provides wrappers and utilities for interacting with the
//! Solana System Program, which handles core blockchain operations like
//! account creation, transfers, and space allocation.

/// System program service coordinator and aggregator
pub mod manager;
/// Version 1 of the System Program API implementation
pub mod v1;

pub use manager::System;
