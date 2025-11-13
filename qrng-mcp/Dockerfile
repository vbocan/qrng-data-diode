# Multi-stage build for QRNG MCP Server
# Build from workspace root: docker build -f qrng-mcp/Dockerfile -t qrng-mcp .

# Stage 1: Builder
FROM rust:bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY qrng-core ./qrng-core
COPY qrng-collector ./qrng-collector
COPY qrng-gateway ./qrng-gateway
COPY qrng-mcp ./qrng-mcp

# Build release binary
RUN cargo build --release --bin qrng-mcp

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash qrng && \
    mkdir -p /app && \
    chown -R qrng:qrng /app

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=qrng:qrng /build/target/release/qrng-mcp /app/qrng-mcp

# Switch to non-root user
USER qrng

# Expose HTTP port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=qrng_mcp=info,rmcp=info

CMD ["/app/qrng-mcp"]
