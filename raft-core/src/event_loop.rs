use crate::types::*;
use crate::node::RaftNode;
use crate::error::RaftError;
use crate::RaftResult;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use std::collections::HashMap;

/// Events that can be sent to the Raft event loop
#[derive(Debug)]
pub enum RaftEvent {
    /// Vote request from another node
    VoteRequest {
        request: VoteRequest,
        response_tx: tokio::sync::oneshot::Sender<VoteResponse>,
    },
    /// Append entries request from leader
    AppendRequest {
        request: AppendRequest,
        response_tx: tokio::sync::oneshot::Sender<AppendResponse>,
    },
    /// Submit a command to the cluster
    SubmitCommand {
        command: Vec<u8>,
        response_tx: tokio::sync::oneshot::Sender<RaftResult<LogIndex>>,
    },
    /// Get current status
    GetStatus {
        response_tx: tokio::sync::oneshot::Sender<NodeStatus>,
    },
    /// Shutdown the event loop
    Shutdown,
}

/// Current status of a Raft node
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeStatus {
    pub node_id: NodeId,
    pub state: NodeState,
    pub current_term: Term,
    pub leader_id: Option<NodeId>,
    pub commit_index: LogIndex,
    pub last_applied: LogIndex,
    pub log_length: usize,
    pub peers: Vec<String>,
}

/// Raft event loop that coordinates all Raft operations
pub struct RaftEventLoop {
    node: Arc<RwLock<RaftNode>>,
    event_rx: mpsc::UnboundedReceiver<RaftEvent>,
    peer_clients: HashMap<NodeId, RaftPeerClient>,
}

/// Client for communicating with peer nodes
pub struct RaftPeerClient {
    node_id: NodeId,
    address: String,
    client: reqwest::Client,
}

impl RaftPeerClient {
    pub fn new(node_id: NodeId, address: String) -> Self {
        Self {
            node_id,
            address,
            client: reqwest::Client::new(),
        }
    }
    
    /// Send a vote request to this peer
    pub async fn request_vote(&self, request: &VoteRequest) -> RaftResult<VoteResponse> {
        let url = format!("{}/raft/vote", self.address);
        
        let response = self.client
            .post(&url)
            .json(request)
            .timeout(Duration::from_millis(1000))
            .send()
            .await
            .map_err(|e| RaftError::Network(e.to_string()))?;
            
        if response.status().is_success() {
            let vote_response: VoteResponse = response
                .json()
                .await
                .map_err(|e| RaftError::Network(e.to_string()))?;
            Ok(vote_response)
        } else {
            Err(RaftError::Network(format!("HTTP {}", response.status())))
        }
    }
    
    /// Send an append entries request to this peer
    pub async fn append_entries(&self, request: &AppendRequest) -> RaftResult<AppendResponse> {
        let url = format!("{}/raft/append", self.address);
        
        let response = self.client
            .post(&url)
            .json(request)
            .timeout(Duration::from_millis(2000))
            .send()
            .await
            .map_err(|e| RaftError::Network(e.to_string()))?;
            
        if response.status().is_success() {
            let append_response: AppendResponse = response
                .json()
                .await
                .map_err(|e| RaftError::Network(e.to_string()))?;
            Ok(append_response)
        } else {
            Err(RaftError::Network(format!("HTTP {}", response.status())))
        }
    }
}

impl RaftEventLoop {
    /// Create a new Raft event loop
    pub fn new(
        node: Arc<RwLock<RaftNode>>,
        event_rx: mpsc::UnboundedReceiver<RaftEvent>,
    ) -> Self {
        Self {
            node,
            event_rx,
            peer_clients: HashMap::new(),
        }
    }
    
    /// Initialize peer clients
    pub async fn initialize_peers(&mut self, peers: &[String]) {
        for (i, peer_addr) in peers.iter().enumerate() {
            let node_id = format!("node-{}", i + 1);
            let client = RaftPeerClient::new(node_id.clone(), peer_addr.clone());
            self.peer_clients.insert(node_id, client);
        }
    }
    
    /// Run the main event loop
    pub async fn run(mut self) -> RaftResult<()> {
        info!("Starting Raft event loop");
        
        // Create timers
        let mut election_timer = interval(Duration::from_millis(50));
        let mut heartbeat_timer = interval(Duration::from_millis(25));
        
        loop {
            tokio::select! {
                // Handle incoming events
                event = self.event_rx.recv() => {
                    match event {
                        Some(event) => {
                            if let Err(e) = self.handle_event(event).await {
                                error!("Error handling event: {}", e);
                            }
                        }
                        None => {
                            info!("Event channel closed, shutting down");
                            break;
                        }
                    }
                }
                
                // Check for election timeout
                _ = election_timer.tick() => {
                    if let Err(e) = self.check_election_timeout().await {
                        error!("Error checking election timeout: {}", e);
                    }
                }
                
                // Send heartbeats if leader
                _ = heartbeat_timer.tick() => {
                    if let Err(e) = self.send_heartbeats().await {
                        error!("Error sending heartbeats: {}", e);
                    }
                }
            }
        }
        
        info!("Raft event loop stopped");
        Ok(())
    }
    
    /// Handle a single event
    async fn handle_event(&mut self, event: RaftEvent) -> RaftResult<()> {
        match event {
            RaftEvent::VoteRequest { request, response_tx } => {
                let mut node = self.node.write().await;
                let response = node.handle_vote_request(request)?;
                let _ = response_tx.send(response);
            }
            
            RaftEvent::AppendRequest { request, response_tx } => {
                let mut node = self.node.write().await;
                let response = node.handle_append_request(request)?;
                let _ = response_tx.send(response);
            }
            
            RaftEvent::SubmitCommand { command, response_tx } => {
                let mut node = self.node.write().await;
                let result = node.submit_command(command);
                let _ = response_tx.send(result);
            }
            
            RaftEvent::GetStatus { response_tx } => {
                let node = self.node.read().await;
                let status = NodeStatus {
                    node_id: node.node_id().clone(),
                    state: node.state(),
                    current_term: node.current_term(),
                    leader_id: node.leader_id().cloned(),
                    commit_index: node.commit_index(),
                    last_applied: node.last_applied(),
                    log_length: node.log_length(),
                    peers: vec![], // TODO: Get from config
                };
                let _ = response_tx.send(status);
            }
            
            RaftEvent::Shutdown => {
                info!("Received shutdown event");
                return Err(RaftError::Configuration("Shutdown requested".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Check if election timeout has occurred and start election if needed
    async fn check_election_timeout(&mut self) -> RaftResult<()> {
        let should_start_election = {
            let node = self.node.read().await;
            node.state() != NodeState::Leader && node.is_election_timeout()
        };
        
        if should_start_election {
            self.start_election().await?;
        }
        
        Ok(())
    }
    
    /// Start a new election
    async fn start_election(&mut self) -> RaftResult<()> {
        let (vote_request, current_term) = {
            let mut node = self.node.write().await;
            node.start_election()?;
            
            let vote_request = VoteRequest {
                term: node.current_term(),
                candidate_id: node.node_id().clone(),
                last_log_index: node.log_length() as LogIndex,
                last_log_term: 0, // TODO: Get last log term
            };
            
            (vote_request, node.current_term())
        };
        
        info!("Starting election for term {}", current_term);
        
        // Send vote requests to all peers
        let mut vote_tasks = Vec::new();
        
        for (peer_id, client) in &self.peer_clients {
            let client = client.clone();
            let request = vote_request.clone();
            let peer_id = peer_id.clone();
            
            let task = tokio::spawn(async move {
                match client.request_vote(&request).await {
                    Ok(response) => Some((peer_id, response)),
                    Err(e) => {
                        warn!("Failed to get vote from {}: {}", peer_id, e);
                        None
                    }
                }
            });
            
            vote_tasks.push(task);
        }
        
        // Collect vote responses
        for task in vote_tasks {
            if let Ok(Some((peer_id, response))) = task.await {
                let mut node = self.node.write().await;
                node.handle_vote_response(&peer_id, response)?;
            }
        }
        
        Ok(())
    }
    
    /// Send heartbeats to all peers (if leader)
    async fn send_heartbeats(&mut self) -> RaftResult<()> {
        let should_send = {
            let node = self.node.read().await;
            node.state() == NodeState::Leader && node.should_send_heartbeat()
        };
        
        if !should_send {
            return Ok(());
        }
        
        let append_request = {
            let node = self.node.read().await;
            AppendRequest {
                term: node.current_term(),
                leader_id: node.node_id().clone(),
                prev_log_index: 0,
                prev_log_term: 0,
                entries: Vec::new(), // Empty for heartbeat
                leader_commit: node.commit_index(),
            }
        };
        
        debug!("Sending heartbeats to {} peers", self.peer_clients.len());
        
        // Send heartbeats to all peers
        let mut heartbeat_tasks = Vec::new();
        
        for (peer_id, client) in &self.peer_clients {
            let client = client.clone();
            let request = append_request.clone();
            let peer_id = peer_id.clone();
            
            let task = tokio::spawn(async move {
                match client.append_entries(&request).await {
                    Ok(response) => Some((peer_id, response)),
                    Err(e) => {
                        warn!("Failed to send heartbeat to {}: {}", peer_id, e);
                        None
                    }
                }
            });
            
            heartbeat_tasks.push(task);
        }
        
        // Process heartbeat responses
        for task in heartbeat_tasks {
            if let Ok(Some((_peer_id, _response))) = task.await {
                // TODO: Process heartbeat response
                // Update next_index and match_index based on response
            }
        }
        
        Ok(())
    }
}

// Make RaftPeerClient cloneable
impl Clone for RaftPeerClient {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            address: self.address.clone(),
            client: reqwest::Client::new(),
        }
    }
}
