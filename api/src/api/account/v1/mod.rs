//! Account service v1 API and implementation
//!
//! This module contains the gRPC service definition and business logic
//! for account management operations.

/// gRPC service wrapper module for account operations
pub mod account_v1_api;
/// Core business logic implementation module for account operations
pub mod service_impl;

pub use account_v1_api::AccountV1API;
pub use service_impl::AccountServiceImpl;
