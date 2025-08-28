//! Solana program interaction services
//!
//! This module provides interfaces for interacting with various Solana programs.
//! Currently supports the System Program with plans to expand to other programs.

/// Program services aggregator and coordinator
pub mod manager;
/// System program specific services and operations
pub mod system;

pub use manager::Program;
