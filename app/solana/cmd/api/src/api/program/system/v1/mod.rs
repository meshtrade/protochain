//! System Program API v1 implementation
//!
//! This module contains the version 1 implementation of the System Program API,
//! including protocol buffer conversions, service implementation, and gRPC wrappers.

/// Protocol buffer conversion utilities for System Program operations
pub mod conversion;
/// Core business logic implementation for System Program operations
pub mod service_impl;
/// gRPC service wrapper for System Program v1 API
pub mod system_program_v1_api;

pub use service_impl::SystemProgramServiceImpl;
pub use system_program_v1_api::SystemProgramV1API;
