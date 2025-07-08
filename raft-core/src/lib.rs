//! # Raft Core
//! 
//! Core implementation of the Raft consensus algorithm.
//! 
//! This module provides the fundamental Raft algorithm components including:
//! - Leader election
//! - Log replication  
//! - Term management
//! - State transitions
//! - Heartbeat mechanism

pub mod node;
pub mod log;
pub mod state;
pub mod election;
pub mod replication;
pub mod types;
pub mod error;
pub mod event_loop;

#[cfg(test)]
mod tests;

pub use node::RaftNode;
pub use types::*;
pub use error::RaftError;
pub use event_loop::{RaftEventLoop, RaftEvent, NodeStatus};

/// Result type for Raft operations
pub type RaftResult<T> = Result<T, RaftError>;
