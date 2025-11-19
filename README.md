# QRNG Data Diode: High-Performance Quantum Entropy Bridge

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/Docker-Supported-blue?logo=docker)](https://docker.com)
[![Build Status](https://github.com/vbocan/qrng-data-diode/workflows/Build%20and%20Push%20Docker%20Images/badge.svg)](https://github.com/vbocan/qrng-data-diode/actions/workflows/docker-build.yml)
[![API](https://img.shields.io/badge/API-REST-orange)](http://localhost:7764/swagger)
[![MCP](https://img.shields.io/badge/MCP-Protocol-purple)](https://modelcontextprotocol.io)

## Overview

A high-performance, secure bridge service that exposes Quantum Random Number Generator (QRNG) entropy to external networks using software-based data diode emulation. Designed for academic research, cryptographic applications, and scientific computing with true quantum randomness.

![Quantis QRNG Appliance](docs/images/quantis-appliance.png)
*IDQuantique Quantis QRNG Appliance - Hardware quantum entropy source*

## Quick Start - Try the Live Service

### Live QRNG Gateway

Get quantum random data instantly from our live gateway at **https://qrng.dataman.ro**:

```bash
# Get 32 bytes of quantum random data (hex encoded)
curl "https://qrng.dataman.ro/api/random?bytes=32&api_key=test-key-1234567890"

# Get 16 bytes in base64 format
curl "https://qrng.dataman.ro/api/random?bytes=16&encoding=base64&api_key=test-key-1234567890"

# Get 16 random integers between 0-100
curl "https://qrng.dataman.ro/api/integers?count=16&min=0&max=100&api_key=test-key-1234567890"

# Get random floats between 0 and 1
curl "https://qrng.dataman.ro/api/floats?count=10&api_key=test-key-1234567890"

# Check service health (returns HTTP 200 if healthy)
curl "https://qrng.dataman.ro/health"

# Get detailed service status (query parameter)
curl "https://qrng.dataman.ro/api/status?api_key=test-key-1234567890"

# Or use Authorization header (all authenticated endpoints support both methods)
curl -H "Authorization: Bearer test-key-1234567890" "https://qrng.dataman.ro/api/status"
```

**Note**: The demo API key `test-key-1234567890` is for testing purposes. For production use, deploy your own instance.

### MCP Integration with Claude Desktop

Add quantum randomness to Claude Desktop (requires Claude Pro):

1. Open Claude Desktop
2. Go to **Settings** → **Connectors**
3. Click **Add Connector**
4. Enter the MCP server URL: `https://qrng-mcp.datamana.ro`
5. Save and the QRNG tools will be available in your conversations

### LM Studio MCP Integration

Configure LM Studio to use the QRNG MCP server:

1. Open LM Studio
2. Go to **Integrations** dialog
3. Add the MCP server URL: `https://qrng-mcp.datamana.ro`
4. The QRNG tools will be available to your LLM

Supported MCP tools:
- `get_random_bytes`: Get raw quantum random bytes
- `get_random_integers`: Generate random integers in range
- `get_random_hex`: Get hex-encoded random data
- `get_random_base64`: Get base64-encoded random data

## Quick Start with Docker (Self-Hosted)

### Prerequisites

> **⚠️ CRITICAL REQUIREMENT**: You must have access to a **Quantis QRNG hardware appliance** or API-compatible quantum entropy source. This software acts as a bridge and cannot generate quantum randomness by itself.

- Docker and Docker Compose
- OpenSSL (for key generation)

### Generate HMAC Secret Key

```bash
openssl rand -hex 32
```

### Configuration

Create configuration files from examples:

```bash
# Clone the repository
git clone https://github.com/vbocan/qrng-data-diode.git
cd qrng-data-diode

# Copy example configs
cp qrng-collector/.env.example qrng-collector/.env
cp qrng-gateway/.env.example qrng-gateway/.env

# Edit with your settings
# - Set QRNG_APPLIANCE_URLS to your Quantis endpoint
# - Set QRNG_HMAC_SECRET_KEY to generated key
# - Set QRNG_API_KEY for gateway authentication
```

### Start Services

```bash
docker-compose up -d
```

That's it! The system will start collecting quantum entropy and serving it via API.

### Access the Services

Once running, the gateway provides the following endpoints:

**API Endpoints** (all require `api_key` parameter or `Authorization: Bearer <key>` header):
- **Random Bytes**: http://localhost:7764/api/random?bytes=32&api_key=YOUR_API_KEY
- **Random Integers**: http://localhost:7764/api/integers?count=10&min=0&max=100&api_key=YOUR_API_KEY
- **Random Floats**: http://localhost:7764/api/floats?count=10&api_key=YOUR_API_KEY
- **UUID Generation**: http://localhost:7764/api/uuid?count=5&api_key=YOUR_API_KEY
- **System Status**: http://localhost:7764/api/status?api_key=YOUR_API_KEY
- **Monte Carlo Test**: http://localhost:7764/api/test/monte-carlo?iterations=1000000&api_key=YOUR_API_KEY (POST)

**Monitoring Endpoints** (no authentication required):
- **Health Check**: http://localhost:7764/health
- **Prometheus Metrics**: http://localhost:7764/metrics

### Running Tests

```bash
# Run all tests
cargo test --all

# Run specific component tests
cargo test -p qrng-collector
cargo test -p qrng-gateway
cargo test -p qrng-mcp
```

## Key Features

- **Software Data Diode**: Unidirectional entropy flow without expensive hardware ($5K-$50K saved)
- **Ultra-Low Latency**: Sub-4ms median, sub-10ms P99 (100x faster than public QRNG services)
- **High Reliability**: 100% success rate when properly configured
- **Configurable Buffer Policy**: Discard or replace mode for handling buffer overflow scenarios
- **AI Integration**: MCP server for quantum randomness - works with Claude Desktop and compatible agents
- **Multi-Source Aggregation**: Combine multiple QRNG appliances with XOR or HKDF mixing
- **Cryptographic Integrity**: HMAC-SHA256 authentication + CRC32 checksums
- **Production Ready**: Prometheus metrics, structured logging, health checks, Docker deployment
- **Quality Validation**: Built-in Monte Carlo π estimation for randomness verification

## Performance Metrics

- **Sustained Throughput**: 28.74 req/s (limited by QRNG appliance entropy generation)
- **Burst Capability**: 400+ req/s (short-term peak performance)
- **Latency**: P50 = 3.62ms, P95 = 6.89ms, P99 = 9.13ms
- **Reliability**: 100% success rate over 10-minute sustained test
- **Quality**: Monte Carlo π error <0.0002% (10M iterations)
- **Comparison**: 6-124x faster latency than ANU QRNG, NIST Beacon
- **Scaling**: Linear throughput scaling with multiple QRNG appliances

See [Performance Benchmarks](docs/BENCHMARK.md) for detailed methodology, results, and analysis.

## Technology Stack

- **Language**: Rust 1.75+
- **Async Runtime**: Tokio 1.35
- **HTTP Server**: Axum 0.7
- **Cryptography**: HMAC-SHA256, CRC32
- **Metrics**: Prometheus
- **Logging**: Structured JSON with tracing
- **Deployment**: Docker, Docker Compose

## Citation

If you use QRNG-DD in your research, please cite:

```bibtex
@software{qrngdd2025,
  title = {QRNG-DD: A High-Performance Rust Implementation of Software-Based Data Diode Architecture for Quantum Random Number Distribution with AI Agent Integration},
  author = {Valer Bocan, PhD, CSSLP},
  year = {2025},
  version = {1.0.0},
  url = {https://github.com/vbocan/qrng-data-diode},
  license = {MIT}
}
```

See [CITATION.cff](CITATION.cff) for structured citation metadata.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Documentation

- **Architecture**: Detailed technical architecture in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **MCP Integration**: AI agent integration guide in [docs/MCP-INTEGRATION.md](docs/MCP-INTEGRATION.md)
- **Security Analysis**: Comprehensive security architecture in [docs/SECURITY-ANALYSIS.md](docs/SECURITY-ANALYSIS.md)
- **Performance Benchmarks**: Detailed performance testing and analysis in [docs/BENCHMARK.md](docs/BENCHMARK.md)

## Support & Contact

**Author**: Valer Bocan, PhD, CSSLP  
**Affiliation**: Department of Computer and Information Technology, Politehnica University of Timișoara  

- **Issues**: Report bugs and request features via [GitHub Issues](https://github.com/vbocan/qrng-data-diode/issues)
- **Discussions**: Community support via [GitHub Discussions](https://github.com/vbocan/qrng-data-diode/discussions)
- **Email**: valer.bocan@upt.ro

---

**Version**: 1.0.0  
**Status**: Active Development  
**Repository**: https://github.com/vbocan/qrng-data-diode