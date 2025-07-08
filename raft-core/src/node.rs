use crate::types::*;
use crate::error::RaftError;
use crate::RaftResult;
use std::time::{Duration, Instant};
use tracing::{info, debug};

/// Main Raft node implementation
pub struct RaftNode {
    // Persistent state on all servers
    current_term: Term,
    voted_for: Option<NodeId>,
    log: Vec<LogEntry>,

    // Volatile state on all servers
    commit_index: LogIndex,
    last_applied: LogIndex,

    // Volatile state on leaders
    next_index: std::collections::HashMap<NodeId, LogIndex>,
    match_index: std::collections::HashMap<NodeId, LogIndex>,

    // Node configuration and state
    config: NodeConfig,
    state: NodeState,

    // Election and timing
    last_heartbeat: Instant,
    election_timeout: Duration,
    votes_received: std::collections::HashSet<NodeId>,

    // Leader state
    leader_id: Option<NodeId>,
}

impl RaftNode {
    /// Create a new Raft node
    pub fn new(config: NodeConfig) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let election_timeout = Duration::from_millis(
            rng.gen_range(config.election_timeout_min..=config.election_timeout_max)
        );

        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: std::collections::HashMap::new(),
            match_index: std::collections::HashMap::new(),
            config,
            state: NodeState::Follower,
            last_heartbeat: Instant::now(),
            election_timeout,
            votes_received: std::collections::HashSet::new(),
            leader_id: None,
        }
    }
    
    /// Get the current term
    pub fn current_term(&self) -> Term {
        self.current_term
    }
    
    /// Get the current state
    pub fn state(&self) -> NodeState {
        self.state
    }
    
    /// Get the node ID
    pub fn node_id(&self) -> &NodeId {
        &self.config.node_id
    }
    
    /// Handle a vote request
    pub fn handle_vote_request(&mut self, request: VoteRequest) -> RaftResult<VoteResponse> {
        debug!("Received vote request from {} for term {}", request.candidate_id, request.term);

        // If term is outdated, reject
        if request.term < self.current_term {
            return Ok(VoteResponse {
                term: self.current_term,
                vote_granted: false,
            });
        }

        // If term is newer, update our term and become follower
        if request.term > self.current_term {
            self.current_term = request.term;
            self.voted_for = None;
            self.state = NodeState::Follower;
            self.leader_id = None;
        }

        // Check if we can vote for this candidate
        let can_vote = self.voted_for.is_none() ||
                      self.voted_for.as_ref() == Some(&request.candidate_id);

        // Check if candidate's log is at least as up-to-date as ours
        let last_log_term = self.log.last().map(|e| e.term).unwrap_or(0);
        let last_log_index = self.log.len() as LogIndex;

        let log_ok = request.last_log_term > last_log_term ||
                    (request.last_log_term == last_log_term &&
                     request.last_log_index >= last_log_index);

        let vote_granted = can_vote && log_ok;

        if vote_granted {
            self.voted_for = Some(request.candidate_id.clone());
            self.reset_election_timeout();
            info!("Granted vote to {} for term {}", request.candidate_id, request.term);
        } else {
            debug!("Denied vote to {} for term {} (can_vote: {}, log_ok: {})",
                   request.candidate_id, request.term, can_vote, log_ok);
        }

        Ok(VoteResponse {
            term: self.current_term,
            vote_granted,
        })
    }
    
    /// Handle an append entries request
    pub fn handle_append_request(&mut self, request: AppendRequest) -> RaftResult<AppendResponse> {
        debug!("Received append entries from {} for term {}", request.leader_id, request.term);

        // Reset election timeout since we heard from a leader
        self.reset_election_timeout();

        // If term is outdated, reject
        if request.term < self.current_term {
            return Ok(AppendResponse {
                term: self.current_term,
                success: false,
                conflict_index: None,
                conflict_term: None,
            });
        }

        // If term is newer or equal, update our term and become follower
        if request.term >= self.current_term {
            self.current_term = request.term;
            self.state = NodeState::Follower;
            self.leader_id = Some(request.leader_id.clone());
            self.voted_for = None;
        }

        // Check if we have the previous log entry
        if request.prev_log_index > 0 {
            if request.prev_log_index > self.log.len() as LogIndex {
                // We don't have enough entries
                return Ok(AppendResponse {
                    term: self.current_term,
                    success: false,
                    conflict_index: Some(self.log.len() as LogIndex + 1),
                    conflict_term: None,
                });
            }

            let prev_entry = &self.log[(request.prev_log_index - 1) as usize];
            if prev_entry.term != request.prev_log_term {
                // Term mismatch - find the first entry with conflicting term
                let conflict_term = prev_entry.term;
                let mut conflict_index = request.prev_log_index;

                // Find first entry of the conflicting term
                for (i, entry) in self.log.iter().enumerate() {
                    if entry.term == conflict_term {
                        conflict_index = (i + 1) as LogIndex;
                        break;
                    }
                }

                return Ok(AppendResponse {
                    term: self.current_term,
                    success: false,
                    conflict_index: Some(conflict_index),
                    conflict_term: Some(conflict_term),
                });
            }
        }

        // If we have conflicting entries, remove them
        if !request.entries.is_empty() {
            let start_index = request.prev_log_index as usize;

            // Check for conflicts and truncate if necessary
            for (i, new_entry) in request.entries.iter().enumerate() {
                let log_index = start_index + i;
                if log_index < self.log.len() {
                    if self.log[log_index].term != new_entry.term {
                        // Conflict found - truncate from here
                        self.log.truncate(log_index);
                        break;
                    }
                } else {
                    break;
                }
            }

            // Append new entries
            for (i, new_entry) in request.entries.iter().enumerate() {
                let log_index = start_index + i;
                if log_index >= self.log.len() {
                    self.log.push(new_entry.clone());
                }
            }
        }

        // Update commit index
        if request.leader_commit > self.commit_index {
            self.commit_index = std::cmp::min(request.leader_commit, self.log.len() as LogIndex);
        }

        Ok(AppendResponse {
            term: self.current_term,
            success: true,
            conflict_index: None,
            conflict_term: None,
        })
    }
    
    /// Start an election
    pub fn start_election(&mut self) -> RaftResult<()> {
        info!("Starting election for term {}", self.current_term + 1);

        // Increment term and vote for ourselves
        self.current_term += 1;
        self.state = NodeState::Candidate;
        self.voted_for = Some(self.config.node_id.clone());
        self.leader_id = None;
        self.reset_election_timeout();

        // Clear votes and vote for ourselves
        self.votes_received.clear();
        self.votes_received.insert(self.config.node_id.clone());

        // If we're the only node, become leader immediately
        if self.config.peers.is_empty() {
            self.become_leader();
        }

        Ok(())
    }

    /// Become the leader
    fn become_leader(&mut self) {
        info!("Becoming leader for term {}", self.current_term);
        self.state = NodeState::Leader;
        self.leader_id = Some(self.config.node_id.clone());

        // Initialize leader state
        let next_index = self.log.len() as LogIndex + 1;
        self.next_index.clear();
        self.match_index.clear();

        for peer in &self.config.peers {
            self.next_index.insert(peer.clone(), next_index);
            self.match_index.insert(peer.clone(), 0);
        }

        // Send initial heartbeat (empty append entries)
        // This would be handled by the server layer
    }

    /// Check if election timeout has occurred
    pub fn is_election_timeout(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.election_timeout
    }

    /// Reset election timeout
    fn reset_election_timeout(&mut self) {
        self.last_heartbeat = Instant::now();

        // Randomize timeout for next election
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.election_timeout = Duration::from_millis(
            rng.gen_range(self.config.election_timeout_min..=self.config.election_timeout_max)
        );
    }

    /// Handle a vote response
    pub fn handle_vote_response(&mut self, from: &NodeId, response: VoteResponse) -> RaftResult<()> {
        // Only process if we're still a candidate and the term matches
        if self.state != NodeState::Candidate || response.term != self.current_term {
            return Ok(());
        }

        // If term is newer, step down
        if response.term > self.current_term {
            self.current_term = response.term;
            self.state = NodeState::Follower;
            self.voted_for = None;
            self.leader_id = None;
            return Ok(());
        }

        // Count the vote
        if response.vote_granted {
            self.votes_received.insert(from.clone());

            // Check if we have majority
            let total_nodes = 1 + self.config.peers.len(); // Include ourselves
            let majority = total_nodes / 2 + 1;

            if self.votes_received.len() >= majority {
                self.become_leader();
            }
        }

        Ok(())
    }
    
    /// Submit a command to the log
    pub fn submit_command(&mut self, command: Vec<u8>) -> RaftResult<LogIndex> {
        // Only leaders can accept commands
        if self.state != NodeState::Leader {
            return Err(RaftError::NotLeader);
        }

        // Create new log entry
        let entry = LogEntry {
            index: self.log.len() as LogIndex + 1,
            term: self.current_term,
            entry_type: EntryType::Command,
            data: command,
            client_id: None,
            sequence_number: None,
        };

        let log_index = entry.index;
        self.log.push(entry);

        info!("Added command to log at index {}", log_index);

        // Update match index for ourselves
        self.match_index.insert(self.config.node_id.clone(), log_index);

        Ok(log_index)
    }

    /// Check if we should send heartbeats (for leaders)
    pub fn should_send_heartbeat(&self) -> bool {
        if self.state != NodeState::Leader {
            return false;
        }

        let heartbeat_interval = Duration::from_millis(self.config.heartbeat_interval);
        self.last_heartbeat.elapsed() >= heartbeat_interval
    }

    /// Update commit index based on majority replication
    pub fn update_commit_index(&mut self) {
        if self.state != NodeState::Leader {
            return;
        }

        // Find the highest index that's replicated on a majority of servers
        let mut indices: Vec<LogIndex> = self.match_index.values().cloned().collect();
        indices.push(self.log.len() as LogIndex); // Include our own log
        indices.sort_unstable();
        indices.reverse();

        let majority_index = indices.len() / 2;
        if majority_index < indices.len() {
            let new_commit_index = indices[majority_index];

            // Only commit entries from current term
            if new_commit_index > self.commit_index {
                if let Some(entry) = self.log.get((new_commit_index - 1) as usize) {
                    if entry.term == self.current_term {
                        self.commit_index = new_commit_index;
                        info!("Updated commit index to {}", self.commit_index);
                    }
                }
            }
        }
    }

    /// Get the current leader ID
    pub fn leader_id(&self) -> Option<&NodeId> {
        self.leader_id.as_ref()
    }

    /// Get the commit index
    pub fn commit_index(&self) -> LogIndex {
        self.commit_index
    }

    /// Get the last applied index
    pub fn last_applied(&self) -> LogIndex {
        self.last_applied
    }

    /// Get the log length
    pub fn log_length(&self) -> usize {
        self.log.len()
    }

    /// Mark entries as applied up to the given index
    pub fn set_last_applied(&mut self, index: LogIndex) {
        self.last_applied = index;
    }

    /// Get entries that need to be applied to the state machine
    pub fn get_entries_to_apply(&self) -> &[LogEntry] {
        let start = self.last_applied as usize;
        let end = std::cmp::min(self.commit_index as usize, self.log.len());

        if start < end {
            &self.log[start..end]
        } else {
            &[]
        }
    }
}
