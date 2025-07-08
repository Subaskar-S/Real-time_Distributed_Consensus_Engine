use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, error};
use tracing_subscriber;
use axum::{
    routing::{get, post},
    Router,
    extract::{State, Json},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};

use server::{ServerConfig, metrics::RaftMetrics};
use raft_core::{RaftNode, RaftEventLoop, RaftEvent, NodeConfig, NodeStatus};
use state::{StateMachine, InMemoryKvStore, state_machine::Command};

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    event_tx: mpsc::UnboundedSender<RaftEvent>,
    metrics: Arc<RaftMetrics>,
    state_machine: Arc<RwLock<dyn StateMachine>>,
}

/// Command request from clients
#[derive(Debug, Deserialize)]
struct CommandRequest {
    #[serde(rename = "type")]
    command_type: String,
    key: String,
    value: Option<String>,
}

/// Command response to clients
#[derive(Debug, Serialize)]
struct CommandResponse {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration (for now, use default)
    let config = ServerConfig::default();
    config.validate().map_err(|e| format!("Invalid configuration: {}", e))?;

    info!("Starting Raft node: {}", config.node_id);
    info!("HTTP server address: {}", config.server_address());

    // Create Raft node
    let node_config = NodeConfig {
        node_id: config.node_id.clone(),
        address: config.server_address(),
        peers: config.peers.clone(),
        election_timeout_min: config.election_timeout_min,
        election_timeout_max: config.election_timeout_max,
        heartbeat_interval: config.heartbeat_interval,
    };

    let raft_node = Arc::new(RwLock::new(RaftNode::new(node_config)));
    let state_machine: Arc<RwLock<dyn StateMachine>> = Arc::new(RwLock::new(InMemoryKvStore::new()));
    let metrics = Arc::new(RaftMetrics::new().map_err(|e| format!("Failed to create metrics: {}", e))?);

    // Create event channel
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // Create application state
    let app_state = AppState {
        event_tx: event_tx.clone(),
        metrics: Arc::clone(&metrics),
        state_machine: Arc::clone(&state_machine),
    };
    
    // Start Raft event loop
    let event_loop = RaftEventLoop::new(Arc::clone(&raft_node), event_rx);
    let event_loop_handle = tokio::spawn(async move {
        if let Err(e) = event_loop.run().await {
            error!("Raft event loop error: {}", e);
        }
    });

    // Create HTTP server
    let app = Router::new()
        .route("/command", post(handle_command))
        .route("/status", get(handle_status))
        .route("/metrics", get(handle_metrics))
        .route("/health", get(handle_health))
        .with_state(app_state);

    // Start HTTP server
    let addr: SocketAddr = config.server_address().parse()?;
    let http_handle = tokio::spawn(async move {
        info!("Starting HTTP server on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        if let Err(e) = axum::serve(listener, app).await {
            error!("HTTP server error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = http_handle => {
            error!("HTTP server terminated unexpectedly");
        }
        _ = event_loop_handle => {
            error!("Event loop terminated unexpectedly");
        }
    }

    // Send shutdown event
    let _ = event_tx.send(RaftEvent::Shutdown);

    info!("Raft node shutting down");
    Ok(())
}

/// Handle command submission
async fn handle_command(
    State(state): State<AppState>,
    Json(request): Json<CommandRequest>,
) -> ResponseJson<CommandResponse> {
    // Convert HTTP request to state machine command
    let command = match request.command_type.as_str() {
        "SET" => {
            if let Some(value) = request.value {
                Command::Set {
                    key: request.key,
                    value,
                }
            } else {
                return ResponseJson(CommandResponse {
                    success: false,
                    result: None,
                    error: Some("SET command requires a value".to_string()),
                });
            }
        }
        "GET" => Command::Get { key: request.key },
        "DELETE" => Command::Delete { key: request.key },
        _ => {
            return ResponseJson(CommandResponse {
                success: false,
                result: None,
                error: Some(format!("Unknown command type: {}", request.command_type)),
            });
        }
    };

    // Serialize command
    let command_bytes = match serde_json::to_vec(&command) {
        Ok(bytes) => bytes,
        Err(e) => {
            return ResponseJson(CommandResponse {
                success: false,
                result: None,
                error: Some(format!("Failed to serialize command: {}", e)),
            });
        }
    };

    // Submit command to Raft
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let event = RaftEvent::SubmitCommand {
        command: command_bytes,
        response_tx,
    };

    if let Err(_) = state.event_tx.send(event) {
        return ResponseJson(CommandResponse {
            success: false,
            result: None,
            error: Some("Failed to submit command to Raft".to_string()),
        });
    }

    // Wait for Raft response
    match response_rx.await {
        Ok(Ok(_log_index)) => {
            // Command was accepted by Raft, now apply it to state machine
            let mut sm = state.state_machine.write().await;
            match sm.apply(command).await {
                Ok(result) => {
                    let result_value = match result {
                        state::state_machine::CommandResult::Success { value } => value,
                        state::state_machine::CommandResult::Error { message } => {
                            return ResponseJson(CommandResponse {
                                success: false,
                                result: None,
                                error: Some(message),
                            });
                        }
                    };

                    ResponseJson(CommandResponse {
                        success: true,
                        result: result_value,
                        error: None,
                    })
                }
                Err(e) => ResponseJson(CommandResponse {
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Ok(Err(e)) => ResponseJson(CommandResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
        Err(_) => ResponseJson(CommandResponse {
            success: false,
            result: None,
            error: Some("Failed to receive response from Raft".to_string()),
        }),
    }
}

/// Handle status requests
async fn handle_status(State(state): State<AppState>) -> ResponseJson<NodeStatus> {
    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let event = RaftEvent::GetStatus { response_tx };

    if let Err(_) = state.event_tx.send(event) {
        // Return a default status if we can't get the real one
        return ResponseJson(NodeStatus {
            node_id: "unknown".to_string(),
            state: raft_core::NodeState::Follower,
            current_term: 0,
            leader_id: None,
            commit_index: 0,
            last_applied: 0,
            log_length: 0,
            peers: vec![],
        });
    }

    match response_rx.await {
        Ok(status) => ResponseJson(status),
        Err(_) => ResponseJson(NodeStatus {
            node_id: "unknown".to_string(),
            state: raft_core::NodeState::Follower,
            current_term: 0,
            leader_id: None,
            commit_index: 0,
            last_applied: 0,
            log_length: 0,
            peers: vec![],
        }),
    }
}

/// Handle metrics requests
async fn handle_metrics(State(state): State<AppState>) -> String {
    match state.metrics.gather() {
        Ok(metrics) => metrics,
        Err(e) => {
            error!("Failed to gather metrics: {}", e);
            "# Failed to gather metrics\n".to_string()
        }
    }
}

/// Handle health check requests
async fn handle_health() -> &'static str {
    "OK"
}
