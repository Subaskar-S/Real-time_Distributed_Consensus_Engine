use crate::types::*;
use crate::error::RaftError;
use crate::RaftResult;

/// Election management for Raft nodes
pub struct ElectionManager {
    election_timeout_min: u64,
    election_timeout_max: u64,
    last_heartbeat: std::time::Instant,
}

impl ElectionManager {
    /// Create a new election manager
    pub fn new(timeout_min: u64, timeout_max: u64) -> Self {
        Self {
            election_timeout_min: timeout_min,
            election_timeout_max: timeout_max,
            last_heartbeat: std::time::Instant::now(),
        }
    }
    
    /// Check if election timeout has occurred
    pub fn is_election_timeout(&self) -> bool {
        let elapsed = self.last_heartbeat.elapsed().as_millis() as u64;
        let timeout = self.random_election_timeout();
        elapsed > timeout
    }
    
    /// Reset the election timeout
    pub fn reset_timeout(&mut self) {
        self.last_heartbeat = std::time::Instant::now();
    }
    
    /// Generate a random election timeout
    fn random_election_timeout(&self) -> u64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(self.election_timeout_min..=self.election_timeout_max)
    }
    
    /// Start a new election
    pub fn start_election(&mut self, current_term: Term) -> RaftResult<Term> {
        self.reset_timeout();
        Ok(current_term + 1)
    }
}
