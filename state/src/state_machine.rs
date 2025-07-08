use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::StateError;

/// Result type for state machine operations
pub type StateResult<T> = Result<T, StateError>;

/// Command that can be applied to the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Set a key-value pair
    Set { key: String, value: String },
    /// Get a value by key
    Get { key: String },
    /// Delete a key
    Delete { key: String },
    /// Custom command with arbitrary data
    Custom { data: Vec<u8> },
}

/// Result of applying a command to the state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    /// Success with optional return value
    Success { value: Option<String> },
    /// Error with message
    Error { message: String },
}

/// Trait for state machines that can be used with Raft
#[async_trait]
pub trait StateMachine: Send + Sync {
    /// Apply a command to the state machine
    async fn apply(&mut self, command: Command) -> StateResult<CommandResult>;
    
    /// Create a snapshot of the current state
    async fn snapshot(&self) -> StateResult<Vec<u8>>;
    
    /// Restore state from a snapshot
    async fn restore(&mut self, snapshot: Vec<u8>) -> StateResult<()>;
    
    /// Get the current state size (for metrics)
    fn size(&self) -> usize;
}
