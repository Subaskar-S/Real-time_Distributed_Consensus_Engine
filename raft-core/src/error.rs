use thiserror::Error;

/// Errors that can occur in Raft operations
#[derive(Error, Debug)]
pub enum RaftError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
    
    #[error("Term mismatch: expected {expected}, got {actual}")]
    TermMismatch { expected: u64, actual: u64 },
    
    #[error("Log inconsistency at index {index}")]
    LogInconsistency { index: u64 },
    
    #[error("Node not found: {node_id}")]
    NodeNotFound { node_id: String },
    
    #[error("Election timeout")]
    ElectionTimeout,
    
    #[error("Not leader")]
    NotLeader,
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}
