use thiserror::Error;

/// Errors that can occur in server operations
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Raft error: {0}")]
    Raft(#[from] raft_core::RaftError),
    
    #[error("State error: {0}")]
    State(#[from] state::StateError),
    
    #[error("HTTP error: {0}")]
    Http(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
