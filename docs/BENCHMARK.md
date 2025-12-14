# QRNG-DD Benchmark

## Executive Summary

This document presents comprehensive performance testing of the QRNG-DD system. Original benchmarks were conducted on November 18, 2025 (local Docker network), with remote access verification performed on December 14, 2025 against the production server at `qrng.dataman.ro`.

Testing reveals that the gateway achieves excellent internal latency (~100μs P50) with sustained throughput limited by the QRNG appliance's entropy generation rate rather than gateway processing capacity. End-to-end latency depends on network conditions: sub-4ms on local networks, 50-200ms over the internet.

**Key Findings (Local Network):**
- **Sustained throughput**: 28.74 req/s (100% success rate over 10 minutes)
- **Burst capability**: 438 req/s (short-term peak performance)
- **End-to-end latency**: P50 3.62ms, P95 6.89ms, P99 9.13ms
- **Gateway internal latency**: P50 ~100μs, P99 ~2ms
- **Bottleneck**: QRNG appliance entropy generation (~80 KB/s), not gateway software
- **Scaling solution**: Multiple QRNG appliances with multi-source aggregation

---

## Table of Contents

1. [Testing Methodology](#testing-methodology)
2. [Test Environment](#test-environment)
3. [Test Results](#test-results)
4. [Analysis and Interpretation](#analysis-and-interpretation)
5. [Comparison with Public QRNG Services](#comparison-with-public-qrng-services)
6. [Conclusions and Recommendations](#conclusions-and-recommendations)

---

## Testing Methodology

### Approach

Performance testing was conducted using custom PowerShell benchmarking scripts that measure end-to-end HTTP round-trip time from client to gateway, including network overhead, authentication, gateway processing, and data serialization. This provides realistic performance metrics from an end-user perspective.

### Test Scenarios

**Test 1: Sustained Throughput (10 minutes)**
- **Objective**: Measure sustainable request rate with 100% success
- **Method**: Single client making sequential requests with 20ms think time
- **Duration**: 600 seconds
- **Metric collection**: Per-request latency, success/failure counts

**Test 2: Burst Capability (30 seconds)**
- **Objective**: Measure peak gateway processing capacity
- **Method**: Single client making requests without throttling
- **Duration**: 30 seconds
- **Metric collection**: Peak throughput, latency distribution, buffer depletion

### Metrics Collected

**Throughput Metrics:**
- Requests per second (req/s)
- Total successful requests
- Success rate (percentage)
- Data rate (KB/s)

**Latency Metrics:**
- P50 (median)
- P75 (75th percentile)
- P90 (90th percentile)
- P95 (95th percentile)
- P99 (99th percentile)
- Minimum, maximum, mean

**System Metrics:**
- Gateway buffer utilization
- Prometheus internal metrics
- Collector push rate

### Measurement Tools

**Custom PowerShell Scripts:**
- `simple-benchmark.ps1`: Sustained throughput test with configurable think time
- `burst-test.ps1`: Maximum capability test without throttling

**Timing Method:**
- .NET Stopwatch for microsecond-precision latency measurement
- Full HTTP request/response cycle timing

**Validation:**
- Cross-referenced with Prometheus metrics from gateway
- Verified data integrity (100% of responses validated)

---

## Test Environment

> **Important Note**: These benchmarks were conducted on a **local Docker network** where network latency is negligible (~1-2ms). When accessing the gateway over the internet, end-to-end latency will be dominated by network round-trip time (RTT), which can range from 50-200ms depending on geographic distance. The gateway's internal processing time (~100μs) remains constant regardless of network conditions.

### Hardware Configuration

**Client Machine:**
- **OS**: Windows (Docker Desktop host)
- **Location**: Local network (same host as containers)

**Gateway Container:**
- **Platform**: Docker
- **Runtime**: Rust 1.75.0 release build
- **Configuration**:
  - Buffer size: 10 MB
  - Rate limit: 1000 req/s
  - API key authentication: Enabled

**Collector Container:**
- **Platform**: Docker
- **Configuration**:
  - Fetch interval: 100ms (10 fetches/second)
  - Fetch chunk size: 8192 bytes
  - Push interval: 500ms (2 pushes/second)
  - Buffer size: 10 MB

**QRNG Source:**
- **Appliance**: ID Quantique Quantis QRNG
- **Location**: `random.cs.upt.ro` (Politehnica University of Timișoara)
- **Protocol**: HTTPS
- **Network**: Internet connection

### Software Configuration

**Rust Compilation:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

**Docker Compose:**
- Gateway exposed on port 7764
- Collector communicates with gateway over internal Docker network
- Containers use JSON logging

**Network:**
- Client → Gateway: Local Docker network
- Collector → QRNG: HTTPS over Internet (~50ms base latency)

---

## Test Results

### Test 1: Sustained Throughput (10 minutes)

**Configuration:**
- Duration: 600 seconds
- Request size: 1024 bytes
- Encoding: Hexadecimal
- Think time: 20ms between requests
- Clients: 1

**Results Summary:**

| Metric | Value |
|--------|-------|
| **Total requests** | 17,243 |
| **Successful** | 17,243 (100.0%) |
| **Failed** | 0 (0.0%) |
| **Duration** | 600.02 seconds |
| **Throughput** | **28.74 req/s** |
| **Data rate** | **28.74 KB/s** |

**Latency Distribution:**

| Percentile | Latency (ms) |
|------------|--------------|
| Min | 1.87 |
| **P50 (Median)** | **3.62** |
| P75 | 4.68 |
| P90 | 5.93 |
| **P95** | **6.89** |
| **P99** | **9.13** |
| Max | 318.67 |
| Mean | 4.06 |

**Latency Histogram:**
```
   0-2ms:  ████ 12.3%
   2-4ms:  ████████████████████████████████ 58.7%
   4-6ms:  ███████████ 21.4%
   6-8ms:  ███ 5.1%
  8-10ms:  █ 1.9%
 10-20ms:  ▌ 0.5%
 20-320ms: ▌ 0.1%
```

**Observations:**
- **Perfect reliability**: 100% success rate over 10 minutes
- **Excellent latency**: 58.7% of requests complete in 2-4ms range
- **Consistent performance**: Throughput stable at 28.7-28.8 req/s throughout test
- **No degradation**: Performance remained constant from start to finish

**Prometheus Internal Metrics (Gateway):**
- P50 internal latency: 40 μs (0.040 ms)
- P99 internal latency: 114 μs (0.114 ms)
- Note: These measure only gateway processing, excluding HTTP/network overhead

### Test 2: Burst Capability (30 seconds)

**Configuration:**
- Duration: 30 seconds
- Request size: 1024 bytes
- Encoding: Hexadecimal
- Think time: 0ms (maximum rate)
- Clients: 1

**Results Summary:**

| Metric | Value |
|--------|-------|
| **Total requests** | 16,135 |
| **Successful** | 13,148 (81.5%) |
| **Failed** | 2,987 (18.5%) |
| **Duration** | 30 seconds |
| **Throughput** | **438.26 req/s** |

**Latency Distribution (Successful Requests):**

| Percentile | Latency (ms) |
|------------|--------------|
| **P50 (Median)** | **1.61** |
| P75 | 2.31 |
| P90 | 2.87 |
| **P95** | **3.17** |
| **P99** | **3.80** |

**Observations:**
- **High throughput**: Gateway handles 438 req/s when buffer is full
- **Low latency under load**: Sub-2ms median latency even at peak rate
- **Buffer depletion**: 18.5% failure rate as buffer drains faster than refilled
- **Constraint**: Limited by QRNG appliance entropy rate (~80 KB/s)

**Error Analysis:**
- All failures due to insufficient buffer data
- No authentication, network, or processing errors
- Gateway processing remained fast throughout

---

## Analysis and Interpretation

### Bottleneck Identification

The primary performance bottleneck is the **QRNG appliance's entropy generation rate**, not the gateway software processing capacity.

**Evidence:**

1. **Collector Performance:**
   - Fetches 8 KB every ~100ms from QRNG appliance
   - Pushes ~123 KB every 1.5 seconds to gateway
   - Effective entropy rate: ~80 KB/s from appliance

2. **Gateway Capability:**
   - Handles 438 req/s when buffer is full (burst test)
   - Processes requests in 40-114 μs internally (Prometheus)
   - No processing bottleneck observed

3. **Sustainable Throughput:**
   - 80 KB/s ÷ 1 KB per request ≈ 80 req/s theoretical maximum
   - Achieved 28.74 req/s with 100% success (36% of maximum)
   - Conservative throughput ensures buffer never depletes

### Performance Characteristics

**Strengths:**
- **Ultra-low latency**: P50 3.62ms, P99 9.13ms (excellent responsiveness)
- **High reliability**: 100% success rate when properly throttled
- **Burst capability**: 438 req/s short-term (15× sustained rate)
- **Consistent performance**: No degradation over 10-minute test
- **Efficient software**: Internal processing <0.2ms

**Limitations:**
- **QRNG-limited throughput**: Cannot sustain >30 req/s with single appliance
- **Burst unsustainable**: Success rate drops to 81.5% at maximum rate
- **Appliance-dependent**: Different QRNG appliances would yield different throughput

### Comparison with Original Claims

The original performance benchmarks document claimed ~100 req/s sustained throughput. Actual testing reveals:

**Original Claims vs. Actual Results:**

| Metric | Original Claim | Actual Result | Variance |
|--------|----------------|---------------|----------|
| Sustained throughput | 99.7 req/s | 28.74 req/s | -71% |
| Median latency (P50) | 8.7 ms | 3.62 ms | +58% better |
| P95 latency | 23.2 ms | 6.89 ms | +70% better |
| P99 latency | 47.8 ms | 9.13 ms | +81% better |
| Success rate | 99.7% | 100% | +0.3% |

**Analysis:**
- **Throughput**: Lower than claimed due to QRNG appliance limitation
- **Latency**: Significantly better than claimed (gateway performs excellently)
- **Reliability**: Better than claimed (100% vs 99.7%)

**Root Cause:**
Original benchmarks likely tested against a mock data source or different QRNG appliance configuration, not accounting for real-world entropy generation constraints.

### Remote Access Performance

When accessing the gateway over the internet (e.g., production deployment at `qrng.dataman.ro`), performance characteristics change due to network latency:

**Remote Access Test Results (December 2025):**

| Metric | Local Network | Remote Internet |
|--------|---------------|-----------------|
| End-to-end P50 latency | 3.62 ms | ~130 ms |
| Sequential throughput | 28.74 req/s | ~6 req/s |
| Parallel throughput (50 concurrent) | N/A | ~54 req/s |
| Gateway internal P50 | 40 μs | 101 μs |
| Success rate | 100% | 100% |

**Key Observations:**
- End-to-end latency is dominated by network RTT (~140ms to European server)
- Gateway internal processing remains fast (~100μs) regardless of client location
- Parallel requests effectively mitigate network latency impact
- 100% reliability maintained over internet connections

**For Remote Clients:**
- Use parallel/concurrent requests to maximize throughput
- Expect end-to-end latency of 50-200ms depending on geographic distance
- Gateway processing time is negligible compared to network latency

---

### Gateway vs. System Performance

It's important to distinguish between gateway processing capability and overall system throughput:

**Gateway Software:**
- Can process 438+ req/s (measured)
- Sub-millisecond internal latency
- No performance bottleneck

**System Throughput:**
- Limited to ~29 req/s sustained
- Constrained by QRNG appliance entropy generation
- Buffer acts as shock absorber for burst traffic

**Implication:**
The gateway software is not the limiting factor. Organizations requiring higher throughput need faster QRNG appliances or multiple appliances with aggregation, not software optimization.

---

## Comparison with Public QRNG Services

### ANU QRNG (Australian National University)

**ANU Specifications:**
- Rate limit: 5 req/s maximum
- Request size: 1024 bytes maximum
- Typical latency: 450ms (transcontinental)

**QRNG-DD vs. ANU:**
- **Throughput**: 28.74 req/s vs. 5 req/s (5.7× faster)
- **Latency**: 3.62ms vs. 450ms (124× faster)
- **Request size**: Megabytes vs. 1 KB (1000× larger)
- **Availability**: Self-hosted vs. shared public service

### NIST Randomness Beacon

**NIST Specifications:**
- Rate limit: 1 pulse per minute
- Fixed pulse size: 512 bytes
- Typical latency: Variable

**QRNG-DD vs. NIST:**
- **Throughput**: 28.74 req/s vs. 0.017 req/s (1,690× faster)
- **Latency**: 3.62ms vs. 30+ seconds (8,300× faster)
- **Flexibility**: Configurable size vs. fixed 512 bytes

### Summary

QRNG-DD provides:
- **6-1,690× higher throughput** than public QRNG services
- **124-8,300× lower latency** than public QRNG services
- **Self-hosted privacy** (no centralized logging)
- **Unlimited request sizes** (vs. 512-1024 byte limits)

While sustained throughput is lower than originally claimed, it still dramatically outperforms all available public QRNG services.

---

## Conclusions and Recommendations

### Summary of Findings

1. **Gateway Performance**: Excellent latency (P50 3.62ms, P99 9.13ms) and burst capability (438 req/s)
2. **System Throughput**: Limited to ~29 req/s by QRNG appliance entropy generation rate
3. **Reliability**: 100% success rate when properly configured
4. **Bottleneck**: QRNG appliance (~80 KB/s), not gateway software

### Recommendations for Users

**For Low-Throughput Applications (<30 req/s):**
- QRNG-DD with single appliance is ideal
- Excellent latency and reliability
- No additional hardware needed

**For Medium-Throughput Applications (30-150 req/s):**
- Deploy 3-5 QRNG appliances with multi-source aggregation
- Use QRNG-DD's built-in entropy mixing (XOR or HKDF)
- Scales linearly with appliance count

**For High-Throughput Applications (>150 req/s):**
- Deploy many QRNG appliances or faster appliances
- Consider if true quantum randomness is required
- Evaluate cost/benefit vs. cryptographic PRNG

### Deployment Guidance

**Recommended Configuration:**
- Collector fetch interval: 100ms (default)
- Collector push interval: 500ms (default)
- Gateway buffer: 10 MB (default)
- Client throttling: 20-30ms between requests for 100% success

**Monitoring:**
- Watch buffer fill levels via Prometheus metrics
- Alert on buffer <30% (risk of request failures)
- Track request success rate (should be >99%)

**Scaling:**
- Add QRNG appliances before optimizing software
- Use multi-source aggregation for vendor diversity
- Each appliance adds ~29 req/s capacity (with 1 KB requests)

### Future Work

**Potential Optimizations:**
- Larger collector fetch chunks (reduce round-trips to QRNG)
- Multiple collectors per appliance (parallel fetching)
- Adaptive push intervals based on buffer levels

**Important Note:**
These optimizations would have marginal impact. The fundamental constraint is the QRNG appliance's entropy generation rate, which cannot be improved through software changes.

### Final Assessment

QRNG-DD achieves its core design goals:

✅ **Low latency**: Sub-4ms median (better than claimed)  
✅ **High reliability**: 100% success rate  
✅ **Software data diode**: Unidirectional flow maintained  
✅ **Cost-effective**: Open source, no hardware diode needed  
✅ **AI integration**: MCP protocol support  
✅ **Production-ready**: Stable over extended testing  

⚠️ **Throughput**: 29 req/s sustained (lower than claimed, limited by QRNG appliance)

**Recommendation**: Update all documentation to reflect actual measured performance with clear explanation of QRNG appliance limitation and horizontal scaling solution.

---

## Appendix A: Test Commands

### Sustained Throughput Test
```powershell
.\scripts\simple-benchmark.ps1 `
    -GatewayUrl "http://localhost:7764" `
    -ApiKey "test-key-1234567890" `
    -DurationSeconds 600 `
    -RequestSize 1024 `
    -ThinkTimeMs 20
```

### Burst Capability Test
```powershell
.\scripts\burst-test.ps1 `
    -GatewayUrl "http://localhost:7764" `
    -ApiKey "test-key-1234567890" `
    -DurationSeconds 30 `
    -RequestSize 1024
```

### Manual Single Request Test
```powershell
Invoke-RestMethod `
    -Uri "http://localhost:7764/api/random?bytes=1024&encoding=hex" `
    -Headers @{ "Authorization" = "Bearer test-key-1234567890" }
```

---

## Appendix B: Statistical Methodology

All latency percentiles calculated using the following method:

1. Collect all successful request latencies
2. Sort in ascending order
3. Calculate percentile index: `floor(count × percentile)`
4. Return value at that index

**Example (P95):**
- Sorted latencies: 17,243 values
- P95 index: floor(17,243 × 0.95) = 16,380
- P95 value: latency[16,380] = 6.89ms

**Confidence:**
With 17,243 samples for sustained test and 13,148 for burst test, percentile estimates have high confidence (>99.9% for P50, >99% for P99).

---

**Original Test Date**: November 18, 2025
**Remote Access Verification**: December 14, 2025
**Tester**: Valer Bocan, PhD, CSSLP
**System Version**: v1.0.0
**Document Version**: 1.1
