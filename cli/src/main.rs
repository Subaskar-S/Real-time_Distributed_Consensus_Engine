//! # Raft CLI
//!
//! Command-line interface for managing Raft clusters.
//!
//! This tool allows you to:
//! - Send commands to the cluster
//! - Trigger leader elections
//! - Inspect node state
//! - Manage cluster membership

use clap::{Parser, Subcommand};
use anyhow::Result;
use serde_json::json;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "raft-cli")]
#[command(about = "A CLI for managing Raft clusters")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a key-value command to the cluster
    Set {
        /// Key to set
        key: String,
        /// Value to set
        value: String,
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
    /// Get a value from the cluster
    Get {
        /// Key to get
        key: String,
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
    /// Delete a key from the cluster
    Delete {
        /// Key to delete
        key: String,
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
    /// Get cluster status
    Status {
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
    /// Get cluster metrics
    Metrics {
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
    /// Check node health
    Health {
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
    /// Benchmark the cluster
    Benchmark {
        /// Number of operations to perform
        #[arg(short, long, default_value = "1000")]
        operations: usize,
        /// Number of concurrent clients
        #[arg(short, long, default_value = "10")]
        clients: usize,
        /// Target node address
        #[arg(short, long, default_value = "http://127.0.0.1:8080")]
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set { key, value, address } => {
            send_kv_command(&address, "SET", &key, Some(&value)).await?;
        }
        Commands::Get { key, address } => {
            send_kv_command(&address, "GET", &key, None).await?;
        }
        Commands::Delete { key, address } => {
            send_kv_command(&address, "DELETE", &key, None).await?;
        }
        Commands::Status { address } => {
            get_status(&address).await?;
        }
        Commands::Metrics { address } => {
            get_metrics(&address).await?;
        }
        Commands::Health { address } => {
            check_health(&address).await?;
        }
        Commands::Benchmark { operations, clients, address } => {
            run_benchmark(&address, operations, clients).await?;
        }
    }

    Ok(())
}

async fn send_kv_command(address: &str, operation: &str, key: &str, value: Option<&str>) -> Result<()> {
    let client = reqwest::Client::new();

    let command = match operation {
        "SET" => {
            let value = value.ok_or_else(|| anyhow::anyhow!("SET operation requires a value"))?;
            json!({
                "type": "SET",
                "key": key,
                "value": value
            })
        }
        "GET" => {
            json!({
                "type": "GET",
                "key": key
            })
        }
        "DELETE" => {
            json!({
                "type": "DELETE",
                "key": key
            })
        }
        _ => return Err(anyhow::anyhow!("Unknown operation: {}", operation)),
    };

    let url = format!("{}/command", address);
    let response = client
        .post(&url)
        .json(&command)
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let result: serde_json::Value = resp.json().await?;
                println!("✅ Success: {}", serde_json::to_string_pretty(&result)?);
            } else {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                println!("❌ Error: HTTP {} - {}", status, error_text);
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to {}: {}", address, e);
            println!("💡 Make sure the Raft server is running and accessible");
        }
    }

    Ok(())
}

async fn get_status(address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/status", address);

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let status: serde_json::Value = resp.json().await?;
                println!("📊 Cluster Status:");
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!("❌ Error getting status: HTTP {}", resp.status());
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to {}: {}", address, e);
        }
    }

    Ok(())
}

async fn get_metrics(address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/metrics", address);

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let metrics = resp.text().await?;
                println!("📈 Cluster Metrics:");
                println!("{}", metrics);
            } else {
                println!("❌ Error getting metrics: HTTP {}", resp.status());
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to {}: {}", address, e);
        }
    }

    Ok(())
}

async fn check_health(address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/health", address);

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("✅ Node at {} is healthy", address);
            } else {
                println!("❌ Node at {} is unhealthy: HTTP {}", address, resp.status());
            }
        }
        Err(e) => {
            println!("❌ Node at {} is unreachable: {}", address, e);
        }
    }

    Ok(())
}

async fn run_benchmark(address: &str, operations: usize, clients: usize) -> Result<()> {
    use std::time::Instant;
    use tokio::task::JoinSet;

    println!("🚀 Starting benchmark:");
    println!("   Operations: {}", operations);
    println!("   Concurrent clients: {}", clients);
    println!("   Target: {}", address);
    println!();

    let ops_per_client = operations / clients;
    let remaining_ops = operations % clients;

    let start_time = Instant::now();
    let mut join_set = JoinSet::new();

    // Spawn client tasks
    for client_id in 0..clients {
        let client_ops = if client_id < remaining_ops {
            ops_per_client + 1
        } else {
            ops_per_client
        };

        let address = address.to_string();
        join_set.spawn(async move {
            run_client_benchmark(client_id, client_ops, &address).await
        });
    }

    // Wait for all clients to complete
    let mut total_successful = 0;
    let mut total_failed = 0;

    while let Some(result) = join_set.join_next().await {
        match result? {
            Ok((successful, failed)) => {
                total_successful += successful;
                total_failed += failed;
            }
            Err(e) => {
                println!("❌ Client error: {}", e);
                total_failed += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    let total_ops = total_successful + total_failed;
    let ops_per_sec = if duration.as_secs_f64() > 0.0 {
        total_ops as f64 / duration.as_secs_f64()
    } else {
        0.0
    };

    println!();
    println!("📊 Benchmark Results:");
    println!("   Duration: {:.2}s", duration.as_secs_f64());
    println!("   Total operations: {}", total_ops);
    println!("   Successful: {} ({:.1}%)", total_successful,
             (total_successful as f64 / total_ops as f64) * 100.0);
    println!("   Failed: {} ({:.1}%)", total_failed,
             (total_failed as f64 / total_ops as f64) * 100.0);
    println!("   Throughput: {:.1} ops/sec", ops_per_sec);
    println!("   Average latency: {:.2}ms",
             duration.as_millis() as f64 / total_ops as f64);

    Ok(())
}

async fn run_client_benchmark(client_id: usize, operations: usize, address: &str) -> Result<(usize, usize)> {
    let client = reqwest::Client::new();
    let mut successful = 0;
    let mut failed = 0;

    for i in 0..operations {
        let key = format!("bench_client_{}_key_{}", client_id, i);
        let value = format!("value_{}", i);

        let command = json!({
            "type": "SET",
            "key": key,
            "value": value
        });

        let url = format!("{}/command", address);
        let result = client
            .post(&url)
            .json(&command)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                successful += 1;
            }
            _ => {
                failed += 1;
            }
        }

        // Print progress for the first client
        if client_id == 0 && (i + 1) % (operations / 10).max(1) == 0 {
            let progress = ((i + 1) as f64 / operations as f64) * 100.0;
            print!("\r⏳ Progress: {:.1}%", progress);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
    }

    if client_id == 0 {
        println!("\r✅ Progress: 100.0%");
    }

    Ok((successful, failed))
}
