use serde::{Deserialize, Serialize};

/// Configuration for the Raft server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Unique node identifier
    pub node_id: String,
    
    /// Address to bind the gRPC server to
    pub bind_address: String,
    
    /// Port for the gRPC server
    pub port: u16,
    
    /// List of peer addresses
    pub peers: Vec<String>,
    
    /// Minimum election timeout in milliseconds
    pub election_timeout_min: u64,
    
    /// Maximum election timeout in milliseconds
    pub election_timeout_max: u64,
    
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval: u64,
    
    /// Maximum number of log entries per append request
    pub max_append_entries: usize,
    
    /// Enable metrics endpoint
    pub enable_metrics: bool,
    
    /// Metrics server port
    pub metrics_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            node_id: "node-1".to_string(),
            bind_address: "0.0.0.0".to_string(),
            port: 50051,
            peers: Vec::new(),
            election_timeout_min: 150,
            election_timeout_max: 300,
            heartbeat_interval: 50,
            max_append_entries: 100,
            enable_metrics: true,
            metrics_port: 8080,
        }
    }
}

impl ServerConfig {
    /// Get the full server address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }
    
    /// Get the metrics address
    pub fn metrics_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.metrics_port)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.node_id.is_empty() {
            return Err("Node ID cannot be empty".to_string());
        }
        
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }
        
        if self.election_timeout_min >= self.election_timeout_max {
            return Err("Election timeout min must be less than max".to_string());
        }
        
        if self.heartbeat_interval == 0 {
            return Err("Heartbeat interval must be greater than 0".to_string());
        }
        
        Ok(())
    }
}
