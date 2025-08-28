//! `ProtoSol` Solana API Backend
//!
//! This crate provides a gRPC API layer over Solana blockchain operations,
//! implementing services for accounts, transactions, and system programs.

/// Core API implementations for Solana blockchain operations
pub mod api;
/// Configuration management for the API server
pub mod config;
/// Service provider pattern for dependency injection
pub mod service_providers;
/// WebSocket manager for real-time transaction monitoring
pub mod websocket;
