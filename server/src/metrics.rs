use prometheus::{Counter, Gauge, Histogram, Registry, Encoder, TextEncoder};
use std::sync::Arc;

/// Metrics collector for Raft operations
#[derive(Clone)]
pub struct RaftMetrics {
    registry: Arc<Registry>,
    
    // Raft state metrics
    pub current_term: Gauge,
    pub commit_index: Gauge,
    pub last_applied: Gauge,
    pub log_size: Gauge,
    
    // Operation counters
    pub vote_requests_total: Counter,
    pub append_requests_total: Counter,
    pub commands_total: Counter,
    pub elections_total: Counter,
    
    // Performance metrics
    pub consensus_latency: Histogram,
    pub append_latency: Histogram,
}

impl RaftMetrics {
    /// Create a new metrics collector
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        
        let current_term = Gauge::new("raft_current_term", "Current Raft term")?;
        let commit_index = Gauge::new("raft_commit_index", "Current commit index")?;
        let last_applied = Gauge::new("raft_last_applied", "Last applied log index")?;
        let log_size = Gauge::new("raft_log_size", "Total number of log entries")?;
        
        let vote_requests_total = Counter::new("raft_vote_requests_total", "Total vote requests")?;
        let append_requests_total = Counter::new("raft_append_requests_total", "Total append requests")?;
        let commands_total = Counter::new("raft_commands_total", "Total commands processed")?;
        let elections_total = Counter::new("raft_elections_total", "Total elections started")?;
        
        let consensus_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("raft_consensus_latency_seconds", "Consensus latency")
        )?;
        let append_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("raft_append_latency_seconds", "Append entries latency")
        )?;
        
        // Register all metrics
        registry.register(Box::new(current_term.clone()))?;
        registry.register(Box::new(commit_index.clone()))?;
        registry.register(Box::new(last_applied.clone()))?;
        registry.register(Box::new(log_size.clone()))?;
        registry.register(Box::new(vote_requests_total.clone()))?;
        registry.register(Box::new(append_requests_total.clone()))?;
        registry.register(Box::new(commands_total.clone()))?;
        registry.register(Box::new(elections_total.clone()))?;
        registry.register(Box::new(consensus_latency.clone()))?;
        registry.register(Box::new(append_latency.clone()))?;
        
        Ok(Self {
            registry,
            current_term,
            commit_index,
            last_applied,
            log_size,
            vote_requests_total,
            append_requests_total,
            commands_total,
            elections_total,
            consensus_latency,
            append_latency,
        })
    }
    
    /// Get metrics in Prometheus format
    pub fn gather(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

impl Default for RaftMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}
