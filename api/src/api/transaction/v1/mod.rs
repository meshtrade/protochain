//! Transaction API v1 implementation
//!
//! This module contains the version 1 implementation of the Transaction API,
//! including state machine validation, service implementation, and gRPC wrappers.

/// Core business logic implementation for transaction operations
pub mod service_impl;
/// gRPC service wrapper for Transaction v1 API
pub mod transaction_v1_api;
/// Transaction state machine validation utilities
pub mod validation;

pub use service_impl::TransactionServiceImpl;
pub use transaction_v1_api::TransactionV1API;
