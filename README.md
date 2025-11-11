# QRNG Data Diode: Software-Based Quantum Entropy Bridge

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A high-performance, secure bridge service that exposes Quantum Random Number Generator (QRNG) entropy to external networks using software-based data diode emulation. Implemented in Rust for maximum safety, performance, and reliability.

## Overview

This system provides secure access to quantum-generated random numbers from a locally-networked Quantis QRNG appliance, designed for academic research, cryptographic applications, and scientific computing. It addresses network restrictions through a split architecture that emulates unidirectional data flow, inspired by hardware data diodes.

### Key Features

- **Software Data Diode**: Unidirectional entropy flow from internal to external networks
- **Multiple Source Aggregation**: Combine entropy from multiple QRNG appliances with XOR or HKDF mixing
- **High Performance**: Lock-free buffers, zero-copy operations, async I/O, parallel fetching
- **Cryptographic Integrity**: HMAC-SHA256 signing + CRC32 checksums
- **Production Ready**: Comprehensive metrics, structured logging, health checks
- **AI Integration**: Model Context Protocol (MCP) server for AI agents
- **Quality Validation**: Built-in Monte Carlo π estimation for randomness verification

## Architecture

### System Architecture

```
┌─────────────────────────┐         ┌─────────────────────────┐
│   Internal Network      │         │   External Network      │
│                         │         │                         │
│  ┌──────────────────┐   │         │  ┌──────────────────┐   │
│  │  Quantis QRNG    │   │         │  │   AI Agents      │   │
│  │    Appliance     │   │         │  │   Clients        │   │
│  └────────┬─────────┘   │         │  └────────▲─────────┘   │
│           │ HTTPS       │         │           │             │
│           │ fetch       │         │           │ REST API    │
│           ▼             │         │           │             │
│  ┌──────────────────┐   │         │  ┌────────┴─────────┐   │
│  │      QRNG        │   │  HTTPS  │  │      QRNG        │   │
│  │    Collector     │───┼────────>│──│     Gateway      │   │
│  │                  │   │  push   │  │                  │   │
│  │  - Fetch loop    │   │  (one-  │  │  - REST API      │   │
│  │  - Buffer (1MB)  │   │   way)  │  │  - Buffer (10MB) │   │
│  │  - HMAC signing  │   │         │  │  - MCP Server    │   │
│  └──────────────────┘   │         │  │  - Metrics       │   │
│                         │         │  └──────────────────┘   │
└─────────────────────────┘         └─────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75 or later
- Access to a Quantis QRNG appliance (or API-compatible endpoint)
- OpenSSL (for key generation)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/qrng-data-diode.git
cd qrng-data-diode

# Build all components (optimized release)
cargo build --release

# The binaries will be in target/release/
```

### Configuration

#### 1. Generate HMAC Secret Key

```bash
# Generate a 256-bit secret key
openssl rand -hex 32
```

#### 2. Configure Entropy Collector

Edit `qrng-collector/.env` (copy from `.env.example`):

**Single Source:**
```bash
# Quantis Appliance API endpoint (check your appliance manual for exact path)
QRNG_APPLIANCE_URL=https://quantis-appliance.local/api/random
QRNG_PUSH_URL=http://gateway-host:8080/push
QRNG_HMAC_SECRET_KEY=<your-generated-key>
```

**Multiple Sources (recommended for enhanced security):**
```bash
# Use different Quantis appliances or different QRNG sources
QRNG_APPLIANCE_URLS=https://quantis1.local/api/random,https://quantis2.local/api/random
QRNG_MIXING_STRATEGY=xor  # or "hkdf" for better mixing
QRNG_PUSH_URL=http://gateway-host:8080/push
QRNG_HMAC_SECRET_KEY=<your-generated-key>
```

**Important**: 
- Use the **API endpoint** (e.g., `/api/random`), NOT the website homepage
- Verify the endpoint returns binary random data, not HTML
- Check your Quantis Appliance user manual for the correct API path

**Mixing Strategies:**
- `xor`: Fast XOR-based mixing (good for independent sources)
- `hkdf`: HMAC-based Key Derivation Function (better for correlated sources)

#### 3. Configure Entropy Gateway

Edit `config/gateway.yaml`:

```yaml
listen_address: "0.0.0.0:8080"
api_keys:
  - "your-secure-api-key"
hmac_secret_key: "<same-key-as-collector>"
```

### Running

```bash
# Terminal 1: Start QRNG Gateway (external network)
./target/release/qrng-gateway --config config/gateway.yaml

# Terminal 2: Start QRNG Collector (internal network)
./target/release/qrng-collector --config config/collector.yaml
```

## API Reference

### GET /api/random

Fetch random bytes from the entropy buffer.

**Query Parameters:**
- `bytes` (required): Number of bytes to fetch (1-65536)
- `encoding` (optional): Output encoding: `binary`, `hex`, `base64` (default: `hex`)

**Headers:**
- `Authorization: Bearer <api-key>` or `?api_key=<api-key>`

**Example:**

```bash
# Hex-encoded random data (64 bytes)
curl -H "Authorization: Bearer your-api-key" \
  "https://gateway.example.com/api/random?bytes=64&encoding=hex"

# Binary random data (for piping)
curl -H "Authorization: Bearer your-api-key" \
  "https://gateway.example.com/api/random?bytes=1024&encoding=binary" \
  > random.bin

# Base64-encoded (for JSON embedding)
curl -H "Authorization: Bearer your-api-key" \
  "https://gateway.example.com/api/random?bytes=32&encoding=base64"
```

**Response:**

```
HTTP/1.1 200 OK
Content-Type: text/plain; charset=utf-8

a3f7c9e21b8d4f6a0c5e8b3d9f1a7c2e...
```

### GET /api/status

Get system health and buffer status.

**Example:**

```bash
curl -H "Authorization: Bearer your-api-key" \
  "https://gateway.example.com/api/status"
```

**Response:**

```json
{
  "status": "healthy",
  "buffer_fill_percent": 73.5,
  "buffer_bytes_available": 7864320,
  "last_data_received": "2025-11-06T09:15:30Z",
  "data_freshness_seconds": 12,
  "uptime_seconds": 3600,
  "total_requests_served": 15234,
  "total_bytes_served": 48234567,
  "requests_per_second": 4.23,
  "warnings": []
}
```

### GET /health

Lightweight health check for load balancers.

**Response:**
- `200 OK` - Service healthy, sufficient buffer
- `503 Service Unavailable` - Buffer depleted or service degraded

### GET /metrics

Prometheus-compatible metrics endpoint.

**Example:**

```bash
curl "https://gateway.example.com/metrics"
```

### POST /api/test/monte-carlo

Run Monte Carlo π estimation to validate randomness quality.

**Query Parameters:**
- `iterations`: Number of samples (default: 1000000, max: 10000000)

**Example:**

```bash
curl -X POST \
  -H "Authorization: Bearer your-api-key" \
  "https://gateway.example.com/api/test/monte-carlo?iterations=1000000"
```

**Response:**

```json
{
  "estimated_pi": 3.141598,
  "error": 0.000005,
  "error_percent": 0.0002,
  "iterations": 1000000,
  "convergence_rate": "excellent",
  "quantum_vs_pseudo": {
    "quantum_error": 0.000005,
    "pseudo_error": 0.000023,
    "improvement_factor": 4.6
  }
}
```

**Convergence Rates:**
- `excellent`: Error < 0.01% (suitable for critical applications)
- `good`: Error < 0.1% (suitable for most uses)
- `fair`: Error < 1.0% (acceptable quality)
- `poor`: Error ≥ 1.0% (may indicate issues)

## MCP Server Integration

The gateway exposes a Model Context Protocol (MCP) server for AI agent integration.

### Available Tools

- `get_random_bytes` - Fetch random bytes
- `get_random_integers` - Generate random integers in range
- `get_random_floats` - Generate random floats in [0,1)
- `get_random_uuid` - Generate UUID v4
- `get_status` - Query system status

### Usage Example (Claude Desktop)

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "qrng": {
      "command": "qrng-gateway",
      "args": ["--mcp-mode", "--config", "config/gateway.yaml"]
    }
  }
}
```

## Development

### Project Structure

```
qrng-data-diode/
├── qrng-core/              # Shared library
│   ├── src/
│   │   ├── buffer.rs       # High-performance entropy buffer
│   │   ├── protocol.rs     # Packet format & serialization
│   │   ├── crypto.rs       # HMAC signing, encoding
│   │   ├── config.rs       # Configuration management
│   │   ├── fetcher.rs      # HTTPS client for QRNG
│   │   ├── retry.rs        # Exponential backoff logic
│   │   └── metrics.rs      # Performance metrics
│   └── Cargo.toml
├── qrng-collector/         # Internal component
│   ├── src/main.rs
│   └── Cargo.toml
├── qrng-gateway/           # External component
│   ├── src/main.rs
│   └── Cargo.toml
├── qrng-mcp/               # MCP server implementation
│   ├── src/lib.rs
│   └── Cargo.toml
├── config/                 # Example configurations
└── docs/                   # Documentation
```

### Building

```bash
# Development build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific test
cargo test test_buffer_operations

# With logging
RUST_LOG=debug cargo test
```

## Performance

Benchmarked on: AMD Ryzen 9 5900X, 32GB RAM, NVMe SSD

| Metric | Value |
|--------|-------|
| Throughput | ~100 requests/second |
| Latency (p50) | <10ms |
| Latency (p99) | <50ms |
| Buffer efficiency | 99.7% |
| Memory footprint | ~15MB (gateway) |

## Security Considerations

1. **HMAC Secret Key**: Use cryptographically secure 256-bit keys
2. **API Keys**: Rotate regularly, use unique keys per client
3. **TLS/HTTPS**: Always use HTTPS in production
4. **Network Isolation**: Deploy collector in restricted network zone
5. **Rate Limiting**: Adjust limits based on threat model
6. **Monitoring**: Enable metrics and set up alerts

## Configuration Guide

### Collector Tuning

- **fetch_chunk_size**: Balance network overhead vs. latency (1-4KB recommended)
- **buffer_size**: Should cover 5-10 minutes of fetching at peak rate
- **push_interval**: Longer intervals reduce requests but increase latency

### Gateway Tuning

- **buffer_size**: Size based on request patterns (10MB default handles ~10K requests)
- **buffer_ttl**: Set based on acceptable staleness (1 hour typical)
- **rate_limit_per_second**: Protect against abuse while allowing legitimate use

## Randomness Validation

The system includes built-in quality validation:

### PowerShell Test Script (Recommended)

```powershell
# Run comprehensive test with quality metrics display
.\test-randomness.ps1 -GatewayUrl "http://localhost:8080" -ApiKey "your-api-key" -Iterations 1000000

# Verbose mode
.\test-randomness.ps1 -Verbose
```

The script provides:
- System status and health metrics
- Monte Carlo π estimation with quality ratings
- Quantum vs pseudo-random comparison
- Visual quality indicators (★★★★★)
- Detailed interpretation of results

### Manual API Testing

```bash
# Run Monte Carlo π estimation
curl -X POST \
  -H "Authorization: Bearer your-api-key" \
  "https://gateway/api/test/monte-carlo?iterations=10000000"
```

## Docker Deployment

Complete Docker deployment for both components on **separate machines**. See the [Docker Deployment Guide](docs/DOCKER_DEPLOYMENT.md) for detailed instructions.

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/qrng-gateway /usr/local/bin/
COPY config/ /etc/qrng/
EXPOSE 8080
CMD ["qrng-gateway", "--config", "/etc/qrng/gateway.yaml"]
```

```bash
# Build and run
docker build -f Dockerfile.collector -t qrng-entropy-collector:latest .
docker run -d --name entropy-collector \
  -v ./config/collector.yaml:/etc/qrng/collector.yaml:ro \
  qrng-entropy-collector:latest
```

**On External Machine (Gateway):**

```bash
# Build and run
docker build -f Dockerfile.gateway -t qrng-entropy-gateway:latest .
docker run -d --name entropy-gateway -p 8080:8080 \
  -v ./config/gateway.yaml:/etc/qrng/gateway.yaml:ro \
  qrng-entropy-gateway:latest --config /etc/qrng/gateway.yaml
```

### Helper Scripts

```bash
# PowerShell (Windows)
.\deploy.ps1 build-collector
.\deploy.ps1 run-collector

# Bash (Linux/macOS)
./deploy.sh build-gateway
./deploy.sh run-gateway

# Makefile
make build-collector run-collector
make build-gateway run-gateway
```

### Available Files

- `Dockerfile.collector` - Multi-stage build for entropy-collector
- `Dockerfile.gateway` - Multi-stage build for entropy-gateway
- `deploy.ps1` / `deploy.sh` - Deployment scripts
- `Makefile` - Build and run commands
- `.dockerignore` - Build optimization

### Features

- ✅ Multi-stage builds for minimal image size (~50MB)
- ✅ Non-root user execution (security)
- ✅ Built-in health checks
- ✅ Automatic restart policies
- ✅ Volume mounts for configuration
- ✅ Production-ready with optimized builds

## 📚 Publications & Research

This software is designed for academic publication:

- **Target**: SoftwareX journal
- **Novel Contributions**:
  - Software-based data diode emulation for QRNG
  - High-performance Rust implementation with zero-copy buffers
  - MCP integration for AI agent accessibility
  - Built-in randomness quality validation

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.