# Multi-stage build for QRNG Collector
# Build from workspace root: docker build -f qrng-collector/Dockerfile -t qrng-collector .

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
COPY qrng-mcp ./qrng-mcp

# Build release binary
RUN cargo build --release --bin qrng-collector

# Stage 2: Runtime
FROM ubuntu:24.04

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
COPY --from=builder --chown=qrng:qrng /build/target/release/qrng-collector /app/qrng-collector

# Switch to non-root user
USER qrng

# Set environment variables
ENV RUST_LOG=info

# Run with --env-mode flag to use environment variables
CMD ["/app/qrng-collector", "--env-mode", "--log-level", "info"]
