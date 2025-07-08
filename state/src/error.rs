use thiserror::Error;

/// Errors that can occur in state machine operations
#[derive(Error, Debug)]
pub enum StateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[cfg(feature = "rocksdb-backend")]
    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),
}
