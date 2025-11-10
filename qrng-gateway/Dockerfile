# Multi-stage build for QRNG Gateway
# Build from workspace root: docker build -f qrng-gateway/Dockerfile -t qrng-gateway .

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
RUN cargo build --release --bin qrng-gateway

# Stage 2: Runtime
FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash qrng && \
    mkdir -p /app && \
    chown -R qrng:qrng /app

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=qrng:qrng /build/target/release/qrng-gateway /app/qrng-gateway

# Switch to non-root user
USER qrng

# Expose HTTP port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run with --env-mode flag to use environment variables
CMD ["/app/qrng-gateway", "--env-mode", "--log-level", "info"]
