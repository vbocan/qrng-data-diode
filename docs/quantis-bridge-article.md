# QRNG Bridge Service: Requirements Document

## Document Metadata
- **Document Title**: Requirements Specification for Rust-Based Quantum Random Number Generator (QRNG) Bridge Service with Split EC-EG Architecture
- **Version**: 1.0
- **Date**: November 05, 2025
- **Author(s)**: [User's Name/Organization] (with conceptual input from Grok 4, xAI)
- **Purpose**: This document provides a comprehensive, formal specification of requirements for a software-based bridge service that exposes a locally networked Quantis QRNG appliance to external clients. It synthesizes discussions on architecture, security, features, and innovations, adopting a split EC-EG design to emulate a data diode for unidirectional data flow. The service addresses network restrictions, rate limiting, and security concerns while enabling creative extensions for quantum entropy applications. This serves as a foundational artifact for development, testing, and a potential SoftwareX submission.
- **Audience**: Developers, security architects, researchers in quantum computing and cryptography.
- **Assumptions**: The Quantis appliance is accessible via HTTPS at `https://random.cs.upt.ro` within the internal network, providing random data in binary format (e.g., via `GET /random?bytes=N`). It has inherent rate limits (e.g., ~1 Mbps). The service will be implemented in Rust for safety and performance. Open-source under MIT license.
- **References**: Internal discussions on MITM proxying, data diode emulation, Rust peculiarities, and innovative features.

## 1. Introduction
### 1.1 Background
The Quantis QRNG appliance generates high-quality quantum-derived randomness, suitable for applications in cryptography, simulations, and scientific computing. However, its restriction to local network access limits broader utilization. This bridge service acts as a secure intermediary, fetching randomness from the appliance and distributing it externally via a public API, while overcoming rate limits through data accumulation.

The chosen architecture splits the service into two decoupled components:
- **Entropy Collector (EC) (Internal Pusher)**: Resides within the local network, fetches data from the appliance, accumulates it, and pushes it unidirectionally to Entropy Gateway (EG).
- **Entropy Gateway (EG) (External Receiver)**: Resides outside the network (e.g., on a public server), receives pushed data, accumulates it in a buffer, and exposes a RESTful API for external clients.

This split emulates a software-based data diode, ensuring unidirectional flow (outbound randomness only) to protect the internal network from external threats. Data accumulation at both ends handles rate limiting and prevents overloading, allowing bursty external demands.

### 1.2 Objectives
- Provide secure, global access to QRNG entropy without direct exposure of the appliance.
- Emulate data diode principles for enhanced security.
- Support data accumulation to manage appliance constraints.
- Include innovative features for entropy enhancement and verification.
- Facilitate a use case demonstration via a Monte Carlo simulation to validate randomness quality.
- Support dual deployment modes: push-based (data diode emulation) and direct access (simplified architecture).
- Provide Model Context Protocol (MCP) server interface for AI agent integration.
- Ensure the design is extensible for research and open-source contributions.

### 1.3 Scope
- **In Scope**: Core fetching, pushing, buffering, API exposure; security features; randomness quality testing via Monte Carlo; dual deployment modes; MCP server integration; Rust-specific implementation guidelines.
- **Out of Scope**: Hardware integration (e.g., physical diodes); full appliance API reverse-engineering; production deployment scripting; non-Rust alternatives.

## 2. Functional Requirements
### 2.1 Deployment Modes
The system shall support two distinct deployment architectures to accommodate different network topologies and security requirements:

#### 2.1.1 Push-Based Mode (Data Diode Emulation)
- **FR-0.1**: The system shall support a push-based deployment where Entropy Collector (EC) resides within the restricted network and Entropy Gateway (EG) resides externally.
- **FR-0.2**: In push-based mode, data flow shall be strictly unidirectional from EC to EG, emulating data diode principles.
- **FR-0.3**: Entropy Collector (EC) shall be responsible for fetching, accumulating, and pushing data to Entropy Gateway (EG).
- **FR-0.4**: Entropy Gateway (EG) shall operate independently without requiring network access to the Quantis appliance.
- **FR-0.5**: This mode is suitable for high-security environments where internal network isolation is critical.

#### 2.1.2 Direct Access Mode
- **FR-0.6**: The system shall support a direct access deployment where Entropy Gateway (EG) has direct network access to the Quantis appliance.
- **FR-0.7**: In direct access mode, Entropy Collector (EC) is not deployed, and Entropy Gateway (EG) directly fetches random data from the appliance.
- **FR-0.8**: Entropy Gateway (EG) shall implement the same fetching, accumulation, and API exposure logic as in push-based mode.
- **FR-0.9**: Direct access mode shall be configurable via deployment parameters (e.g., environment variables, configuration files).
- **FR-0.10**: This mode is suitable for simplified deployments where both the service and appliance reside within the same trusted network boundary or where network restrictions are not a concern.
- **FR-0.11**: The system architecture shall be designed to allow runtime mode selection without code modification.

### 2.2 Core Functionality
- **FR-1: Data Fetching (Entropy Collector (EC))**: Entropy Collector (EC) shall periodically fetch random data from the Quantis appliance using HTTPS requests (e.g., configurable chunk sizes like 1 KB every 500 ms to respect rate limits).
- **FR-2: Data Accumulation in EC**: Entropy Collector (EC) shall maintain a local buffer (e.g., 1-5 MB configurable) to accumulate fetched data before batching for pushes, preventing frequent small pushes and handling temporary appliance unavailability.
- **FR-3: Unidirectional Pushing (EC to EG)**: Entropy Collector (EC) shall push accumulated data batches (e.g., every 10-60 seconds) to Entropy Gateway (EG) using a one-way protocol (e.g., UDP, HTTP POST with no acknowledgments, or write-only shared storage like S3). Pushes shall include metadata (e.g., timestamps, batch size) but no mechanisms for reverse communication.
- **FR-4: Data Reception and Accumulation in EG**: Entropy Gateway (EG) shall receive pushes, verify authenticity (e.g., via HMAC signatures), and accumulate data in an external buffer (e.g., 10-50 MB configurable) to support burst serving, overcoming aggregate rate limits.
- **FR-5: Public API Exposure (Entropy Gateway (EG))**: Entropy Gateway (EG) shall provide a RESTful API:
  - `GET /api/random?bytes=N&format=hex|base64|binary`: Serve N bytes from the buffer.
  - `GET /api/status`: Return JSON with buffer levels, last push timestamp, and system health.
  - `POST /api/bulk`: Handle larger requests with parameters (e.g., priority queuing).
- **FR-6: Buffer Management**: Both components shall implement FIFO buffering with overflow policies (e.g., discard oldest data) and freshness expiration (e.g., discard data older than 5 minutes to ensure quantum freshness).
- **FR-6.1: Mode-Specific Behavior**: In direct access mode, Entropy Gateway (EG) shall implement the fetching logic internally (equivalent to merging EC and EG functionality).

### 2.3 Model Context Protocol (MCP) Server Integration
To enable AI agents and language models to programmatically access quantum random numbers for computational tasks, Entropy Gateway (EG) shall expose an MCP server interface alongside the REST API.

- **FR-11: MCP Server Implementation**: Entropy Gateway (EG) shall implement an MCP server conforming to the Model Context Protocol specification.
- **FR-12: MCP Tools for Random Number Generation**: The MCP server shall expose the following tools:
  - `get_random_bytes`: Request N bytes of random data with format options (hex, base64, binary).
  - `get_random_integers`: Request N random integers within a specified range [min, max].
  - `get_random_floats`: Request N random floating-point values in [0, 1) or specified range.
  - `get_status`: Query system buffer levels and health metrics.
- **FR-13: MCP Resources**: The MCP server shall expose resources providing:
  - System configuration and deployment mode information.
  - Buffer statistics and entropy quality metrics.
  - Historical usage patterns (if logging enabled).
- **FR-14: MCP Prompts**: The MCP server shall provide prompt templates for common use cases:
  - Monte Carlo simulation setup.
  - Cryptographic key material generation.
  - Statistical randomness testing.
- **FR-15: Authentication for MCP**: The MCP server shall support authentication via API keys or standard MCP authentication mechanisms.
- **FR-16: Concurrency**: The MCP server shall handle concurrent requests from multiple AI agents efficiently using Rust's async runtime.
- **FR-17: Error Handling**: MCP tool invocations shall return structured error responses when buffer is depleted or system is unavailable.
- **FR-18: Documentation**: MCP tool schemas shall include comprehensive documentation of parameters, return types, and usage examples.

**Rationale**: MCP integration enables AI systems to leverage quantum randomness for enhanced computational tasks, including cryptographic applications, simulations requiring high-quality entropy, and research experiments. This positions the bridge service as an AI-accessible quantum resource, expanding its utility beyond traditional API consumers.

### 2.4 Security Features
- **FR-7: Authentication and Authorization**: API endpoints in B shall require API keys or JWT. Pushes from A to B shall use cryptographic signing for integrity.
- **FR-8: Rate Limiting**: B shall enforce per-client rate limits (e.g., 1 MB/min) to prevent abuse. A shall throttle fetches based on appliance health.
- **FR-9: Error Handling and Resilience**: Both components shall retry operations (e.g., exponential backoff for fetches/pushes) and log errors without creating feedback loops.
- **FR-10: Unidirectionality Enforcement**: No code paths in B shall allow data or commands to flow back to A; use firewall rules or protocol choices to enforce this.

### 2.5 Innovative Extensions
- **FR-11: Entropy Enhancement**: B shall support optional post-processing (e.g., von Neumann debiasing, hashing via SHA-256) and hybrid fusion with other sources (e.g., OS entropy).
- **FR-12: Verifiable Randomness**: B shall offer signed random blocks for applications like blockchain VRF.
- **FR-13: Streaming Mode**: B shall provide a WebSocket endpoint `/ws/random?rate=kbps` for continuous delivery.
- **FR-14: Monitoring and Analytics**: Both components shall expose Prometheus metrics for buffer usage, throughput, and health. B may include ML-based demand prediction for proactive buffering.
- **FR-15: Diagnostic Tools**: Integrate randomness tests (e.g., NIST SP 800-90B) in B for on-demand quality checks.

## 3. Non-Functional Requirements
### 3.1 Performance
- **NFR-1: Throughput**: The system shall sustain at least 2x the appliance's rate limit via accumulation (e.g., serve 10 MB bursts from B's buffer).
- **NFR-2: Latency**: Fetch-to-serve latency < 1 second for buffered data; push intervals configurable to balance load.
- **NFR-3: Scalability**: B shall handle 100 concurrent clients; support horizontal scaling.

### 3.2 Reliability and Availability
- **NFR-4: Uptime**: 99.9% availability; components shall restart automatically on failure.
- **NFR-5: Data Integrity**: Ensure no corruption during accumulation/pushing (e.g., via checksums).
- **NFR-6: Fault Tolerance**: Handle network partitions by queuing in A; fallback to pseudo-random in B with warnings.

### 3.3 Security
- **NFR-7: Compliance**: Align with zero-trust and data diode principles; support audits via logged hashes.
- **NFR-8: Encryption**: All traffic (fetches, pushes, API) shall use TLS 1.3.
- **NFR-9: Vulnerability Mitigation**: Use Rust's safety features to prevent overflows/races; regular dependency audits.

### 3.4 Usability and Maintainability
- **NFR-10: Configuration**: YAML-based configs for buffer sizes, intervals, endpoints.
- **NFR-11: Logging**: Structured logging (e.g., JSON) for debugging.
- **NFR-12: Documentation**: Inline code docs; user guide for deployment.
- **NFR-13: Testing**: Unit/integration tests covering 90% code; simulate diode breaches.

### 3.5 Deployment
- **NFR-14: Environment**: Rust 1.70+; Docker containers for A (internal server) and B (cloud VPS).
- **NFR-15: Monitoring**: Integrate with Grafana for visualizations.

## 4. Architecture Overview
### 4.1 High-Level Design

#### 4.1.1 Push-Based Mode Architecture
- **Entropy Collector (EC)**: Async Rust app using `tokio` for periodic fetches (`reqwest` to appliance), local buffering (`Vec<u8>` or `bytes` crate), and pushes (e.g., UDP via `tokio::net` or S3 via `rusoto`).
- **Entropy Gateway (EG)**: `axum`-based server for REST API and MCP server, with `tokio::sync::RwLock` for buffer, receiving pushes via dedicated endpoint or listener.
- **Data Flow**: Appliance → EC (fetch/accumulate) → Push (unidirectional) → EG (receive/accumulate/serve) → Clients/AI Agents.

#### 4.1.2 Direct Access Mode Architecture
- **Entropy Gateway (EG) (Unified)**: Single `axum`-based server that integrates:
  - Fetching module: Periodic appliance requests using `reqwest`.
  - Buffer management: Same accumulation logic as push-based EG.
  - API exposure: REST API and MCP server endpoints.
- **Data Flow**: Appliance → EG (fetch/accumulate/serve) → Clients/AI Agents.
- **Configuration**: Mode selected via environment variable (e.g., `DEPLOYMENT_MODE=direct` vs `DEPLOYMENT_MODE=push`).

#### 4.1.3 Shared Components
- **Rust Peculiarities**: Leverage ownership for safe buffering; async traits for modularity; crates like `serde` for configs, `ring` for crypto.
- **MCP Integration**: Use `mcp-server-rs` crate or implement custom MCP protocol handler over stdio/HTTP transport.

### 4.2 Data Accumulation Strategy
- **In A**: Accumulate to batch pushes, reducing overhead and handling rate limits (e.g., fetch 1 KB chunks, push 100 KB batches).
- **In B**: Larger buffer for external bursts; if overloaded, respond with partial data or HTTP 429.
- **Rationale**: Dual accumulation prevents appliance overload (A paces fetches) and ensures availability (B handles spikes).

## 5. Use Cases
### 5.1 Primary Use Case: External Randomness Consumption
- Actor: External client (e.g., researcher).
- Steps: Client requests via B's API; B serves from buffer; if low, waits for next push from A (push mode) or fetches directly (direct mode).
- Preconditions: System deployed; buffers initialized.
- Postconditions: Client receives verifiable quantum randomness.

### 5.1.1 Deployment Mode Selection Use Case
- **UC-MODE-1: Push-Based Deployment**:
  - Actor: System administrator.
  - Preconditions: Quantis appliance accessible only from internal network; external server available for Entropy Gateway (EG).
  - Steps:
    1. Deploy Entropy Collector (EC) within internal network with appliance access.
    2. Configure EC with appliance URL and push target (EG's endpoint).
    3. Deploy Entropy Gateway (EG) on external server with public API exposure.
    4. Configure EG to receive and authenticate pushes from EC.
    5. Initialize both components; verify unidirectional data flow.
  - Postconditions: System operates with network isolation; external clients access randomness without appliance visibility.
  
- **UC-MODE-2: Direct Access Deployment**:
  - Actor: System administrator.
  - Preconditions: Deployment environment has direct network access to Quantis appliance; simplified architecture desired.
  - Steps:
    1. Deploy Entropy Gateway (EG) with `DEPLOYMENT_MODE=direct` configuration.
    2. Configure EG with appliance URL and API credentials.
    3. EG initializes fetching module and begins accumulating data.
    4. Expose REST and MCP APIs for client access.
  - Postconditions: System operates in unified mode; Entropy Collector (EC) is not required; same API interface maintained.

### 5.1.2 AI Agent Integration Use Case
- **UC-MCP-1: AI-Driven Monte Carlo Simulation**:
  - Actor: AI agent (e.g., Claude, GPT-4 with MCP support).
  - Preconditions: MCP server running on Entropy Gateway (EG); agent configured with MCP client.
  - Steps:
    1. AI agent connects to MCP server using stdio or HTTP transport.
    2. Agent discovers available tools via MCP protocol negotiation.
    3. Agent invokes `get_random_floats` tool requesting 1,000,000 samples.
    4. EG serves data from buffer; agent uses for π estimation via Monte Carlo.
    5. Agent analyzes convergence and invokes `get_status` to verify entropy quality.
    6. Agent reports results to user.
  - Postconditions: AI leverages quantum randomness for superior statistical results.

- **UC-MCP-2: Cryptographic Key Generation**:
  - Actor: AI agent performing security task.
  - Steps:
    1. Agent invokes `get_random_bytes` requesting 32 bytes for AES-256 key.
    2. EG serves quantum random data; agent uses for key derivation.
    3. Agent logs key generation metadata (excluding key material).
  - Postconditions: Cryptographic operations benefit from quantum-grade entropy.

### 5.2 Creative Use Case: Monte Carlo Simulation for Randomness Quality Validation
To demonstrate the bridge's efficacy and randomness quality, integrate a built-in test endpoint in EG that uses served entropy for a Monte Carlo simulation. Creatively, this could estimate π via a "Quantum Dartboard" method, where random points are thrown at a unit square enclosing a quarter-circle, leveraging quantum entropy for superior convergence compared to pseudo-random alternatives.

- **UC-1: Simulation Execution**:
  - Actor: User/admin via API (e.g., `POST /api/test/monte-carlo?iterations=N`).
  - Steps:
    1. EG pulls N pairs of random floats (derived from buffered entropy) in [0,1].
    2. For each pair (x,y), check if x² + y² ≤ 1 (inside quarter-circle).
    3. Estimate π as 4 * (hits / N).
    4. Compare convergence rate/error to a pseudo-random baseline (e.g., using Rust's `rand` crate).
    5. Return JSON with estimate, error bounds, and visualizations (e.g., scatter plot data for client rendering).
  - Preconditions: Sufficient buffer data; N configurable (e.g., 10^6 for accuracy).
  - Postconditions: Validates entropy quality (e.g., faster convergence due to true randomness); logs results for SoftwareX benchmarks.
- **Creativity Extensions**: 
  - Multi-run averaging to compute variance.
  - Integrate with physics simulations (e.g., quantum walk models using `qutip` if bridged to Python interop).
  - Gamify: "Quantum Lottery" mode where users simulate draws, proving non-bias.
- **Rationale**: Monte Carlo tests highlight quantum superiority (e.g., passing statistical tests like DIEHARDER implicitly through better results), providing empirical evidence for the bridge's value in scientific computing.

## 6. Risks and Assumptions
- **Risks**: Push failures due to network issues (mitigated by queuing); security misconfigurations (mitigated by audits).
- **Assumptions**: Stable network boundary; no malicious insiders.

## 7. Appendices
### 7.1 Glossary
- QRNG: Quantum Random Number Generator.
- Data Diode: Unidirectional data flow enforcer.
- Accumulation: Buffering to handle rate disparities.
- MCP: Model Context Protocol - standardized protocol for AI agent tool integration.
- EC: Entropy Collector - Internal component that fetches and pushes data.
- EG: Entropy Gateway - External component that receives, buffers, and serves data.
- Push-Based Mode: Deployment architecture with separated EC and EG for network isolation.
- Direct Access Mode: Unified deployment where EG directly accesses the Quantis appliance.

### 7.2 Change History
- v1.0: Initial draft based on discussions.
- v1.1: Added dual deployment mode support (push-based and direct access) and MCP server integration for AI agent accessibility.
- v1.2: Renamed Component A to "Entropy Collector (EC)" and Component B to "Entropy Gateway (EG)" for improved semantic clarity.

This document shall be reviewed and updated iteratively during development. For implementation, proceed to prototyping Entropy Collector (EC) first.