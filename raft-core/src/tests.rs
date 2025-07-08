#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::node::RaftNode;

    fn create_test_config(node_id: &str) -> NodeConfig {
        NodeConfig {
            node_id: node_id.to_string(),
            address: format!("127.0.0.1:500{}", node_id.chars().last().unwrap()),
            peers: vec![],
            election_timeout_min: 150,
            election_timeout_max: 300,
            heartbeat_interval: 50,
        }
    }

    #[tokio::test]
    async fn test_node_creation() {
        let config = create_test_config("1");
        let node = RaftNode::new(config);
        
        assert_eq!(node.state(), NodeState::Follower);
        assert_eq!(node.current_term(), 0);
        assert_eq!(node.node_id(), "1");
        assert_eq!(node.log_length(), 0);
    }

    #[tokio::test]
    async fn test_vote_request_handling() {
        let config = create_test_config("1");
        let mut node = RaftNode::new(config);
        
        // Test vote request with higher term
        let vote_request = VoteRequest {
            term: 1,
            candidate_id: "2".to_string(),
            last_log_index: 0,
            last_log_term: 0,
        };
        
        let response = node.handle_vote_request(vote_request).unwrap();
        assert_eq!(response.term, 1);
        assert!(response.vote_granted);
        assert_eq!(node.current_term(), 1);
    }

    #[tokio::test]
    async fn test_vote_request_rejection() {
        let config = create_test_config("1");
        let mut node = RaftNode::new(config);
        
        // Set current term to 2
        node.start_election().unwrap(); // term becomes 1
        node.start_election().unwrap(); // term becomes 2
        
        // Test vote request with lower term
        let vote_request = VoteRequest {
            term: 1,
            candidate_id: "2".to_string(),
            last_log_index: 0,
            last_log_term: 0,
        };
        
        let response = node.handle_vote_request(vote_request).unwrap();
        assert_eq!(response.term, 2);
        assert!(!response.vote_granted);
    }

    #[tokio::test]
    async fn test_append_entries_heartbeat() {
        let config = create_test_config("1");
        let mut node = RaftNode::new(config);
        
        // Test heartbeat from leader
        let append_request = AppendRequest {
            term: 1,
            leader_id: "2".to_string(),
            prev_log_index: 0,
            prev_log_term: 0,
            entries: vec![],
            leader_commit: 0,
        };
        
        let response = node.handle_append_request(append_request).unwrap();
        assert_eq!(response.term, 1);
        assert!(response.success);
        assert_eq!(node.current_term(), 1);
        assert_eq!(node.state(), NodeState::Follower);
    }

    #[tokio::test]
    async fn test_command_submission() {
        let config = create_test_config("1");
        let mut node = RaftNode::new(config);
        
        // Start election to become leader
        node.start_election().unwrap();
        
        // Submit a command
        let command = b"test command".to_vec();
        let result = node.submit_command(command);
        
        assert!(result.is_ok());
        let log_index = result.unwrap();
        assert_eq!(log_index, 1);
        assert_eq!(node.log_length(), 1);
    }

    #[tokio::test]
    async fn test_election_timeout() {
        let config = create_test_config("1");
        let node = RaftNode::new(config);
        
        // Initially should not timeout
        assert!(!node.is_election_timeout());
        
        // After starting election, timeout should reset
        // Note: This test is timing-dependent and might be flaky
        // In a real implementation, we'd use mock time
    }

    #[tokio::test]
    async fn test_log_replication() {
        let config = create_test_config("1");
        let mut node = RaftNode::new(config);
        
        // Become leader
        node.start_election().unwrap();
        
        // Add some entries
        node.submit_command(b"command1".to_vec()).unwrap();
        node.submit_command(b"command2".to_vec()).unwrap();
        
        assert_eq!(node.log_length(), 2);
        
        // Test append entries with new entries
        let new_entry = LogEntry {
            index: 3,
            term: 1,
            entry_type: EntryType::Command,
            data: b"command3".to_vec(),
            client_id: None,
            sequence_number: None,
        };
        
        let append_request = AppendRequest {
            term: 1,
            leader_id: "2".to_string(),
            prev_log_index: 2,
            prev_log_term: 1,
            entries: vec![new_entry],
            leader_commit: 2,
        };
        
        let response = node.handle_append_request(append_request).unwrap();
        assert!(response.success);
        assert_eq!(node.log_length(), 3);
    }

    #[tokio::test]
    async fn test_commit_index_update() {
        let config = create_test_config("1");
        let mut node = RaftNode::new(config);
        
        // Become leader and add entries
        node.start_election().unwrap();
        node.submit_command(b"command1".to_vec()).unwrap();
        node.submit_command(b"command2".to_vec()).unwrap();
        
        // Initially commit index should be 0
        assert_eq!(node.commit_index(), 0);
        
        // Update commit index
        node.update_commit_index();
        
        // Since we're the only node, commit index should advance
        assert!(node.commit_index() > 0);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let mut config = create_test_config("1");
        config.peers = vec!["node-2".to_string()]; // Add a peer so it doesn't immediately become leader
        let mut node = RaftNode::new(config);

        // Start as follower
        assert_eq!(node.state(), NodeState::Follower);

        // Start election -> become candidate
        node.start_election().unwrap();
        assert_eq!(node.state(), NodeState::Candidate);
    }

    #[tokio::test]
    async fn test_vote_response_handling() {
        let mut config = create_test_config("1");
        config.peers = vec!["node-2".to_string(), "node-3".to_string(), "node-4".to_string()];
        let mut node = RaftNode::new(config);

        // Start election
        node.start_election().unwrap();
        assert_eq!(node.state(), NodeState::Candidate);

        // Receive vote from peer (total: self + 1 peer = 2 out of 4, not majority yet)
        let vote_response = VoteResponse {
            term: 1,
            vote_granted: true,
        };

        node.handle_vote_response(&"node-2".to_string(), vote_response).unwrap();

        // Should still be candidate (need majority: 3 out of 4)
        assert_eq!(node.state(), NodeState::Candidate);

        // Receive another vote (total: self + 2 peers = 3 out of 4, now majority!)
        let vote_response2 = VoteResponse {
            term: 1,
            vote_granted: true,
        };

        node.handle_vote_response(&"node-3".to_string(), vote_response2).unwrap();

        // Now should be leader (have majority: self + 2 peers = 3/4)
        assert_eq!(node.state(), NodeState::Leader);
    }
}
