use crate::types::*;
use crate::error::RaftError;
use crate::RaftResult;

/// Raft log implementation
pub struct RaftLog {
    entries: Vec<LogEntry>,
    commit_index: LogIndex,
}

impl RaftLog {
    /// Create a new empty log
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            commit_index: 0,
        }
    }
    
    /// Get the length of the log
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get the last log index
    pub fn last_index(&self) -> LogIndex {
        self.entries.len() as LogIndex
    }
    
    /// Get the term of the last log entry
    pub fn last_term(&self) -> Term {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }
    
    /// Get the commit index
    pub fn commit_index(&self) -> LogIndex {
        self.commit_index
    }
    
    /// Append entries to the log
    pub fn append(&mut self, entries: Vec<LogEntry>) -> RaftResult<()> {
        // TODO: Implement log append logic
        self.entries.extend(entries);
        Ok(())
    }
    
    /// Get an entry at a specific index
    pub fn get(&self, index: LogIndex) -> Option<&LogEntry> {
        if index == 0 || index > self.entries.len() as LogIndex {
            None
        } else {
            self.entries.get((index - 1) as usize)
        }
    }
    
    /// Truncate the log from a specific index
    pub fn truncate(&mut self, from_index: LogIndex) -> RaftResult<()> {
        if from_index <= self.entries.len() as LogIndex {
            self.entries.truncate((from_index - 1) as usize);
        }
        Ok(())
    }
    
    /// Update the commit index
    pub fn set_commit_index(&mut self, index: LogIndex) {
        self.commit_index = index.min(self.entries.len() as LogIndex);
    }
}
