# QRNG Data Diode: System Architecture

**Technical overview of the software-based data diode architecture for quantum entropy distribution**

## Table of Contents

1. [Overview](#overview)
2. [System Components](#system-components)
3. [Data Flow](#data-flow)
4. [Security Model](#security-model)
5. [API Design](#api-design)
6. [Deployment](#deployment)

---

## Overview

QRNG Data Diode bridges quantum random number generators on isolated internal networks to external applications while maintaining strong security guarantees. The architecture achieves this through a software-based data diode that emulates hardware data diode properties at near-zero cost.

**Key Properties:**
- **Unidirectional data flow**: Push-only architecture prevents reverse communication
- **Sub-4ms latency**: High-performance Rust implementation with async I/O
- **Cryptographic integrity**: Multi-layer verification with HMAC-SHA256
- **Zero-configuration AI integration**: Model Context Protocol (MCP) support

---

## System Components

The system consists of two independent services:

### Entropy Collector (Internal Network)

**Purpose**: Fetch quantum entropy from QRNG appliances and push to gateway

**Responsibilities:**
- Connects to Quantis QRNG appliances via HTTPS
- Fetches random data at configured intervals
- Signs packets with HMAC-SHA256
- Pushes signed packets to Gateway
- **Critical**: Has no listening sockets - cannot receive inbound connections

**Configuration:**
```env
QRNG_APPLIANCE_URLS=https://qrng-1.internal:443,https://qrng-2.internal:443
QRNG_HMAC_SECRET_KEY=<64-char-hex-key>
QRNG_GATEWAY_PUSH_URL=https://gateway.external:7764/push
```

### Entropy Gateway (External Network)

**Purpose**: Receive entropy and serve external API clients

**Responsibilities:**
- Receives pushed entropy packets
- Verifies cryptographic signatures
- Maintains entropy buffer (default: 10MB)
- Serves REST API to clients
- Provides MCP server for AI agents
- **Critical**: Cannot initiate connections to Collector

**Configuration:**
```env
QRNG_BUFFER_SIZE=10485760
QRNG_BUFFER_OVERFLOW_POLICY=discard
QRNG_API_KEYS=key1,key2,key3
```

---

## Data Flow

### 1. Entropy Collection

```
QRNG Appliance → Collector (fetch) → Sign Packet → Push to Gateway
```

The Collector periodically fetches entropy from one or more QRNG appliances:
- Default fetch size: 4KB per request
- Configurable fetch interval: 100ms-10s
- Multiple appliances: Data combined using XOR or HKDF

### 2. Packet Format

Each entropy packet contains:

| Field | Size | Description |
|-------|------|-------------|
| Timestamp | 8 bytes | UTC timestamp (µs precision) |
| Entropy | Variable | Raw quantum random bytes |
| CRC32 | 4 bytes | Data integrity checksum |
| HMAC | 32 bytes | SHA256 authentication tag |

### 3. Gateway Processing

```
Receive Packet → Verify HMAC → Verify CRC32 → Buffer → Serve API
```

**Buffer Management:**
- Circular buffer (default: 10MB)
- Overflow policies: `discard` (drop new data) or `replace` (overwrite oldest)
- Thread-safe concurrent access
- Metrics: fill percentage, bytes available, data age

### 4. API Distribution

External clients request entropy via REST API:

```bash
# Get random bytes
GET /api/random?bytes=32&encoding=hex

# Get random integers
GET /api/integers?count=10&min=0&max=100

# Get UUIDs
GET /api/uuid?count=5
```

---

## Security Model

### Software Data Diode Properties

**Unidirectional Flow**: Achieved through architectural constraints:

1. **Collector**: No listening sockets → cannot receive connections
2. **Gateway**: No reverse connection capability → cannot probe internal network
3. **Network rules**: Firewall blocks inbound to Collector, outbound from Gateway to internal IPs

This creates a "push-only" data flow where compromising the Gateway provides no path to attack the internal network.

### Multi-Layer Verification

**Layer 1: Network Authentication**
- TLS 1.3 for encrypted transport
- Client certificate validation (optional)

**Layer 2: Application Signature**
- HMAC-SHA256 with shared secret key
- Prevents packet forgery and replay attacks

**Layer 3: Data Integrity**
- CRC32 checksum on entropy payload
- Detects transmission errors

**Layer 4: Timestamp Validation**
- Rejects packets >60s old
- Prevents replay attacks

### Firewall Configuration

**Internal Network (Collector):**
```
ALLOW: Outbound HTTPS to Gateway (specific IP:7764)
ALLOW: Outbound HTTPS to QRNG appliance (port 443)
DENY:  All inbound connections
```

**External Network (Gateway):**
```
ALLOW: Inbound HTTPS from Collector (specific IP)
ALLOW: Inbound HTTPS from API clients
DENY:  All outbound to RFC 1918 addresses (10/8, 172.16/12, 192.168/16)
```

---

## API Design

### REST API Endpoints

**Entropy Distribution:**
- `GET /api/random` - Raw random bytes (hex/base64/binary)
- `GET /api/integers` - Random integers in range
- `GET /api/floats` - Random floats [0, 1)
- `GET /api/uuid` - UUIDv4 generation

**Monitoring:**
- `GET /health` - Simple health check (no auth)
- `GET /api/status` - Detailed system status (auth required)
- `GET /metrics` - Prometheus metrics (no auth)

**Testing:**
- `POST /api/test/monte-carlo` - Randomness quality validation

### Authentication

Two methods supported (both work for all authenticated endpoints):

**1. Query Parameter:**
```bash
curl "https://gateway/api/random?bytes=32&api_key=YOUR_KEY"
```

**2. Authorization Header:**
```bash
curl -H "Authorization: Bearer YOUR_KEY" "https://gateway/api/random?bytes=32"
```

### Rate Limiting

Token bucket algorithm per API key:
- Default: 100 requests/second per key
- Configurable per deployment
- Returns `HTTP 429` when exceeded

### Model Context Protocol (MCP)

Built-in MCP server provides quantum randomness to AI agents.

**Tools Available:**
- `get_random_bytes` - Fetch random bytes
- `get_random_integers` - Generate random integers
- `get_random_hex` - Get hex-encoded data
- `get_random_base64` - Get base64-encoded data

**Using the Public MCP Server:**

The public MCP server at `https://qrng-mcp.datamana.ro` requires no authentication from clients (the MCP server handles Gateway authentication internally).

**Claude Desktop** - Add to Settings → Connectors:
```
Server URL: https://qrng-mcp.datamana.ro
```

**LM Studio** - Add in Integrations:
```
MCP Server URL: https://qrng-mcp.datamana.ro
```

**Self-Hosted MCP Server:**

For self-hosted deployments, run the MCP server with Docker:

```bash
docker run -d \
  -p 8080:8080 \
  -e QRNG_GATEWAY_URL=http://your-gateway:7764 \
  -e QRNG_GATEWAY_API_KEY=your-api-key \
  ghcr.io/vbocan/qrng-mcp:latest
```

Then configure Claude/LM Studio to connect to `http://localhost:8080`.

---

## Deployment

### Docker Compose (Recommended)

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  qrng-collector:
    image: ghcr.io/vbocan/qrng-collector:latest
    environment:
      - QRNG_APPLIANCE_URLS=https://qrng.internal:443
      - QRNG_HMAC_SECRET_KEY=${HMAC_KEY}
      - QRNG_GATEWAY_PUSH_URL=https://gateway:7764/push
    restart: unless-stopped

  qrng-gateway:
    image: ghcr.io/vbocan/qrng-gateway:latest
    environment:
      - QRNG_BUFFER_SIZE=10485760
      - QRNG_HMAC_SECRET_KEY=${HMAC_KEY}
      - QRNG_API_KEYS=${API_KEYS}
    ports:
      - "7764:7764"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:7764/health"]
      interval: 30s
    restart: unless-stopped
```

**Start services:**
```bash
# Generate shared secret
export HMAC_KEY=$(openssl rand -hex 32)
export API_KEYS=production-key-1,production-key-2

docker-compose up -d
```

### Configuration

**Environment Variables:**

| Variable | Component | Default | Description |
|----------|-----------|---------|-------------|
| `QRNG_APPLIANCE_URLS` | Collector | - | Comma-separated QRNG endpoints |
| `QRNG_HMAC_SECRET_KEY` | Both | - | Shared authentication secret |
| `QRNG_GATEWAY_PUSH_URL` | Collector | - | Gateway push endpoint |
| `QRNG_BUFFER_SIZE` | Gateway | 10485760 | Buffer size in bytes (10MB) |
| `QRNG_BUFFER_OVERFLOW_POLICY` | Gateway | discard | `discard` or `replace` |
| `QRNG_API_KEYS` | Gateway | - | Comma-separated API keys |
| `QRNG_RATE_LIMIT` | Gateway | 100 | Requests/second per key |

### Monitoring

**Prometheus Metrics:**
```
# Buffer status
qrng_buffer_fill_percent
qrng_buffer_bytes_available

# Request metrics
qrng_requests_total
qrng_bytes_served_total
qrng_request_latency_seconds

# System health
qrng_uptime_seconds
qrng_collector_fetch_errors_total
```

**Grafana Dashboard:**
- Buffer fill percentage over time
- Request throughput (req/s)
- Latency distribution (P50/P95/P99)
- Error rates

**Alerting Rules:**
```yaml
- alert: BufferCriticallyLow
  expr: qrng_buffer_fill_percent < 10
  for: 5m

- alert: HighErrorRate
  expr: rate(qrng_collector_fetch_errors_total[5m]) > 0.1
  for: 2m
```

### Scaling

**Horizontal Scaling:**
- Multiple Gateway instances behind load balancer
- Each Gateway has independent buffer
- Collector pushes to all Gateways (broadcast mode)

**Vertical Scaling:**
- Increase buffer size for higher burst capacity
- More QRNG appliances → higher sustained throughput
- Buffer size vs. memory: 10MB ≈ 10MB RAM

---

## Performance Characteristics

**Latency:**
- P50: 3.62ms
- P95: 6.89ms  
- P99: 9.13ms

**Throughput:**
- Sustained: ~29 req/s (limited by QRNG hardware)
- Burst: 400+ req/s (from buffer)

**Scalability:**
- Linear scaling with multiple QRNG appliances
- Buffer provides burst absorption

**Comparison to Alternatives:**
- 6-124x faster than public QRNG services (ANU, NIST Beacon)
- $5K-$50K cheaper than hardware data diodes
- Sub-10ms vs. 200-500ms for cloud services

---

## Technology Stack

**Implementation:**
- Language: Rust 1.75+
- Async Runtime: Tokio
- HTTP Server: Axum
- Cryptography: RustCrypto

**Deployment:**
- Containers: Docker
- Orchestration: Docker Compose / Kubernetes
- Monitoring: Prometheus + Grafana

**License:** MIT - Free for academic and commercial use

---

## See Also

- [Security Analysis](SECURITY-ANALYSIS.md) - Detailed threat model and mitigations
- [Performance Benchmarks](BENCHMARK.md) - Comprehensive performance testing
- [MCP Integration Guide](MCP-INTEGRATION.md) - AI agent integration
- [Main README](../README.md) - Quick start and examples

A single security mechanism, no matter how strong, represents a single point of failure. QRNG-DD implements four independent integrity checks that must all pass for packet acceptance.

**Layer 1: HMAC-SHA256 Authentication**

Every entropy packet includes a 256-bit HMAC computed over the payload, timestamp, and sequence number using a shared secret key. This cryptographic signature serves multiple purposes:

- **Authentication**: Verifies packets originated from the legitimate Collector
- **Integrity**: Detects any tampering with packet contents
- **Non-repudiation**: Provides cryptographic proof of packet origin

The shared secret is a 256-bit random value generated at deployment time using OpenSSL's cryptographically secure PRNG. Both Collector and Gateway load this secret from environment variables, never from configuration files that might leak into version control.

HMAC-SHA256 provides collision resistance of approximately 2^128 operations and preimage resistance of 2^256 operations—well beyond the capabilities of any current or foreseeable adversary. Even quantum computers running Grover's algorithm would require 2^128 operations to find a collision, keeping this secure well into the post-quantum era for our use case.

**Layer 2: CRC32 Checksum**

While HMAC provides cryptographic authentication, CRC32 serves a complementary role: detecting accidental corruption from bit flips, network errors, or memory faults. This distinction matters because:

- CRC32 computation takes ~290 microseconds (negligible overhead)
- Cryptographic verification should catch deliberate attacks
- Checksums should catch accidental corruption
- Two independent checks catch more errors than one

The separation of concerns means we can use fast, non-cryptographic error detection for the common case (random bit flips) while reserving expensive cryptographic verification for the threat model (deliberate attacks).

**Layer 3: Timestamp Freshness Validation**

Each packet carries a UTC timestamp recording its creation time. The Gateway rejects packets that are:

- **Too old**: Exceeding configurable TTL (default: 300 seconds)
- **Too new**: Future timestamps indicating clock skew or manipulation

This temporal validation prevents replay attacks where an attacker captures valid packets and retransmits them later. The 5-minute TTL balances security (shorter window for replay) against operational tolerance (network delays, clock drift).

We chose 300 seconds after analyzing typical network latencies between internal and external networks in academic institutions, which rarely exceed 60 seconds even during congestion. The remaining 4-minute margin accommodates clock skew between systems even without perfectly synchronized NTP.

**Layer 4: Sequence Number Monotonicity**

The Collector assigns monotonically increasing sequence numbers to packets using atomic operations. The Gateway tracks the last observed sequence and rejects any packet with a sequence number less than or equal to this value.

This catches replay attacks even within the TTL window. An attacker who captures packet #100 and attempts to replay it will fail because the Gateway has already processed #100 and incremented its "last seen" counter to at least 101.

Sequence gaps (e.g., receiving #105 after #103) are permitted and logged but not rejected, since network packet loss is a reality. However, monotonicity violations always indicate attacks or misconfigurations and trigger immediate rejection plus security alerts.

**Combined Security Posture**

An attacker attempting to inject malicious entropy must simultaneously:
1. Forge an HMAC signature without knowing the 256-bit secret
2. Generate a valid CRC32 checksum
3. Set a timestamp within the current 5-minute window
4. Guess a sequence number higher than the Gateway's current value

This combinatorial defense makes successful attacks computationally infeasible with current technology.

### Configurable Buffer Overflow Policy

When incoming entropy accumulates faster than consumption—such as during idle periods or development testing—the buffer eventually fills. The system supports two policies for handling this scenario, chosen based on operational priorities:

**Discard Policy (Default)**

When the buffer reaches capacity, incoming packets are rejected and logged. This conservative approach:

- Preserves the temporal distribution of buffered data
- Provides predictable behavior for monitoring systems
- Clearly signals consumption issues through metrics
- Aligns with traditional queue overflow semantics

In production deployments with regular consumption, a full buffer typically indicates healthy backlog, not a problem. Discarding new data prevents masking consumption failures.

**Replace Policy (Alternative)**

When the buffer reaches capacity, the oldest buffered data is evicted (FIFO) to make room for fresh incoming entropy. This approach:

- Maximizes freshness—buffer always contains most recently generated quantum data
- Prevents stale data accumulation during low-consumption scenarios
- Reduces theoretical exposure window for side-channel observations
- Makes cryptographic sense: quantum entropy has no temporal dependencies

The cryptographic soundness of the replace policy stems from a fundamental property of quantum randomness: each sample is statistically independent of generation time. Unlike cryptographic keys (which can be compromised) or nonces (which must be unique), raw entropy doesn't "expire" or gain semantic meaning from age. QRNG output at T=0 is mathematically equivalent to output at T=100 in terms of randomness quality.

The choice between policies reflects operational philosophy:
- **Discard**: "Alert me when consumption drops; preserve backlog"  
- **Replace**: "Always serve freshest entropy; I'll monitor other metrics"

Both policies maintain cryptographic quality; the difference lies in operational semantics.

---

## Data Flow and Protocol Design

### MessagePack: Efficient Binary Serialization

Entropy packets traverse the network boundary thousands of times per day. Serialization format directly impacts both performance and wire efficiency. We evaluated three options:

**JSON** - The obvious choice for interoperability:
- Packet size: 143 bytes
- Serialize: 2.3μs
- Deserialize: 3.1μs
- Human-readable for debugging
- Universal tooling support

**Protocol Buffers** - The performance champion:
- Packet size: 76 bytes (46% smaller than JSON)
- Serialize: 1.1μs (2× faster than JSON)
- Deserialize: 1.5μs (2× faster than JSON)
- Requires schema compilation
- Complex build pipeline

**MessagePack** - The pragmatic middle ground:
- Packet size: 89 bytes (38% smaller than JSON)
- Serialize: 0.8μs (2.9× faster than JSON)
- Deserialize: 1.2μs (2.6× faster than JSON)
- No schema compilation
- Simple Rust integration via `serde`

MessagePack delivered 90% of Protocol Buffers' benefits without the operational complexity. For a research project where build simplicity matters and wire efficiency gains beyond 40% provide diminishing returns, this was the clear winner.

The Rust implementation leverages `serde`'s derive macros:

```rust
#[derive(Serialize, Deserialize)]
pub struct EntropyPacket {
    pub version: u8,
    pub id: Uuid,
    pub sequence: u64,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
    pub checksum: Option<u32>,
}
```

The `#[serde(with = "serde_bytes")]` attributes optimize byte array encoding, avoiding base64 overhead that would inflate packet size.

### Packet Structure and Wire Protocol

Each entropy packet encapsulates:

1. **Protocol Version** (1 byte): Enables future wire format changes while maintaining backward compatibility
2. **Packet ID** (16 bytes): UUID for distributed tracing and deduplication
3. **Sequence Number** (8 bytes): Monotonically increasing counter for replay detection
4. **Entropy Payload** (variable): The actual quantum random bytes
5. **Timestamp** (8 bytes): UTC creation time for freshness validation
6. **HMAC Signature** (32 bytes): SHA-256 authentication tag
7. **CRC32 Checksum** (4 bytes): Error detection for payload integrity

Total overhead: 69 bytes + UUID (16) = 85 bytes per packet. For a typical 1KB entropy payload, this represents 8.3% overhead—acceptable given the security guarantees provided.

The push protocol uses standard HTTPS POST requests:

```http
POST /push HTTP/1.1
Host: gateway.example.com:7764
Content-Type: application/x-msgpack
Content-Length: 1109

[binary MessagePack data]
```

This simplicity has profound operational benefits:
- Standard load balancers handle routing
- HTTPS provides transport encryption
- Debugging tools (tcpdump, Wireshark) understand the protocol
- No custom TCP protocol implementation to test

### In-Memory Buffering Strategy

Entropy buffers exist in both Collector and Gateway, serving different purposes but sharing implementation. The design deliberately avoids disk persistence:

**Why Memory-Only?**

1. **Performance**: Memory access is 1000× faster than SSD, 10,000× faster than spinning disk
2. **Simplicity**: No filesystem I/O, no file rotation, no corruption recovery
3. **Security**: Sensitive random data never written to disk
4. **Statelessness**: Container crashes simply restart, no cleanup required
5. **Scalability**: Horizontal scaling requires no shared storage

**What About Data Loss?**

Entropy is fundamentally disposable. Unlike user data or configuration, random bytes have no semantic meaning and are continuously regenerated. A Collector crash loses at most 1MB of buffered entropy—trivially replaced by 10 seconds of QRNG operation.

This architectural choice reflects the fundamental nature of the resource being managed. Entropy is a consumable that flows continuously, not a precious asset requiring durable storage.

**Buffer Implementation Details**

The buffer uses a VecDeque (double-ended queue) wrapped in an Arc<RwLock>:

- **VecDeque**: Efficient FIFO operations with O(1) push/pop at both ends
- **Arc**: Atomic reference counting for sharing between async tasks
- **RwLock**: Reader-writer lock optimized for read-heavy workloads

The `parking_lot` crate provides RwLock implementation that's 2-3× faster than std::sync::RwLock through futex-based locking and uncontended fast paths. For our workload—many concurrent reads (API requests), occasional writes (packet arrival)—this optimization measurably reduces tail latencies.

Zero-copy operations throughout: when the Gateway serves API requests, it uses `Bytes::clone()` which performs reference counting increment (atomic operation) rather than data copying. A 64KB API response requires copying a pointer, not 64KB of memory.

---

## Performance Engineering

### Asynchronous I/O and Cooperative Multitasking

The Gateway must handle hundreds of concurrent API requests while simultaneously receiving pushed packets from the Collector. Traditional threaded servers would spawn a thread per connection, rapidly exhausting resources with thousands of concurrent clients.

QRNG-DD uses Tokio's asynchronous runtime for cooperative multitasking:

**How It Works**

Each connection becomes a lightweight "task" (essentially a state machine) scheduled by Tokio's work-stealing executor. Tasks voluntarily yield at `.await` points, allowing the executor to interleave thousands of tasks across a small thread pool (typically matching CPU core count).

Example: Serving an API request:

```rust
async fn get_random_bytes(
    State(state): State<AppState>,
    Query(params): Query<BytesRequest>,
) -> Result<impl IntoResponse> {
    // Acquire read lock - async-aware, yields if contended
    let data = state.buffer.pop(params.bytes).await?;
    
    // Encoding computation - CPU-bound, stays on current task
    let encoded = match params.encoding {
        Encoding::Hex => hex::encode(data),
        Encoding::Base64 => base64::encode(data),
        Encoding::Binary => data.to_vec(),
    };
    
    Ok(Json(RandomBytesResponse { data: encoded }))
}
```

This function handles potentially millions of requests without creating millions of threads. The `.await` point allows Tokio to schedule other tasks if the buffer lock is contended, maximizing CPU utilization.

**Performance Characteristics**

Our benchmarks show:
- 400+ requests/second burst capacity on modest hardware (4-core laptop)
- Sub-4ms P50 latency, sub-10ms P99 latency
- Sustained 29 req/s limited by QRNG appliance, not gateway processing

The last point is critical: production throughput is bounded by quantum entropy generation rate (hardware physics), not software processing capacity. This headroom ensures the system never becomes the bottleneck.

### Lock-Free Sequence Generation

The Collector assigns monotonically increasing sequence numbers to packets. Naive implementation would use a Mutex:

```rust
// DON'T DO THIS
static mut SEQUENCE: u64 = 0;
let seq = {
    let mut guard = SEQUENCE_MUTEX.lock();
    *guard += 1;
    *guard
};
```

But Mutex::lock() is a heavyweight operation requiring kernel system calls on contention. For a critical path executed thousands of times per second, this creates unacceptable overhead.

Instead, we use atomic operations:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn next_sequence() -> u64 {
    SEQUENCE.fetch_add(1, Ordering::SeqCst)
}
```

`fetch_add` compiles to a single CPU instruction (LOCK XADD on x86) that atomically reads, increments, and returns the value. No kernel involvement, no waiting for lock contention—just a single instruction that's maybe 10-20 cycles.

The `Ordering::SeqCst` provides sequential consistency (strongest guarantee), ensuring all threads observe sequence numbers in total order. Weaker orderings like `Relaxed` would be unsafe here since the Gateway must observe sequence numbers in strict order to detect replay attacks.

### Zero-Copy Byte Buffer Operations

Moving large byte arrays is expensive. Copying 64KB of entropy from buffer to API response to JSON encoding to HTTP body could quadruple memory bandwidth requirements and trash CPU caches.

The solution: reference-counted byte buffers via the `bytes` crate:

```rust
use bytes::{Bytes, BytesMut};

// Gateway buffer stores Bytes (immutable, ref-counted)
let data: Bytes = buffer.pop(64 * 1024)?;

// "Cloning" just increments reference count
let copy1 = data.clone();  // Atomic increment
let copy2 = data.clone();  // Another atomic increment

// Serve API response
Ok((
    StatusCode::OK,
    [(header::CONTENT_TYPE, "application/octet-stream")],
    data,  // Bytes implements IntoResponse, no copy!
))
```

Each `clone()` performs one atomic increment operation instead of copying 64KB. The underlying memory is freed only when the last reference is dropped. Throughput-sensitive code paths never copy bulk data.

### Multi-Source Parallel Fetching

Deployments with multiple QRNG appliances could fetch sequentially:

```rust
// Sequential - SLOW
let mut combined = Vec::new();
for url in appliance_urls {
    let data = fetch(url).await?;
    combined.extend(data);
}
```

But if each fetch takes 100ms, three appliances require 300ms total—unacceptable latency for a real-time system.

Parallel fetching using `join_all`:

```rust
// Parallel - FAST
use futures::future::join_all;

let futures: Vec<_> = appliance_urls
    .iter()
    .map(|url| fetch(url))
    .collect();

let results = join_all(futures).await;
```

Three 100ms fetches complete in ~100ms (network RTT dominates), reducing aggregate latency from sum-of-durations to max-of-durations. Independent timeout handling ensures slow appliances don't delay responsive ones.

---

## Observability and Operations

### Structured Logging with Tracing

Traditional text logs are human-readable but difficult to query programmatically:

```
[2025-11-18 09:15:30] INFO: Received packet, sequence=42, bytes=1024
```

Structured JSON logging makes every log entry a queryable object:

```json
{
  "timestamp": "2025-11-18T09:15:30.123Z",
  "level": "info",
  "target": "qrng_gateway::handlers",
  "message": "Received entropy packet",
  "fields": {
    "sequence": 42,
    "bytes": 1024,
    "checksum": "0xdeadbeef",
    "source_ip": "10.0.1.42"
  },
  "span": {
    "name": "push_packet",
    "trace_id": "a3f7c9e21b8d4f6a"
  }
}
```

This enables powerful queries:

```bash
# Find all packets from specific source
jq 'select(.fields.source_ip == "10.0.1.42")' logs.json

# Analyze latency distribution
jq -r '.fields.latency_ms' logs.json | sort -n | tail -100
```

The `tracing` crate provides this through instrumentation:

```rust
#[instrument(skip(state))]
async fn push_packet(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<impl IntoResponse> {
    let packet = EntropyPacket::from_msgpack(&body)?;
    
    info!(
        sequence = packet.sequence,
        bytes = packet.data.len(),
        "Received entropy packet"
    );
    
    // Logs automatically include span context
    state.buffer.push(packet.data).await?;
    Ok(StatusCode::CREATED)
}
```

The `#[instrument]` macro automatically creates a span that appears in all logs within the function, providing request-level context for every operation.

### Prometheus Metrics

Logs capture discrete events; metrics capture aggregate trends. The Gateway exposes Prometheus-format metrics:

```
# Request count by status code
qrng_requests_total{status="200"} 15234
qrng_requests_total{status="400"} 42
qrng_requests_total{status="429"} 8

# Bytes served
qrng_bytes_served 48234567

# Latency histogram buckets
qrng_latency_seconds_bucket{le="0.001"} 1234
qrng_latency_seconds_bucket{le="0.005"} 14523
qrng_latency_seconds_bucket{le="0.010"} 15234
qrng_latency_seconds_bucket{le="+Inf"} 15234
qrng_latency_seconds_sum 45.678
qrng_latency_seconds_count 15234

# Buffer fill level
qrng_buffer_fill_percent 73.2
```

These metrics power Grafana dashboards and Prometheus alerting:

```yaml
# Alert if buffer drops critically low
- alert: BufferCriticallyLow
  expr: qrng_buffer_fill_percent < 10
  for: 5m
  annotations:
    summary: "Entropy buffer critically low"
```

### Health Checks and Graceful Degradation

The Gateway provides health endpoints for load balancer integration:

```http
GET /health HTTP/1.1

{
  "status": "healthy",
  "buffer_fill_percent": 73.2,
  "uptime_seconds": 86400,
  "warnings": []
}
```

Status levels:
- **healthy**: Buffer >30%, no warnings
- **degraded**: Buffer 10-30%, operation continues with warnings
- **unhealthy**: Buffer <10%, may fail requests

Load balancers can use this for intelligent routing, directing traffic away from degraded instances while they recover.

Graceful degradation philosophy: warn early, fail gracefully, never silent failure. A buffer at 25% triggers warnings but continues serving requests, giving operators time to investigate before critical failure.

---

## Deployment

### Docker Containerization

The entire system deploys via Docker Compose:

```yaml
services:
  qrng-gateway:
    image: ghcr.io/vbocan/qrng-gateway:latest
    environment:
      - QRNG_BUFFER_SIZE=10485760
      - QRNG_BUFFER_OVERFLOW_POLICY=discard
      - QRNG_API_KEYS=secret-key-here
    ports:
      - "7764:7764"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:7764/health"]
      interval: 30s
```

The Rust release builds produce static binaries with minimal dependencies, creating tiny Docker images (~50MB) that start in milliseconds.

### Configuration Management

Configuration follows twelve-factor app principles: environment variables for secrets and runtime config, YAML files for complex structured configuration.

```bash
# Secrets via environment
export QRNG_HMAC_SECRET_KEY=$(openssl rand -hex 32)
export QRNG_API_KEYS=production-key-1,production-key-2

# Structured config via YAML
cat > config.yaml <<EOF
gateway:
  buffer_size: 10485760
  buffer_overflow_policy: "replace"
  rate_limit_per_second: 1000
EOF
```

This separation keeps secrets out of version control while allowing complex configuration to be declarative and versioned.

---
