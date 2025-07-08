//! # Server
//!
//! HTTP server implementation for Raft nodes.
//!
//! This module provides the HTTP server that handles client requests
//! and provides metrics and status endpoints.

pub mod metrics;
pub mod config;
pub mod error;

pub use config::ServerConfig;
pub use error::ServerError;
