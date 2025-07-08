use crate::types::*;

/// Persistent state that must be maintained across restarts
#[derive(Debug, Clone)]
pub struct PersistentState {
    pub current_term: Term,
    pub voted_for: Option<NodeId>,
    pub log: Vec<LogEntry>,
}

impl PersistentState {
    /// Create new persistent state
    pub fn new() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
        }
    }
}

/// Volatile state maintained by all servers
#[derive(Debug, Clone)]
pub struct VolatileState {
    pub commit_index: LogIndex,
    pub last_applied: LogIndex,
}

impl VolatileState {
    /// Create new volatile state
    pub fn new() -> Self {
        Self {
            commit_index: 0,
            last_applied: 0,
        }
    }
}

/// Volatile state maintained by leaders
#[derive(Debug, Clone)]
pub struct LeaderState {
    pub next_index: std::collections::HashMap<NodeId, LogIndex>,
    pub match_index: std::collections::HashMap<NodeId, LogIndex>,
}

impl LeaderState {
    /// Create new leader state
    pub fn new() -> Self {
        Self {
            next_index: std::collections::HashMap::new(),
            match_index: std::collections::HashMap::new(),
        }
    }
    
    /// Initialize leader state for a set of peers
    pub fn initialize_for_peers(&mut self, peers: &[NodeId], last_log_index: LogIndex) {
        for peer in peers {
            self.next_index.insert(peer.clone(), last_log_index + 1);
            self.match_index.insert(peer.clone(), 0);
        }
    }
}
