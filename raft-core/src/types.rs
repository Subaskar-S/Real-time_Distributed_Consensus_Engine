use serde::{Deserialize, Serialize};

/// Unique identifier for a Raft node
pub type NodeId = String;

/// Raft term number
pub type Term = u64;

/// Log entry index
pub type LogIndex = u64;

/// Node state in the Raft protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Follower,
    Candidate,
    Leader,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeState::Follower => write!(f, "Follower"),
            NodeState::Candidate => write!(f, "Candidate"),
            NodeState::Leader => write!(f, "Leader"),
        }
    }
}

/// A log entry in the Raft log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub index: LogIndex,
    pub term: Term,
    pub entry_type: EntryType,
    pub data: Vec<u8>,
    pub client_id: Option<String>,
    pub sequence_number: Option<u64>,
}

/// Type of log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    Command,
    Configuration,
    NoOp,
}

/// Configuration for a Raft node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: NodeId,
    pub address: String,
    pub peers: Vec<String>,
    pub election_timeout_min: u64,
    pub election_timeout_max: u64,
    pub heartbeat_interval: u64,
}

/// Information about a peer node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: NodeId,
    pub address: String,
    pub voting: bool,
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub nodes: Vec<PeerInfo>,
    pub config_index: LogIndex,
}

/// Vote request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub term: Term,
    pub candidate_id: NodeId,
    pub last_log_index: LogIndex,
    pub last_log_term: Term,
}

/// Vote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    pub term: Term,
    pub vote_granted: bool,
}

/// Append entries request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendRequest {
    pub term: Term,
    pub leader_id: NodeId,
    pub prev_log_index: LogIndex,
    pub prev_log_term: Term,
    pub entries: Vec<LogEntry>,
    pub leader_commit: LogIndex,
}

/// Append entries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendResponse {
    pub term: Term,
    pub success: bool,
    pub conflict_index: Option<LogIndex>,
    pub conflict_term: Option<Term>,
}
