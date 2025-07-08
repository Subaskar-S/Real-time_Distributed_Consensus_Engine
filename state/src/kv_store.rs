use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::state_machine::{StateMachine, Command, CommandResult, StateResult};
use crate::error::StateError;

/// In-memory key-value store implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryKvStore {
    data: HashMap<String, String>,
}

impl InMemoryKvStore {
    /// Create a new empty key-value store
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    /// Get the number of key-value pairs
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for InMemoryKvStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateMachine for InMemoryKvStore {
    async fn apply(&mut self, command: Command) -> StateResult<CommandResult> {
        match command {
            Command::Set { key, value } => {
                self.data.insert(key, value);
                Ok(CommandResult::Success { value: None })
            }
            Command::Get { key } => {
                match self.data.get(&key) {
                    Some(value) => Ok(CommandResult::Success { 
                        value: Some(value.clone()) 
                    }),
                    None => Ok(CommandResult::Error { 
                        message: format!("Key '{}' not found", key) 
                    }),
                }
            }
            Command::Delete { key } => {
                match self.data.remove(&key) {
                    Some(_) => Ok(CommandResult::Success { value: None }),
                    None => Ok(CommandResult::Error { 
                        message: format!("Key '{}' not found", key) 
                    }),
                }
            }
            Command::Custom { .. } => {
                Ok(CommandResult::Error { 
                    message: "Custom commands not supported by KV store".to_string() 
                })
            }
        }
    }
    
    async fn snapshot(&self) -> StateResult<Vec<u8>> {
        serde_json::to_vec(&self.data).map_err(StateError::from)
    }
    
    async fn restore(&mut self, snapshot: Vec<u8>) -> StateResult<()> {
        self.data = serde_json::from_slice(&snapshot)?;
        Ok(())
    }
    
    fn size(&self) -> usize {
        self.data.len()
    }
}
