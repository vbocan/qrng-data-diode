### Implementation Plan: QRNG Bridge Service

This plan outlines the development phases for creating the two core Rust components: the Entropy Collector (EC) and the Entropy Gateway (EG). The development is structured to prioritize core functionality and allow for iterative addition of advanced features.

---

### Phase 1: Project Setup & Core Modules

This phase establishes the project structure and defines common components that will be shared between the EC and EG.

*   **Step 1.1: Create a Rust Workspace**
    *   A monorepo using a Cargo workspace is recommended to manage the two components and shared code.
    *   **`qrng-data-diode/`**
        *   **`Cargo.toml`** (Workspace definition)
        *   **`ec/`**: Crate for the Entropy Collector binary.
        *   **`eg/`**: Crate for the Entropy Gateway binary.
        *   **`common/`**: Crate for shared logic (data structures, configuration models).

*   **Step 1.2: Define Shared Data Structures (in `common` crate)**
    *   Define a `struct` for the data packets pushed from EC to EG. This will include:
        *   `data: Vec<u8>`
        *   `timestamp: u64`
        *   `sequence_number: u64`
        *   `signature: Vec<u8>` (for HMAC signature)
    *   Use `serde` for serialization to a format like JSON or MessagePack.

*   **Step 1.3: Configuration Management (in `common` crate)**
    *   Define `structs` to represent the configuration from a YAML file (Requirement NFR-10).
    *   **`ECConfig`**: `appliance_url`, `fetch_chunk_size`, `fetch_interval`, `buffer_size`, `push_url`, `hmac_secret_key`.
    *   **`EGConfig`**: `listen_address`, `buffer_size`, `api_keys`, `rate_limit`, `deployment_mode` (`Push` or `Direct`), `direct_mode_config`.

---

### Phase 2: Entropy Collector (EC) Implementation

This phase focuses on the internal component responsible for fetching and pushing data.

*   **Step 2.1: Implement the Data Fetching Module**
    *   Use `tokio` for the async runtime and `reqwest` to perform HTTPS GET requests to the Quantis appliance (FR-1).
    *   Create a loop (`tokio::time::interval`) that periodically fetches data based on configuration.

*   **Step 2.2: Implement Data Accumulation**
    *   Use a thread-safe, in-memory buffer, such as `tokio::sync::RwLock<Vec<u8>>`, to accumulate the fetched random data (FR-2).
    *   The buffer size will be configurable.

*   **Step 2.3: Implement the Unidirectional Pushing Module**
    *   Create a second `tokio::time::interval` loop that periodically triggers a push.
    *   This function will:
        1.  Extract a batch of data from the accumulator.
        2.  Create a data packet (as defined in `common`).
        3.  Sign the packet using HMAC-SHA256 with a shared secret (FR-7).
        4.  Send the packet to the EG's push endpoint via an HTTP POST request (FR-3).

*   **Step 2.4: Add Resilience and Logging**
    *   Implement exponential backoff for both fetching and pushing to handle transient network errors (FR-9).
    *   Integrate the `tracing` crate for structured JSON logging (NFR-11).

---

### Phase 3: Entropy Gateway (EG) Implementation

This phase focuses on the external component that serves data to clients.

*   **Step 3.1: Set Up the Web Server and Data Reception**
    *   Use the `axum` web framework to build the server.
    *   Create a `/push` endpoint to receive data from the EC. This handler will:
        1.  Deserialize the incoming data packet.
        2.  Verify the HMAC signature using the shared secret (FR-4).
        3.  If valid, append the random data to the EG's internal buffer.

*   **Step 3.2: Implement Buffer Management**
    *   Create a large, thread-safe in-memory buffer, similar to the EC's but with a larger capacity (FR-4).
    *   Implement FIFO (First-In, First-Out) logic with policies for overflow (discard oldest data) and data freshness (discard data older than a configured time) (FR-6).

*   **Step 3.3: Implement the Public REST API**
    *   **`GET /api/random`**: Serves N bytes from the buffer with configurable encoding (`hex`, `base64`, `binary`) (FR-5).
    *   **`GET /api/status`**: Returns a JSON object with system health, buffer fill level, and the timestamp of the last received push (FR-5).

*   **Step 3.4: Implement Direct Access Mode**
    *   Create a feature module within the `eg` crate that replicates the EC's fetching logic.
    *   In `main.rs`, use the `deployment_mode` from the configuration to conditionally start either the push receiver listener or the direct fetching loop (FR-0.11).

*   **Step 3.5: Implement Security Features**
    *   Create an `axum` middleware for API key authentication on all `/api/*` routes (FR-7).
    *   Implement a rate-limiting middleware (e.g., using a token bucket algorithm) to protect against abuse (FR-8).

---

### Phase 4: Advanced Features, Testing, and Documentation

This final phase adds the innovative extensions and ensures the project is robust and usable.

*   **Step 4.1: Integrate the MCP Server**
    *   Add the `mcp-server-rs` crate or implement the protocol manually.
    *   Expose the required tools (`get_random_bytes`, `get_random_integers`, etc.) that draw from the same entropy buffer (FR-11, FR-12).

*   **Step 4.2: Implement Innovative Extensions (as optional features)**
    *   **Entropy Enhancement**: Add optional post-processing functions (e.g., hashing) that can be applied before serving data (FR-11).
    *   **Monitoring**: Add a `/metrics` endpoint and use the `prometheus` crate to expose metrics for buffer usage, requests, and throughput (FR-14).
    *   **Monte Carlo Test**: Implement the `POST /api/test/monte-carlo` endpoint to validate randomness quality (UC-1).

*   **Step 4.3: Comprehensive Testing**
    *   Write unit tests for business logic (e.g., buffer management, signature verification).
    *   Write integration tests that spin up a mock EC and EG to test the full data pipeline.
    *   Aim for a high code coverage target (e.g., 90%) as specified in NFR-13.

*   **Step 4.4: Finalize Documentation and Deployment**
    *   Create detailed `README.md` files for both `ec` and `eg`, explaining configuration and setup.
    *   Provide example `config.yaml` files for both push-based and direct-access modes.
    *   Write `Dockerfile`s for each component to facilitate containerized deployment (NFR-14).