# Multi-stage build for Rust application
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY raft-core/Cargo.toml ./raft-core/
COPY proto/Cargo.toml ./proto/
COPY server/Cargo.toml ./server/
COPY state/Cargo.toml ./state/
COPY cli/Cargo.toml ./cli/

# Create dummy source files to cache dependencies
RUN mkdir -p raft-core/src proto/src server/src state/src cli/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > cli/src/main.rs && \
    echo "// dummy" > raft-core/src/lib.rs && \
    echo "// dummy" > proto/src/lib.rs && \
    echo "// dummy" > server/src/lib.rs && \
    echo "// dummy" > state/src/lib.rs

# Build dependencies
RUN cargo build --release --bin raft-server
RUN rm -rf raft-core/src proto/src server/src state/src cli/src

# Copy source code
COPY raft-core/src ./raft-core/src/
COPY proto/src ./proto/src/
COPY proto/raft.proto ./proto/
COPY proto/build.rs ./proto/
COPY server/src ./server/src/
COPY state/src ./state/src/
COPY cli/src ./cli/src/

# Build the application
RUN cargo build --release --bin raft-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false raftuser

# Create directories
RUN mkdir -p /app/data && chown raftuser:raftuser /app/data

# Copy binary
COPY --from=builder /app/target/release/raft-server /usr/local/bin/raft-server

# Set permissions
RUN chmod +x /usr/local/bin/raft-server

# Switch to app user
USER raftuser

# Expose ports
EXPOSE 50051 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set working directory
WORKDIR /app

# Run the application
CMD ["raft-server"]
