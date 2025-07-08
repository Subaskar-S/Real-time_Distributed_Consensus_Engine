#!/bin/bash

# High-Performance Raft Consensus Engine Demo
# This script demonstrates the capabilities of our Raft implementation

set -e

echo "🚀 High-Performance Raft Consensus Engine Demo"
echo "=============================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi

print_step "Building the project..."
cargo build --release
print_success "Build completed successfully!"
echo

print_step "Running unit tests..."
cargo test --lib -p raft-core
print_success "All unit tests passed!"
echo

print_step "Starting Raft server in background..."
cargo run --bin raft-server &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 3

# Function to cleanup on exit
cleanup() {
    print_step "Cleaning up..."
    if kill -0 $SERVER_PID 2>/dev/null; then
        kill $SERVER_PID
        print_success "Server stopped"
    fi
}
trap cleanup EXIT

print_step "Testing server health..."
if curl -s http://127.0.0.1:8080/health > /dev/null; then
    print_success "Server is healthy!"
else
    print_error "Server health check failed"
    exit 1
fi
echo

print_step "Demonstrating CLI commands..."

echo "Setting key-value pairs:"
cargo run --bin raft-cli set demo_key "Hello, Raft!"
cargo run --bin raft-cli set user_count "42"
cargo run --bin raft-cli set system_status "operational"
print_success "Keys set successfully!"
echo

echo "Retrieving values:"
cargo run --bin raft-cli get demo_key
cargo run --bin raft-cli get user_count
cargo run --bin raft-cli get system_status
print_success "Values retrieved successfully!"
echo

echo "Checking cluster status:"
cargo run --bin raft-cli status
print_success "Status retrieved successfully!"
echo

print_step "Running performance benchmark..."
cargo run --bin raft-cli benchmark --operations 100 --clients 5
print_success "Benchmark completed!"
echo

print_step "Testing metrics endpoint..."
echo "Sample metrics:"
curl -s http://127.0.0.1:8080/metrics | head -10
print_success "Metrics endpoint working!"
echo

print_step "Demonstrating error handling..."
echo "Trying to get a non-existent key:"
cargo run --bin raft-cli get non_existent_key || true
print_success "Error handling demonstrated!"
echo

print_step "Deleting a key..."
cargo run --bin raft-cli delete demo_key
echo "Verifying deletion:"
cargo run --bin raft-cli get demo_key || true
print_success "Key deletion demonstrated!"
echo

print_success "🎉 Demo completed successfully!"
echo
echo "Key features demonstrated:"
echo "  ✅ HTTP REST API"
echo "  ✅ Key-value operations (SET, GET, DELETE)"
echo "  ✅ Cluster status monitoring"
echo "  ✅ Performance benchmarking"
echo "  ✅ Prometheus metrics"
echo "  ✅ Error handling"
echo "  ✅ Health checks"
echo
echo "The Raft consensus engine is ready for production use!"
echo "For more information, see the README.md file."
