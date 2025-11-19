# QRNG Gateway PowerShell Scripts

This directory contains PowerShell scripts for testing, benchmarking, and demonstrating the QRNG Gateway. All scripts work on Windows, Linux (via PowerShell Core), and macOS.

## Scripts Overview

### Quick Start & Demos
- **consume-random.ps1**: Interactive demo generating quantum passwords and UUIDs
- **test-randomness.ps1**: Validate quantum randomness quality using Monte Carlo π estimation

### Performance Testing
- **benchmark-simple.ps1**: Basic sustained throughput test (10 minutes)
- **benchmark-burst.ps1**: Maximum throughput test with no throttling (30 seconds)
- **benchmark-performance.ps1**: Comprehensive benchmark suite with multiple test scenarios

## Prerequisites

- PowerShell 5.1+ (Windows) or PowerShell Core 7+ (cross-platform)
- Network access to a QRNG Gateway instance

## Quick Start

### Test the Public Demo Gateway

```powershell
# Try the live demo gateway (EU-hosted)
.\consume-random.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"

# Run randomness quality test
.\test-randomness.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"

# Quick 30-second burst test
.\benchmark-burst.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"
```

### Test Your Local Gateway

```powershell
# Generate passwords and UUIDs continuously
.\consume-random.ps1

# Validate randomness quality (waits for full buffer)
.\test-randomness.ps1

# Simple sustained throughput test (10 minutes)
.\benchmark-simple.ps1 -DurationSeconds 600

# Quick burst test (30 seconds)
.\benchmark-burst.ps1

# Full benchmark suite (default: 10 minutes)
.\benchmark-performance.ps1 -Duration 600 -Clients 10
```

## Script Details

### consume-random.ps1

Interactive demonstration that continuously generates quantum-random passwords and UUIDs.

**Usage:**
```powershell
.\consume-random.ps1 [-GatewayUrl <url>] [-ApiKey <key>]
```

**Default behavior:**
- Connects to `http://localhost:7764`
- Generates 3 passwords (20 characters) and 5 UUIDs every 2 seconds
- Displays results in color-coded output
- Press Ctrl+C to stop

**Example:**
```powershell
.\consume-random.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"
```

### test-randomness.ps1

Waits for the buffer to fill to 100% capacity, then performs the maximum possible Monte Carlo π estimation test to validate randomness quality.

**Usage:**
```powershell
.\test-randomness.ps1 [-GatewayUrl <url>] [-ApiKey <key>] [-PollIntervalSeconds <seconds>] [-Verbose]
```

**Features:**
- Polls buffer status until 100% full
- Calculates maximum iterations based on buffer capacity
- Consumes entire buffer for most rigorous test
- Displays convergence quality (excellent/good/fair/poor)
- Shows comparison with pseudo-random (statistical only)

**Example:**
```powershell
.\test-randomness.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"
```

### benchmark-simple.ps1

Basic sustained throughput test with configurable duration and think time.

**Usage:**
```powershell
.\benchmark-simple.ps1 [-GatewayUrl <url>] [-ApiKey <key>] [-DurationSeconds <sec>] [-RequestSize <bytes>] [-ThinkTimeMs <ms>]
```

**Parameters:**
- `DurationSeconds`: Test duration (default: 600 = 10 minutes)
- `RequestSize`: Bytes per request (default: 1024)
- `ThinkTimeMs`: Delay between requests (default: 50ms)

**Metrics:**
- Throughput (requests/second)
- Success rate
- Latency distribution (P50, P75, P90, P95, P99)
- Data rate (KB/s)
- Error summary

**Example:**
```powershell
# 5-minute test with 100ms think time
.\benchmark-simple.ps1 -DurationSeconds 300 -ThinkTimeMs 100
```

### benchmark-burst.ps1

Quick maximum throughput test with no throttling.

**Usage:**
```powershell
.\benchmark-burst.ps1 [-GatewayUrl <url>] [-ApiKey <key>] [-DurationSeconds <sec>] [-RequestSize <bytes>]
```

**Features:**
- No think time - maximum request rate
- 30-second default duration
- Measures burst capacity
- Quick latency statistics

**Example:**
```powershell
.\benchmark-burst.ps1 -DurationSeconds 60
```

### benchmark-performance.ps1

Comprehensive performance benchmark suite with multiple test scenarios.

**Usage:**
```powershell
.\benchmark-performance.ps1 [-GatewayUrl <url>] [-ApiKey <key>] [-Duration <sec>] [-Clients <count>] [-RequestSize <bytes>]
```

**Test Scenarios:**
1. Single-client baseline (60 seconds)
2. Concurrent clients (configurable duration and client count)
3. Final Prometheus metrics snapshot

**Parameters:**
- `Duration`: Main test duration (default: 600 = 10 minutes)
- `Clients`: Concurrent clients (default: 10)
- `RequestSize`: Bytes per request (default: 1024)

**Output:**
- Creates timestamped results directory
- Saves JSON results for each test
- Generates summary report
- Captures Prometheus metrics

**Example:**
```powershell
# 30-minute test with 20 concurrent clients
.\benchmark-performance.ps1 -Duration 1800 -Clients 20 -RequestSize 2048
```

## Common Parameters

All scripts support these parameters:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `GatewayUrl` | `http://localhost:7764` | Gateway endpoint URL |
| `ApiKey` | `test-key-1234567890` | API authentication key |

## Examples

### Testing the Public Demo Gateway

```powershell
# Interactive demo with public gateway
.\consume-random.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"

# Validate randomness quality
.\test-randomness.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"

# Quick performance check (30 seconds)
.\benchmark-burst.ps1 -GatewayUrl "https://qrng.dataman.ro" -ApiKey "test-key-1234567890"
```

### Testing Local Development Gateway

```powershell
# Start with interactive demo
.\consume-random.ps1

# Run quality validation
.\test-randomness.ps1

# Performance baseline (10 minutes)
.\benchmark-simple.ps1

# Full benchmark suite
.\benchmark-performance.ps1 -Duration 600
```

### Custom Gateway Deployment

```powershell
$url = "https://qrng.mycompany.com"
$key = "my-production-api-key"

.\test-randomness.ps1 -GatewayUrl $url -ApiKey $key
.\benchmark-simple.ps1 -GatewayUrl $url -ApiKey $key -DurationSeconds 3600
```

## Interpreting Results

### Randomness Quality (test-randomness.ps1)

- **Excellent** (< 0.01% error): High-quality quantum entropy, suitable for all applications
- **Good** (< 0.1% error): Suitable for most cryptographic applications
- **Fair** (< 1.0% error): Acceptable but consider more iterations
- **Poor** (≥ 1.0% error): Investigate entropy source issues

### Performance Metrics

- **Throughput**: Sustained requests/second (typically 25-30 req/s limited by QRNG hardware)
- **Burst Capacity**: Peak requests/second (400+ req/s from buffer)
- **Latency P99**: 99th percentile latency (should be < 10ms for local gateway)
- **Success Rate**: Should be 100% when buffer is properly maintained

## Troubleshooting

### "Gateway is not accessible"
- Check that the gateway is running: `Invoke-RestMethod http://localhost:7764/health`
- Verify network connectivity and firewall settings
- For remote gateways, check URL and port

### "Buffer critically low" or 503 errors
- Wait for the collector to fill the buffer (30-60 seconds)
- Check collector is running and connected to QRNG appliance
- Monitor buffer status: `Invoke-RestMethod http://localhost:7764/api/status -Headers @{ Authorization = "Bearer test-key-1234567890" }`

### "Unauthorized" or 401 errors
- Verify API key matches gateway configuration
- Check both query parameter and Authorization header methods

### Poor randomness quality results
- Ensure you're using actual quantum entropy source
- Check for QRNG appliance connection issues
- Verify data mixing configuration if using multiple sources

## Notes

- All scripts default to `http://localhost:7764` for local testing
- The public demo gateway `https://qrng.dataman.ro` is available for quick testing
- Demo API key `test-key-1234567890` is for testing only
- For production deployments, use secure API keys and HTTPS
- Buffer fill time depends on QRNG hardware entropy generation rate
- Benchmark results vary based on network latency and buffer state

## See Also

- [Rust Examples](../examples/README.md) - Rust-based example applications
- [Architecture Documentation](../docs/ARCHITECTURE.md) - System design details
- [Performance Benchmarks](../docs/BENCHMARK.md) - Detailed performance analysis
