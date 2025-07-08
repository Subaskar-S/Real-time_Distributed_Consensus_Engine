#[cfg(feature = "rocksdb-backend")]
use std::path::Path;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use rocksdb::{DB, Options};
use crate::state_machine::{StateMachine, Command, CommandResult, StateResult};
use crate::error::StateError;

/// RocksDB-backed key-value store implementation
pub struct RocksDbStore {
    db: DB,
}

impl RocksDbStore {
    /// Create a new RocksDB store
    pub fn new<P: AsRef<Path>>(path: P) -> StateResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self { db })
    }
    
    /// Get the number of key-value pairs (approximate)
    pub fn len(&self) -> StateResult<usize> {
        // RocksDB doesn't provide exact count efficiently
        // This is an approximation
        let mut count = 0;
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for _ in iter {
            count += 1;
        }
        Ok(count)
    }
    
    /// Check if the store is empty
    pub fn is_empty(&self) -> StateResult<bool> {
        let mut iter = self.db.iterator(rocksdb::IteratorMode::Start);
        Ok(iter.next().is_none())
    }
}

#[async_trait]
impl StateMachine for RocksDbStore {
    async fn apply(&mut self, command: Command) -> StateResult<CommandResult> {
        match command {
            Command::Set { key, value } => {
                self.db.put(key.as_bytes(), value.as_bytes())?;
                Ok(CommandResult::Success { value: None })
            }
            Command::Get { key } => {
                match self.db.get(key.as_bytes())? {
                    Some(value) => {
                        let value_str = String::from_utf8_lossy(&value).to_string();
                        Ok(CommandResult::Success { 
                            value: Some(value_str) 
                        })
                    }
                    None => Ok(CommandResult::Error { 
                        message: format!("Key '{}' not found", key) 
                    }),
                }
            }
            Command::Delete { key } => {
                // Check if key exists first
                match self.db.get(key.as_bytes())? {
                    Some(_) => {
                        self.db.delete(key.as_bytes())?;
                        Ok(CommandResult::Success { value: None })
                    }
                    None => Ok(CommandResult::Error { 
                        message: format!("Key '{}' not found", key) 
                    }),
                }
            }
            Command::Custom { .. } => {
                Ok(CommandResult::Error { 
                    message: "Custom commands not supported by RocksDB store".to_string() 
                })
            }
        }
    }
    
    async fn snapshot(&self) -> StateResult<Vec<u8>> {
        // Create a snapshot of all key-value pairs
        let mut snapshot_data = std::collections::HashMap::new();
        
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, value) = item?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            let value_str = String::from_utf8_lossy(&value).to_string();
            snapshot_data.insert(key_str, value_str);
        }
        
        serde_json::to_vec(&snapshot_data).map_err(StateError::from)
    }
    
    async fn restore(&mut self, snapshot: Vec<u8>) -> StateResult<()> {
        let snapshot_data: std::collections::HashMap<String, String> = 
            serde_json::from_slice(&snapshot)?;
        
        // Clear existing data
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        let keys_to_delete: Vec<Vec<u8>> = iter.map(|item| {
            item.map(|(key, _)| key.to_vec())
        }).collect::<Result<Vec<_>, _>>()?;
        
        for key in keys_to_delete {
            self.db.delete(&key)?;
        }
        
        // Restore from snapshot
        for (key, value) in snapshot_data {
            self.db.put(key.as_bytes(), value.as_bytes())?;
        }
        
        Ok(())
    }
    
    fn size(&self) -> usize {
        // This is expensive for RocksDB, so we return 0
        // In a real implementation, you might want to cache this
        0
    }
}
