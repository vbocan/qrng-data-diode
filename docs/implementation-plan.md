### Implementation Plan: QRNG Bridge Service

This plan outlines the development phases for creating the two core Rust components: the Entropy Collector (EC) and the Entropy Gateway (EG). The development is structured to prioritize core functionality and allow for iterative addition of advanced features using push-based (data diode emulation) architecture.

---

### Phase 1: Project Setup & Core Modules

This phase establishes the project structure and defines common components that will be shared between the EC and EG.

*   **Step 1.1: Create a Rust Workspace**
    *   A monorepo using a Cargo workspace is recommended to manage the two components and shared code.
    *   **`qrng-data-diode/`**
        *   **`Cargo.toml`** (Workspace definition)
        *   **`qrng-collector/`**: Crate for the QRNG Collector binary (internal pusher).
        *   **`qrng-gateway/`**: Crate for the QRNG Gateway binary (external receiver/server).
        *   **`qrng-core/`**: Crate for shared logic (data structures, configuration models, fetching logic).
        *   **`qrng-mcp/`**: Optional crate for Model Context Protocol server implementation.

*   **Step 1.2: Define Shared Data Structures (in `qrng-core` crate)**
    *   Define a `struct` for the data packets pushed from EC to EG. This will include:
        *   `data: Vec<u8>` (random entropy payload)
        *   `timestamp: u64` (Unix timestamp for freshness tracking)
        *   `sequence_number: u64` (monotonic counter for ordering/gap detection)
        *   `signature: Vec<u8>` (HMAC-SHA256 signature for integrity verification)
        *   `checksum: Option<u32>` (CRC32 for additional integrity validation - FR-4)
    *   Use `serde` with MessagePack for efficient binary serialization (preferred over JSON for performance).
    *   Define health/status structures for `/api/status` endpoint responses.

*   **Step 1.3: Configuration Management (in `qrng-core` crate)**
    *   Define `structs` to represent the configuration from a YAML file (Requirement NFR-10).
    *   **`ECConfig`**:
        *   `appliance_url` (target QRNG appliance endpoint - legacy single source)
        *   `appliance_urls` (multiple QRNG appliance endpoints - for aggregation)
        *   `mixing_strategy` (XOR or HKDF for multi-source aggregation - see [Multi-Source Aggregation](multi-source-aggregation.md))
        *   `fetch_chunk_size` (bytes per request, default 1024)
        *   `fetch_interval` (polling interval in seconds)
        *   `buffer_size` (max accumulation before push)
        *   `push_url` (EG endpoint URL)
        *   `push_interval` (push frequency)
        *   `hmac_secret_key` (shared secret for signing)
        *   `retry_policy` (exponential backoff parameters)
    *   **`EGConfig`**:
        *   `listen_address` (bind address for HTTP server)
        *   `buffer_size` (max bytes, default 10MB - NFR-1)
        *   `buffer_ttl` (data freshness threshold - FR-6)
        *   `api_keys` (list of valid authentication keys)
        *   `rate_limit` (requests per second per client - FR-8)
        *   `hmac_secret_key` (for push mode verification)
        *   `mcp_enabled` (enable MCP server - FR-10)
        *   `metrics_enabled` (expose Prometheus metrics - FR-14)
    *   Implement configuration validation with helpful error messages (NFR-10).

*   **Step 1.4: Implement Shared Fetching Logic (in `qrng-core` crate)**
    *   Create a reusable module for HTTPS GET requests to Quantis appliance (FR-1).
    *   Use `reqwest` with TLS verification and connection pooling.
    *   Implement exponential backoff retry logic with jitter (FR-9).
    *   This module will be used by the Entropy Collector.

---

### Phase 2: Entropy Collector (EC) Implementation

This phase focuses on the internal component responsible for fetching and pushing data in push-based deployment mode.

*   **Step 2.1: Implement the Data Fetching Module**
    *   Use `tokio` for the async runtime and leverage the shared fetching module from `qrng-core` crate (FR-1).
    *   Create a loop (`tokio::time::interval`) that periodically fetches data based on configuration.
    *   Handle rate limiting from appliance gracefully with adaptive backoff (FR-9).
    *   Validate received data integrity (check for non-zero entropy, proper length).

*   **Step 2.2: Implement Data Accumulation**
    *   Use a thread-safe, lock-free buffer architecture for high throughput (consider `crossbeam` or `tokio::sync::RwLock<VecDeque<u8>>`).
    *   Implement FIFO (First-In, First-Out) queue with configurable maximum size (FR-2).
    *   Add buffer monitoring: track fill levels, oldest data timestamp for metrics.
    *   Implement overflow protection: when buffer reaches capacity, apply backpressure to fetching loop.

*   **Step 2.3: Implement the Unidirectional Pushing Module**
    *   Create a second `tokio::time::interval` loop that periodically triggers a push (FR-3).
    *   This function will:
        1.  Extract a batch of data from the accumulator (configurable batch size).
        2.  Create a data packet with monotonically increasing sequence number (as defined in `qrng-core`).
        3.  Sign the packet using HMAC-SHA256 with the shared secret (FR-7, SEC-1).
        4.  Optionally compute CRC32 checksum for additional integrity (FR-4).
        5.  Serialize packet using MessagePack for efficient transmission.
        6.  Send the packet to the EG's `/push` endpoint via HTTPS POST request.
    *   Implement fire-and-forget pattern with local queuing for push failures (FR-3, FR-9).
    *   Track push metrics: success rate, latency, bytes transferred.

*   **Step 2.4: Add Resilience and Logging**
    *   Implement exponential backoff with jitter for both fetching and pushing to handle transient network errors (FR-9).
    *   Add circuit breaker pattern: temporarily halt operations if consecutive failures exceed threshold.
    *   Integrate the `tracing` crate for structured JSON logging with configurable log levels (NFR-11).
    *   Log critical events: fetch success/failure, buffer states, push operations, configuration loading.
    *   Implement graceful shutdown: flush pending data on SIGTERM/SIGINT signals (NFR-6).

*   **Step 2.5: Health Monitoring**
    *   Implement internal health checks: verify appliance connectivity, buffer integrity.
    *   Optionally expose a local health endpoint for container orchestration (e.g., `/health` on localhost).
    *   Track uptime, total bytes fetched, total pushes attempted/succeeded.

---

### Phase 3: Entropy Gateway (EG) Implementation

This phase focuses on the external component that serves data to clients via push-based data diode architecture.

*   **Step 3.1: Set Up the Web Server and Data Reception**
    *   Use the `axum` web framework with `tokio` runtime to build a high-performance async server (NFR-2).
    *   Create a `POST /push` endpoint to receive data from the EC. This handler will:
        1.  Deserialize the incoming MessagePack data packet.
        2.  Verify the HMAC-SHA256 signature using the shared secret (FR-4, SEC-1).
        3.  Optionally validate CRC32 checksum for additional integrity assurance.
        4.  Check sequence number for gaps/duplicates and log anomalies (FR-4).
        5.  If valid, append the random data to the EG's internal buffer with timestamp.
        6.  Update metrics: last push timestamp, bytes received, sequence tracking.
        7.  Return appropriate HTTP status codes (200 OK, 401 Unauthorized, 400 Bad Request).
    *   Implement request size limits and timeout protections (SEC-2, SEC-3).

*   **Step 3.2: Implement Advanced Buffer Management**
    *   Create a thread-safe, high-capacity in-memory buffer (default 10MB, configurable - FR-4, NFR-1).
    *   Implement sophisticated FIFO with multiple policies (FR-6):
        *   **Age-based eviction**: Discard data older than configured TTL (e.g., 1 hour).
        *   **Overflow handling**: When buffer reaches capacity, remove oldest chunk.
        *   **Freshness guarantee**: Track timestamps and warn if data becomes stale.
    *   Use lock-free data structures or efficient read-write locks to minimize contention.
    *   Implement watermark monitoring: low (< 10%), medium (10-80%), high (> 80%), critical (> 95%).
    *   Add buffer compaction/defragmentation to prevent memory fragmentation.
    *   Consider implementing this as a reusable module in `qrng-core` for potential use in both components.

*   **Step 3.3: Implement the Public REST API**
    *   **`GET /api/random`** (FR-5):
        *   Query parameters: `bytes` (1-65536), `encoding` (hex/base64/binary).
        *   Validates requested size against buffer availability and rate limits.
        *   Extracts N bytes from buffer head (FIFO order).
        *   Applies requested encoding transformation.
        *   Returns data with appropriate Content-Type headers.
        *   Logs request metadata (excluding actual entropy) for audit trails.
    *   **`GET /api/status`** (FR-5):
        *   Returns comprehensive JSON object:
            *   `status`: "healthy" | "degraded" | "unhealthy"
            *   `buffer_fill_percent`: current utilization
            *   `buffer_bytes_available`: actual byte count
            *   `last_data_received`: timestamp of last push
            *   `data_freshness_seconds`: age of oldest data in buffer
            *   `uptime_seconds`: service runtime
            *   `total_requests_served`: counter
            *   `total_bytes_served`: counter
            *   `requests_per_second`: current rate
        *   Include warnings array if buffer is low, data is stale, or errors detected.
    *   **`GET /health`** (NFR-7):
        *   Lightweight endpoint for load balancer/orchestration health checks.
        *   Returns 200 OK if buffer has minimum data threshold, 503 otherwise.

*   **Step 3.4: Implement Security Features**
    *   **API Key Authentication** (FR-7, SEC-1):
        *   Create an `axum` middleware for authentication on all `/api/*` routes.
        *   Support header-based (`Authorization: Bearer <key>`) and query parameter (`?api_key=<key>`) methods.
        *   Use constant-time comparison to prevent timing attacks.
        *   Return 401 Unauthorized with generic error messages to avoid information leakage.
    *   **Rate Limiting** (FR-8, SEC-2):
        *   Implement token bucket algorithm with configurable refill rate per API key.
        *   Track limits per client IP and per API key separately.
        *   Return 429 Too Many Requests with Retry-After header when limit exceeded.
        *   Use efficient in-memory store (e.g., `moka` cache with TTL).
    *   **Request Validation** (SEC-3):
        *   Validate all query parameters with strict bounds checking.
        *   Sanitize inputs to prevent injection attacks.
        *   Set maximum request sizes to prevent resource exhaustion.
    *   **TLS/HTTPS** (SEC-4):
        *   Configure `axum` with TLS support using `rustls` or `native-tls`.
        *   Provide clear documentation for certificate setup.
        *   Recommend certificate management best practices.

---

### Phase 4: Advanced Features, Testing, and Documentation

This final phase adds the innovative extensions and ensures the project is robust and usable.

*   **Step 4.1: Integrate the Model Context Protocol (MCP) Server**
    *   Implement in the `qrng-mcp` crate or add directly to `qrng-gateway` (FR-10, FR-11, FR-12).
    *   Expose MCP tools that draw from the same entropy buffer:
        *   **`get_random_bytes`**: Returns N bytes (hex/base64 encoded).
        *   **`get_random_integers`**: Returns array of random integers in specified range.
        *   **`get_random_floats`**: Returns array of random floats in [0,1) for Monte Carlo simulations.
        *   **`get_random_uuid`**: Generates cryptographic-grade UUIDv4.
        *   **`get_status`**: Returns buffer health and service status.
    *   Support both stdio and HTTP transports for AI agent integration (FR-10).
    *   Implement proper tool schemas with parameter validation.
    *   Add comprehensive tool descriptions and examples for AI agent discovery.
    *   Test integration with popular AI frameworks (Claude, GPT, local models).

*   **Step 4.2: Implement Innovative Extensions**
    *   **Optional Entropy Enhancement** (FR-11):
        *   Add configurable post-processing pipeline (e.g., SHA-256 hashing, XOR whitening).
        *   Implement as middleware layer that processes data before serving.
        *   Document trade-offs: processing overhead vs. enhanced uniformity.
        *   Make enhancement optional and transparent to clients.
    *   **Monitoring & Metrics** (FR-14, NFR-8):
        *   Integrate `prometheus` crate and expose `/metrics` endpoint.
        *   Track key metrics:
            *   Buffer utilization histogram
            *   Request rate and latency percentiles (p50, p95, p99)
            *   Bytes served total and rate
            *   Error counters by type
            *   Data freshness gauge
            *   Push success/failure ratio (for EC)
        *   Add Grafana dashboard JSON templates in documentation.
    *   **Monte Carlo Randomness Test** (UC-1, FR-13):
        *   Implement `POST /api/test/monte-carlo?iterations=N` endpoint.
        *   Algorithm:
            1.  Generate N pairs of random floats in [0,1) from buffer.
            2.  For each (x,y), test if x² + y² ≤ 1 (inside quarter-circle).
            3.  Estimate π = 4 × (hits / N).
            4.  Calculate error: |estimated_π - actual_π|.
            5.  Compare with pseudo-random baseline (Rust's `rand` crate).
        *   Return JSON response:
            *   `estimated_pi`: computed value
            *   `error`: absolute error
            *   `convergence_rate`: iterations needed for target precision
            *   `quantum_vs_pseudo`: performance comparison
            *   `scatter_plot_data`: optional visualization data
        *   Support configurable iteration counts (default 1M, max 10M).
        *   Add statistical analysis: confidence intervals, chi-square test.
        *   Document use case in SoftwareX submission as validation methodology.

*   **Step 4.3: Comprehensive Testing**
    *   **Unit Tests** (NFR-13):
        *   Test all business logic modules in isolation.
        *   Focus on: buffer management, signature verification, encoding transformations, rate limiting.
        *   Use property-based testing (`proptest` crate) for buffer invariants.
        *   Mock external dependencies (appliance, network).
        *   Aim for 90%+ code coverage target.
    *   **Integration Tests**:
        *   Spin up mock Entropy Collector and Entropy Gateway in test environment.
        *   Test full data pipeline: fetch → accumulate → push → verify → serve.
        *   Simulate network failures and verify retry/recovery logic.
        *   Validate MCP server integration with mock AI agents.
    *   **Security Tests** (NFR-13):
        *   Test authentication bypass attempts.
        *   Verify rate limiting effectiveness under load.
        *   Test HMAC signature validation with tampered data.
        *   Attempt timing attacks on authentication.
        *   Validate input sanitization against edge cases.
    *   **Performance Tests** (NFR-2, NFR-3):
        *   Benchmark throughput under concurrent load (target: 100 req/s).
        *   Measure latency percentiles (target: < 100ms p99).
        *   Test buffer performance with various sizes.
        *   Profile memory usage and identify leaks.
        *   Stress test: sustained high load for extended periods.
    *   **Randomness Quality Tests** (NFR-12):
        *   Validate output using Monte Carlo π estimation built into the gateway.
        *   Verify entropy distribution and statistical properties.
        *   Document results in project README and SoftwareX paper.

*   **Step 4.4: Finalize Documentation and Deployment**
    *   **Core Documentation**:
        *   Create detailed `README.md` with architecture diagrams (NFR-9).
        *   Provide step-by-step setup guides for Entropy Collector and Entropy Gateway.
        *   Include API reference with OpenAPI/Swagger specs.
        *   Add MCP server usage guide with AI agent examples.
        *   Create troubleshooting guide for common issues.
    *   **Configuration Examples**:
        *   Provide example `config.yaml` files for:
            *   Entropy Collector configuration
            *   Entropy Gateway configuration
            *   High-security configurations
            *   High-throughput configurations
        *   Document all configuration parameters with defaults.
        *   Add validation checklist for production deployments.
    *   **Containerization** (NFR-14):
        *   Write optimized `Dockerfile`s for both Entropy Collector and Entropy Gateway.
        *   Use multi-stage builds for minimal image size.
        *   Provide `docker-compose.yml` for easy local testing.
        *   Add Kubernetes manifests (Deployment, Service, ConfigMap).
        *   Document orchestration best practices.
    *   **Security Hardening Guide**:
        *   Document TLS/HTTPS setup procedures (SEC-4).
        *   Provide HMAC secret key generation recommendations.
        *   Security audit checklist for production.
        *   Incident response guidelines.
    *   **SoftwareX Submission Materials**:
        *   Write comprehensive software paper draft.
        *   Include benchmark results, validation tests, use cases.
        *   Prepare code availability statement (open-source, MIT license).
        *   Create reproducible experiment scripts.
        *   Document novel contributions: data diode emulation, MCP integration, Monte Carlo validation.
---

### Phase 5: Additional Considerations and Future Enhancements

*   **Step 5.1: Performance Optimization**
    *   Profile critical paths using `cargo flamegraph` and optimize hot spots.
    *   Consider zero-copy techniques for buffer operations.
    *   Evaluate SIMD optimizations for data processing.
    *   Benchmark against performance targets (NFR-2, NFR-3).

*   **Step 5.2: Operational Tooling**
    *   Create CLI utility for administrative tasks (potentially in `qrng-cli` crate):
        *   Buffer inspection and diagnostics
        *   Configuration validation
        *   Health check testing
        *   Log analysis helpers
    *   Add systemd service files for Linux deployments.
    *   Provide Windows Service wrapper for Entropy Collector.

*   **Step 5.3: Future Research Directions**
    *   Investigate blockchain integration for entropy provenance.
    *   **✅ Multi-Source Aggregation** (Implemented): The system now supports federated QRNG networks with multiple appliances. See [Multi-Source Aggregation Guide](multi-source-aggregation.md) for details on parallel fetching, XOR/HKDF mixing, and configuration.
    *   Research quantum-inspired algorithms using served entropy.
    *   Study long-term entropy quality metrics and drift detection.

---

### Development Workflow Recommendations

1.  **Start with Phase 1**: Establish solid foundations with shared modules and clear interfaces.
2.  **Develop Entropy Collector first** (Phase 2): Easier to test in isolation with mock Entropy Gateway endpoint.
3.  **Implement Entropy Gateway core** (Phase 3): Start with push-based mode, then add direct access.
4.  **Iterate on features** (Phase 4): Add MCP, monitoring, and tests incrementally.
5.  **Continuous testing**: Run tests after each phase; maintain high coverage.
6.  **Documentation as you go**: Update docs with each feature addition.

### Key Dependencies

*   **Runtime**: `tokio` (async runtime)
*   **HTTP Client**: `reqwest` (HTTPS fetching)
*   **Web Framework**: `axum` (REST API server)
*   **Serialization**: `serde`, `serde_json`, `rmp-serde` (MessagePack)
*   **Cryptography**: `hmac`, `sha2`, `crc32fast`
*   **Configuration**: `serde_yaml`, `config`
*   **Logging**: `tracing`, `tracing-subscriber`
*   **Metrics**: `prometheus`, `metrics`
*   **Testing**: `tokio-test`, `proptest`, `mockito`
*   **MCP**: Custom implementation in `qrng-mcp` crate

### Success Criteria

*   ✅ Push-based deployment functional and well-tested
*   ✅ REST API meets all functional requirements (FR-5)
*   ✅ MCP server enables AI agent integration (FR-10-12)
*   ✅ Security features pass penetration testing (SEC-1-4)
*   ✅ Performance targets achieved (NFR-2, NFR-3)
*   ✅ Monte Carlo test validates randomness quality (UC-1)
*   ✅ Documentation complete and clear (NFR-9)
*   ✅ Code coverage ≥ 90% (NFR-13)
*   ✅ Ready for open-source release and SoftwareX submission
