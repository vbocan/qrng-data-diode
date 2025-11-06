# QRNG Data Diode: Software-Based Quantum Entropy Bridge

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A high-performance, secure bridge service that exposes Quantum Random Number Generator (QRNG) entropy to external networks using software-based data diode emulation. Implemented in Rust for maximum safety, performance, and reliability.

## 🎯 Overview

This system provides secure access to quantum-generated random numbers from a locally-networked Quantis QRNG appliance, designed for academic research, cryptographic applications, and scientific computing. It addresses network restrictions through a split architecture that emulates unidirectional data flow, inspired by hardware data diodes.

### Key Features

- **🔐 Software Data Diode**: Unidirectional entropy flow from internal to external networks
- **🚀 High Performance**: Lock-free buffers, zero-copy operations, async I/O
- **🛡️ Cryptographic Integrity**: HMAC-SHA256 signing + CRC32 checksums
- **📊 Production Ready**: Comprehensive metrics, structured logging, health checks
- **🤖 AI Integration**: Model Context Protocol (MCP) server for AI agents
- **🧪 Quality Validation**: Built-in Monte Carlo π estimation for randomness verification
- **📦 Flexible Deployment**: Push-based (data diode) or direct access modes

## 🏗️ Architecture

### Push-Based Mode (Data Diode Emulation)

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
│  │    Entropy       │   │  HTTPS  │  │    Entropy       │   │
│  │   Collector      │───┼────────>│──│    Gateway       │   │
│  │                  │   │  push   │  │                  │   │
│  │  - Fetch loop    │   │  (one-  │  │  - REST API      │   │
│  │  - Buffer (1MB)  │   │   way)  │  │  - Buffer (10MB) │   │
│  │  - HMAC signing  │   │         │  │  - MCP Server    │   │
│  └──────────────────┘   │         │  │  - Metrics       │   │
│                         │         │  └──────────────────┘   │
└─────────────────────────┘         └─────────────────────────┘
```

### Direct Access Mode (Simplified)

```
┌──────────────────────────────────┐
│        Trusted Network           │
│                                  │
│  ┌──────────────────┐            │
│  │  Quantis QRNG    │            │
│  │    Appliance     │            │
│  └────────┬─────────┘            │
│           │ HTTPS                │
│           │ fetch                │
│           ▼                      │
│  ┌──────────────────┐            │
│  │    Entropy       │◄───────────┼──── Clients (REST API)
│  │    Gateway       │            │
│  │                  │            │
│  │  - Fetch loop    │            │
│  │  - Buffer (10MB) │            │
│  │  - REST API      │            │
│  │  - MCP Server    │            │
│  └──────────────────┘            │
└──────────────────────────────────┘
```

## 🚀 Quick Start

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

Edit `config/collector.yaml`:

```yaml
appliance_url: "https://your-qrng-appliance.example.com/random"
push_url: "https://your-gateway.example.com/push"
hmac_secret_key: "<your-generated-key>"
```

#### 3. Configure Entropy Gateway

Edit `config/gateway-push.yaml`:

```yaml
listen_address: "0.0.0.0:8080"
api_keys:
  - "your-secure-api-key"
hmac_secret_key: "<same-key-as-collector>"
```

### Running

#### Push-Based Mode

```bash
# Terminal 1: Start Entropy Gateway (external network)
./target/release/entropy-gateway --config config/gateway-push.yaml

# Terminal 2: Start Entropy Collector (internal network)
./target/release/entropy-collector --config config/collector.yaml
```

#### Direct Access Mode

```bash
# Single component deployment
./target/release/entropy-gateway --config config/gateway-direct.yaml
```

## 📡 API Reference

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
  "deployment_mode": "push",
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
  "iterations": 1000000,
  "convergence_rate": "excellent",
  "quantum_vs_pseudo": {
    "quantum_error": 0.000005,
    "pseudo_error": 0.000023,
    "improvement_factor": 4.6
  }
}
```

## 🤖 MCP Server Integration

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
      "command": "entropy-gateway",
      "args": ["--mcp-mode", "--config", "config/gateway.yaml"]
    }
  }
}
```

## 🛠️ Development

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
├── entropy-collector/      # Internal component
│   ├── src/main.rs
│   └── Cargo.toml
├── entropy-gateway/        # External component
│   ├── src/main.rs
│   └── Cargo.toml
├── qrng-mcp/              # MCP server implementation
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

## 📊 Performance

Benchmarked on: AMD Ryzen 9 5900X, 32GB RAM, NVMe SSD

| Metric | Value |
|--------|-------|
| Throughput | ~100 requests/second |
| Latency (p50) | <10ms |
| Latency (p99) | <50ms |
| Buffer efficiency | 99.7% |
| Memory footprint | ~15MB (gateway) |

## 🔒 Security Considerations

1. **HMAC Secret Key**: Use cryptographically secure 256-bit keys
2. **API Keys**: Rotate regularly, use unique keys per client
3. **TLS/HTTPS**: Always use HTTPS in production
4. **Network Isolation**: Deploy collector in restricted network zone
5. **Rate Limiting**: Adjust limits based on threat model
6. **Monitoring**: Enable metrics and set up alerts

## 📝 Configuration Guide

### Collector Tuning

- **fetch_chunk_size**: Balance network overhead vs. latency (1-4KB recommended)
- **buffer_size**: Should cover 5-10 minutes of fetching at peak rate
- **push_interval**: Longer intervals reduce requests but increase latency

### Gateway Tuning

- **buffer_size**: Size based on request patterns (10MB default handles ~10K requests)
- **buffer_ttl**: Set based on acceptable staleness (1 hour typical)
- **rate_limit_per_second**: Protect against abuse while allowing legitimate use

## 🧪 Randomness Validation

The system includes built-in quality validation:

```bash
# Run Monte Carlo π estimation
curl -X POST "https://gateway/api/test/monte-carlo?iterations=10000000"

# External validation with DIEHARDER
curl "https://gateway/api/random?bytes=10485760&encoding=binary" > random.bin
dieharder -a -g 201 -f random.bin

# NIST Statistical Test Suite
curl "https://gateway/api/random?bytes=1048576&encoding=binary" > data.bin
sts -f data.bin
```

## 🐳 Docker Deployment

Complete Docker deployment for both components on **separate machines**. See the [Docker Deployment Guide](docs/DOCKER_DEPLOYMENT.md) for detailed instructions.

### Architecture

Each component runs on its own physical machine:
- **Machine A (Internal)**: `entropy-collector` → connects to QRNG appliance
- **Machine B (External)**: `entropy-gateway` → serves clients
- **Data Flow**: One-way push from collector to gateway

### Quick Start

**On Internal Machine (Collector):**

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
  -v ./config/gateway-push.yaml:/etc/qrng/gateway.yaml:ro \
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

## 🤝 Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Quantis QRNG appliance by ID Quantique
- Rust async ecosystem (Tokio, Axum)
- Model Context Protocol by Anthropic

## 📧 Contact

For questions or support:
- Open an issue on GitHub
- Email: [your-email]
- Documentation: [your-docs-site]

---

**Built with ❤️ and Rust**

*"True randomness from the quantum realm, securely delivered."*
