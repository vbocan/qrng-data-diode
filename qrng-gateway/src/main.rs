// SPDX-License-Identifier: MIT
//
// QRNG Data Diode: High-Performance Quantum Entropy Bridge
// Copyright (c) 2025 Valer Bocan, PhD, CSSLP
// Email: valer.bocan@upt.ro
//
// Department of Computer and Information Technology
// Politehnica University of Timisoara
//
// https://github.com/vbocan/qrng-data-diode

//! Entropy Gateway - External Component for QRNG Data Diode
//!
//! The Entropy Gateway serves as the public-facing component that receives entropy
//! from the Collector via push-based delivery.
//!
//! # Features
//!
//! - REST API for entropy distribution
//! - API key authentication
//! - Rate limiting per client
//! - Prometheus metrics
//! - Health monitoring

use anyhow::{Context, Result};
use axum::{
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use qrng_core::{
    buffer::EntropyBuffer,
    config::GatewayConfig,
    crypto::{encode_base64, encode_hex, PacketSigner},
    metrics::Metrics,
    protocol::{EncodingFormat, EntropyPacket, GatewayStatus, HealthStatus},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "qrng-gateway")]
#[command(about = "QRNG Gateway - Serves quantum random data via REST API", long_about = None)]
struct Args {
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

/// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    config: GatewayConfig,
    buffer: EntropyBuffer,
    metrics: Metrics,
    signer: Option<PacketSigner>,
    start_time: Instant,
    rate_limiter: Arc<RateLimiter>,
}

/// Application error type
struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl From<StatusCode> for AppError {
    fn from(status: StatusCode) -> Self {
        AppError(status, status.to_string())
    }
}

/// Simple token-bucket rate limiter
struct RateLimiter {
    buckets: parking_lot::RwLock<std::collections::HashMap<String, TokenBucket>>,
    rate: u32,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(rate: u32) -> Self {
        Self {
            buckets: parking_lot::RwLock::new(std::collections::HashMap::new()),
            rate,
        }
    }

    fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write();
        let bucket = buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
            tokens: self.rate as f64,
            last_refill: Instant::now(),
        });

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.rate as f64).min(self.rate as f64);
        bucket.last_refill = now;

        // Try to consume a token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Extract and validate API key from request
fn extract_api_key(headers: &HeaderMap, config: &GatewayConfig) -> Result<String, StatusCode> {
    // Try Authorization header first
    if let Some(auth) = headers.get("authorization") {
        let auth_str = auth.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
        if let Some(key) = auth_str.strip_prefix("Bearer ") {
            if config.api_keys.contains(&key.to_string()) {
                return Ok(key.to_string());
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Extract User-Agent from headers
fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// Mask API key for logging (show last 4 chars only)
fn mask_api_key(key: &str) -> String {
    if key.len() <= 4 {
        "****".to_string()
    } else {
        format!("****{}", &key[key.len() - 4..])
    }
}

/// Log client connection details
fn log_client_request(
    ip: SocketAddr,
    user_agent: &str,
    endpoint: &str,
    api_key: &str,
    request_info: &str,
    status: StatusCode,
) {
    let masked_key = mask_api_key(api_key);
    info!(
        client_ip = %ip,
        user_agent = %user_agent,
        endpoint = %endpoint,
        api_key = %masked_key,
        request = %request_info,
        status = %status.as_u16(),
        "Client request"
    );
}

/// Query parameters for /api/random endpoint
#[derive(serde::Deserialize)]
struct RandomQuery {
    bytes: usize,
    #[serde(default = "default_encoding")]
    encoding: String,
    #[serde(default)]
    api_key: Option<String>,
}

fn default_encoding() -> String {
    "hex".to_string()
}

/// Query parameters for /api/integers endpoint
#[derive(serde::Deserialize)]
struct IntegersQuery {
    count: usize,
    #[serde(default = "default_min")]
    min: i64,
    #[serde(default = "default_max")]
    max: i64,
    #[serde(default)]
    api_key: Option<String>,
}

fn default_min() -> i64 {
    0
}

fn default_max() -> i64 {
    100
}

/// Query parameters for /api/floats endpoint
#[derive(serde::Deserialize)]
struct FloatsQuery {
    count: usize,
    #[serde(default)]
    api_key: Option<String>,
}

/// Query parameters for /api/uuid endpoint
#[derive(serde::Deserialize)]
struct UuidQuery {
    #[serde(default = "default_uuid_count")]
    count: usize,
    #[serde(default)]
    api_key: Option<String>,
}

fn default_uuid_count() -> usize {
    1
}

/// Query parameters for /api/status endpoint
#[derive(serde::Deserialize)]
struct StatusQuery {
    #[serde(default)]
    api_key: Option<String>,
}

/// GET /api/random - Serve random entropy
async fn serve_random(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<RandomQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let user_agent = extract_user_agent(&headers);

    // Extract API key (from header or query param)
    let api_key = if let Some(key) = params.api_key {
        if state.config.api_keys.contains(&key) {
            key
        } else {
            log_client_request(
                addr,
                &user_agent,
                "/api/random",
                "",
                &format!("bytes={}", params.bytes),
                StatusCode::UNAUTHORIZED,
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        match extract_api_key(&headers, &state.config) {
            Ok(key) => key,
            Err(status) => {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/random",
                    "",
                    &format!("bytes={}", params.bytes),
                    status,
                );
                return Err(status);
            }
        }
    };

    // Rate limiting
    if !state.rate_limiter.check(&api_key) {
        state.metrics.record_request_failure();
        log_client_request(
            addr,
            &user_agent,
            "/api/random",
            &api_key,
            &format!("bytes={}", params.bytes),
            StatusCode::TOO_MANY_REQUESTS,
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Validate request size
    if params.bytes == 0 || params.bytes > qrng_core::MAX_REQUEST_SIZE {
        log_client_request(
            addr,
            &user_agent,
            "/api/random",
            &api_key,
            &format!("bytes={} (invalid)", params.bytes),
            StatusCode::BAD_REQUEST,
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Parse encoding
    let encoding = match EncodingFormat::parse(&params.encoding) {
        Some(e) => e,
        None => {
            log_client_request(
                addr,
                &user_agent,
                "/api/random",
                &api_key,
                &format!("bytes={} encoding={} (invalid)", params.bytes, params.encoding),
                StatusCode::BAD_REQUEST,
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Get entropy from buffer
    let data = state.buffer.pop(params.bytes)
        .ok_or_else(|| {
            state.metrics.record_request_failure();
            log_client_request(
                addr,
                &user_agent,
                "/api/random",
                &api_key,
                &format!("bytes={} encoding={}", params.bytes, params.encoding),
                StatusCode::SERVICE_UNAVAILABLE,
            );
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Encode based on format
    let (body, content_type) = match encoding {
        EncodingFormat::Binary => (data.to_vec(), encoding.mime_type()),
        EncodingFormat::Hex => (encode_hex(&data).into_bytes(), encoding.mime_type()),
        EncodingFormat::Base64 => (encode_base64(&data).into_bytes(), encoding.mime_type()),
    };

    // Record metrics
    let latency = start.elapsed().as_micros() as u64;
    state.metrics.record_request(params.bytes, latency);

    // Log successful request
    log_client_request(
        addr,
        &user_agent,
        "/api/random",
        &api_key,
        &format!("bytes={} encoding={}", params.bytes, params.encoding),
        StatusCode::OK,
    );

    Ok((
        StatusCode::OK,
        [(hyper::header::CONTENT_TYPE, content_type)],
        body,
    )
        .into_response())
}

/// GET /api/status - System status
async fn get_status(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<StatusQuery>,
    headers: HeaderMap,
) -> Result<Json<GatewayStatus>, StatusCode> {
    let user_agent = extract_user_agent(&headers);

    // Extract API key (from header or query param)
    let api_key = if let Some(key) = params.api_key {
        if state.config.api_keys.contains(&key) {
            key
        } else {
            log_client_request(
                addr,
                &user_agent,
                "/api/status",
                "",
                "status_check",
                StatusCode::UNAUTHORIZED,
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        match extract_api_key(&headers, &state.config) {
            Ok(key) => key,
            Err(status) => {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/status",
                    "",
                    "status_check",
                    status,
                );
                return Err(status);
            }
        }
    };

    let fill_percent = state.buffer.fill_percent();
    let status = if fill_percent < 10.0 {
        HealthStatus::Unhealthy
    } else if fill_percent < 30.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    let mut warnings = Vec::new();
    if fill_percent < 10.0 {
        warnings.push("Buffer critically low".to_string());
    }
    if let Some(age) = state.buffer.freshness_seconds() {
        if age > 300 {
            warnings.push(format!("Data is {} seconds old", age));
        }
    }

    log_client_request(
        addr,
        &user_agent,
        "/api/status",
        &api_key,
        &format!("buffer_fill={:.1}%", fill_percent),
        StatusCode::OK,
    );

    Ok(Json(GatewayStatus {
        status,        
        buffer_fill_percent: fill_percent,
        buffer_bytes_available: state.buffer.len(),
        last_data_received: state.buffer.oldest_timestamp(),
        data_freshness_seconds: state.buffer.freshness_seconds(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        total_requests_served: state.metrics.requests_total(),
        total_bytes_served: state.metrics.bytes_served(),
        requests_per_second: state.metrics.requests_per_second(),
        warnings,
    }))
}

/// GET /health - Simple health check
async fn health_check(State(state): State<AppState>) -> StatusCode {
    if state.buffer.fill_percent() > 5.0 {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

/// GET /api/integers - Generate random integers in range
async fn serve_integers(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<IntegersQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let user_agent = extract_user_agent(&headers);

    // Extract and validate API key
    let api_key = if let Some(key) = params.api_key {
        if state.config.api_keys.contains(&key) {
            key
        } else {
            log_client_request(
                addr,
                &user_agent,
                "/api/integers",
                "",
                &format!("count={} min={} max={}", params.count, params.min, params.max),
                StatusCode::UNAUTHORIZED,
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        match extract_api_key(&headers, &state.config) {
            Ok(key) => key,
            Err(status) => {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/integers",
                    "",
                    &format!("count={} min={} max={}", params.count, params.min, params.max),
                    status,
                );
                return Err(status);
            }
        }
    };

    // Rate limiting
    if !state.rate_limiter.check(&api_key) {
        state.metrics.record_request_failure();
        log_client_request(
            addr,
            &user_agent,
            "/api/integers",
            &api_key,
            &format!("count={} min={} max={}", params.count, params.min, params.max),
            StatusCode::TOO_MANY_REQUESTS,
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Validate parameters
    if params.count == 0 || params.count > 1000 {
        log_client_request(
            addr,
            &user_agent,
            "/api/integers",
            &api_key,
            &format!("count={} (invalid)", params.count),
            StatusCode::BAD_REQUEST,
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    if params.min >= params.max {
        log_client_request(
            addr,
            &user_agent,
            "/api/integers",
            &api_key,
            &format!("min={} max={} (invalid)", params.min, params.max),
            StatusCode::BAD_REQUEST,
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let range = (params.max - params.min + 1) as u64;

    // Get entropy from buffer (8 bytes per integer)
    let bytes_needed = params.count * 8;
    let data = state.buffer.pop(bytes_needed)
        .ok_or_else(|| {
            state.metrics.record_request_failure();
            log_client_request(
                addr,
                &user_agent,
                "/api/integers",
                &api_key,
                &format!("count={} min={} max={}", params.count, params.min, params.max),
                StatusCode::SERVICE_UNAVAILABLE,
            );
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Convert bytes to integers
    let mut integers = Vec::with_capacity(params.count);
    for chunk in data.chunks_exact(8) {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(chunk);
        let value = u64::from_le_bytes(bytes);
        let result = params.min + (value % range) as i64;
        integers.push(result);
    }

    // Record metrics
    let latency = start.elapsed().as_micros() as u64;
    state.metrics.record_request(bytes_needed, latency);

    // Log successful request
    log_client_request(
        addr,
        &user_agent,
        "/api/integers",
        &api_key,
        &format!("count={} min={} max={}", params.count, params.min, params.max),
        StatusCode::OK,
    );

    // Return as JSON array
    Ok((
        StatusCode::OK,
        [(hyper::header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&integers).unwrap(),
    )
        .into_response())
}

/// GET /api/floats - Generate random floats in [0, 1)
async fn serve_floats(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<FloatsQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let user_agent = extract_user_agent(&headers);

    // Extract and validate API key
    let api_key = if let Some(key) = params.api_key {
        if state.config.api_keys.contains(&key) {
            key
        } else {
            log_client_request(
                addr,
                &user_agent,
                "/api/floats",
                "",
                &format!("count={}", params.count),
                StatusCode::UNAUTHORIZED,
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        match extract_api_key(&headers, &state.config) {
            Ok(key) => key,
            Err(status) => {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/floats",
                    "",
                    &format!("count={}", params.count),
                    status,
                );
                return Err(status);
            }
        }
    };

    // Rate limiting
    if !state.rate_limiter.check(&api_key) {
        state.metrics.record_request_failure();
        log_client_request(
            addr,
            &user_agent,
            "/api/floats",
            &api_key,
            &format!("count={}", params.count),
            StatusCode::TOO_MANY_REQUESTS,
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Validate parameters
    if params.count == 0 || params.count > 1000 {
        log_client_request(
            addr,
            &user_agent,
            "/api/floats",
            &api_key,
            &format!("count={} (invalid)", params.count),
            StatusCode::BAD_REQUEST,
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get entropy from buffer (8 bytes per float)
    let bytes_needed = params.count * 8;
    let data = state.buffer.pop(bytes_needed)
        .ok_or_else(|| {
            state.metrics.record_request_failure();
            log_client_request(
                addr,
                &user_agent,
                "/api/floats",
                &api_key,
                &format!("count={}", params.count),
                StatusCode::SERVICE_UNAVAILABLE,
            );
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Convert bytes to floats using proper precision
    let mut floats = Vec::with_capacity(params.count);
    for chunk in data.chunks_exact(8) {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(chunk);
        let random_u64 = u64::from_le_bytes(bytes);
        // Use only top 53 bits to avoid rounding bias (same as Monte Carlo)
        let float = (random_u64 >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        floats.push(float);
    }

    // Record metrics
    let latency = start.elapsed().as_micros() as u64;
    state.metrics.record_request(bytes_needed, latency);

    // Log successful request
    log_client_request(
        addr,
        &user_agent,
        "/api/floats",
        &api_key,
        &format!("count={}", params.count),
        StatusCode::OK,
    );

    // Return as JSON array
    Ok((
        StatusCode::OK,
        [(hyper::header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&floats).unwrap(),
    )
        .into_response())
}

/// GET /api/uuid - Generate UUID v4
async fn serve_uuid(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<UuidQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let user_agent = extract_user_agent(&headers);

    // Extract and validate API key
    let api_key = if let Some(key) = params.api_key {
        if state.config.api_keys.contains(&key) {
            key
        } else {
            log_client_request(
                addr,
                &user_agent,
                "/api/uuid",
                "",
                &format!("count={}", params.count),
                StatusCode::UNAUTHORIZED,
            );
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        match extract_api_key(&headers, &state.config) {
            Ok(key) => key,
            Err(status) => {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/uuid",
                    "",
                    &format!("count={}", params.count),
                    status,
                );
                return Err(status);
            }
        }
    };

    // Rate limiting
    if !state.rate_limiter.check(&api_key) {
        state.metrics.record_request_failure();
        log_client_request(
            addr,
            &user_agent,
            "/api/uuid",
            &api_key,
            &format!("count={}", params.count),
            StatusCode::TOO_MANY_REQUESTS,
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Validate parameters
    if params.count == 0 || params.count > 100 {
        log_client_request(
            addr,
            &user_agent,
            "/api/uuid",
            &api_key,
            &format!("count={} (invalid)", params.count),
            StatusCode::BAD_REQUEST,
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get entropy from buffer (16 bytes per UUID)
    let bytes_needed = params.count * 16;
    let data = state.buffer.pop(bytes_needed)
        .ok_or_else(|| {
            state.metrics.record_request_failure();
            log_client_request(
                addr,
                &user_agent,
                "/api/uuid",
                &api_key,
                &format!("count={}", params.count),
                StatusCode::SERVICE_UNAVAILABLE,
            );
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Convert bytes to UUIDs
    let mut uuids = Vec::with_capacity(params.count);
    for chunk in data.chunks_exact(16) {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(chunk);
        
        // Set version (4) and variant (RFC 4122)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        
        let uuid = uuid::Uuid::from_bytes(bytes);
        uuids.push(uuid.to_string());
    }

    // Record metrics
    let latency = start.elapsed().as_micros() as u64;
    state.metrics.record_request(bytes_needed, latency);

    // Log successful request
    log_client_request(
        addr,
        &user_agent,
        "/api/uuid",
        &api_key,
        &format!("count={}", params.count),
        StatusCode::OK,
    );

    // Return as single string or JSON array
    let response_body = if params.count == 1 {
        uuids[0].clone()
    } else {
        serde_json::to_string(&uuids).unwrap()
    };

    Ok((
        StatusCode::OK,
        [(hyper::header::CONTENT_TYPE, if params.count == 1 { "text/plain" } else { "application/json" })],
        response_body,
    )
        .into_response())
}

/// GET /metrics - Prometheus metrics
async fn get_metrics(State(state): State<AppState>) -> String {
    state.metrics.prometheus_format()
}

/// Monte Carlo test parameters
#[derive(Debug, Deserialize)]
struct MonteCarloParams {
    #[serde(default = "default_iterations")]
    iterations: u64,
    #[serde(default)]
    api_key: Option<String>,
}

fn default_iterations() -> u64 {
    1_000_000
}

/// Monte Carlo test results
#[derive(Debug, Serialize)]
struct MonteCarloResult {
    estimated_pi: f64,
    error: f64,
    error_percent: f64,
    iterations: u64,
    convergence_rate: String,
    quality_assessment: String,
    note: String,
    quantum_vs_pseudo: Option<PseudoComparison>,
}

#[derive(Debug, Serialize)]
struct PseudoComparison {
    quantum_error: f64,
    pseudo_error: f64,
    improvement_factor: f64,
}

/// GET /api/test/monte-carlo - Run Monte Carlo π estimation test
async fn monte_carlo_test(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<MonteCarloParams>,
) -> Result<Json<MonteCarloResult>, AppError> {
    let user_agent = extract_user_agent(&headers);

    // Extract and validate API key
    let api_key = match params.api_key {
        Some(ref key) => {
            if state.config.api_keys.contains(key) {
                key.clone()
            } else {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/test/monte-carlo",
                    "",
                    &format!("iterations={}", params.iterations),
                    StatusCode::UNAUTHORIZED,
                );
                return Err(AppError(StatusCode::UNAUTHORIZED, "Invalid API key".to_string()));
            }
        }
        None => match extract_api_key(&headers, &state.config) {
            Ok(key) => key,
            Err(status) => {
                log_client_request(
                    addr,
                    &user_agent,
                    "/api/test/monte-carlo",
                    "",
                    &format!("iterations={}", params.iterations),
                    status,
                );
                return Err(AppError(status, "Authentication required".to_string()));
            }
        },
    };

    // Rate limiting
    if !state.rate_limiter.check(&api_key) {
        log_client_request(
            addr,
            &user_agent,
            "/api/test/monte-carlo",
            &api_key,
            &format!("iterations={}", params.iterations),
            StatusCode::TOO_MANY_REQUESTS,
        );
        return Err(AppError(StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()));
    }

    // Validate iterations
    const MAX_ITERATIONS: u64 = 10_000_000;
    if params.iterations == 0 || params.iterations > MAX_ITERATIONS {
        log_client_request(
            addr,
            &user_agent,
            "/api/test/monte-carlo",
            &api_key,
            &format!("iterations={} (invalid)", params.iterations),
            StatusCode::BAD_REQUEST,
        );
        return Err(AppError(
            StatusCode::BAD_REQUEST,
            format!("iterations must be between 1 and {}", MAX_ITERATIONS),
        ));
    }

    info!("Running Monte Carlo test with {} iterations", params.iterations);

    // Generate random floats from quantum source
    // Monte Carlo needs 2 floats (x, y) per iteration
    let bytes_needed = (params.iterations * 16) as usize; // 16 bytes per iteration (2 × f64)
    let data = state.buffer.pop(bytes_needed).ok_or_else(|| {
        AppError(
            StatusCode::INSUFFICIENT_STORAGE,
            "Insufficient entropy in buffer".to_string(),
        )
    })?;

    // Convert bytes to floats in [0,1)
    let mut floats = Vec::with_capacity((params.iterations * 2) as usize);
    for chunk in data.chunks_exact(8) {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(chunk);
        let random_u64 = u64::from_be_bytes(bytes);
        // Convert to float in [0, 1) using proper precision
        // Use only top 53 bits to avoid rounding bias
        let float = (random_u64 >> 11) as f64 * (1.0 / (1u64 << 53) as f64);
        floats.push(float);
    }

    // Perform Monte Carlo π estimation
    let quantum_pi = estimate_pi(&floats);
    let quantum_error = (quantum_pi - std::f64::consts::PI).abs();
    let quantum_error_percent = (quantum_error / std::f64::consts::PI) * 100.0;

    // Assess convergence rate
    let convergence_rate = if quantum_error_percent < 0.01 {
        "excellent".to_string()
    } else if quantum_error_percent < 0.1 {
        "good".to_string()
    } else if quantum_error_percent < 1.0 {
        "fair".to_string()
    } else {
        "poor".to_string()
    };

    let quality_assessment = if quantum_error_percent < 0.1 {
        "high_quality".to_string()
    } else if quantum_error_percent < 1.0 {
        "acceptable".to_string()
    } else {
        "poor_quality".to_string()
    };

    // Compare with pseudo-random (for statistical demonstration only)
    // Note: Pseudo-random can occasionally produce better Monte Carlo estimates
    // due to statistical variance, but lacks cryptographic unpredictability
    let comparison = if params.iterations <= 1_000_000 {
        // Generate pseudo-random for comparison
        use rand::Rng;
        let mut rng = rand::rng();
        // Need 2 floats per iteration (x, y coordinates)
        let pseudo_floats: Vec<f64> = (0..(params.iterations * 2))
            .map(|_| rng.random::<f64>())
            .collect();
        let pseudo_pi = estimate_pi(&pseudo_floats);
        let pseudo_error = (pseudo_pi - std::f64::consts::PI).abs();

        Some(PseudoComparison {
            quantum_error,
            pseudo_error,
            improvement_factor: if pseudo_error > 0.0 {
                pseudo_error / quantum_error.max(1e-10)
            } else {
                1.0
            },
        })
    } else {
        None
    };

    info!(
        "Monte Carlo test completed: π ≈ {:.6}, error: {:.6} ({:.4}%)",
        quantum_pi, quantum_error, quantum_error_percent
    );

    // Log successful request
    log_client_request(
        addr,
        &user_agent,
        "/api/test/monte-carlo",
        &api_key,
        &format!("iterations={}", params.iterations),
        StatusCode::OK,
    );

    Ok(Json(MonteCarloResult {
        estimated_pi: quantum_pi,
        error: quantum_error,
        error_percent: quantum_error_percent,
        iterations: params.iterations,
        convergence_rate,
        quality_assessment,
        note: "Monte Carlo tests measure statistical uniformity, not cryptographic security. Both quantum and pseudo-random can pass these tests, but only quantum provides true unpredictability.".to_string(),
        quantum_vs_pseudo: comparison,
    }))
}

/// Estimate π using Monte Carlo method
///
/// Uses pairs of random numbers as (x, y) coordinates and checks if they fall
/// inside a unit circle. The ratio of points inside vs total approximates π/4.
fn estimate_pi(floats: &[f64]) -> f64 {
    let pairs = floats.len() / 2;
    let mut inside_circle = 0u64;

    for i in 0..pairs {
        let x = floats[i * 2];
        let y = floats[i * 2 + 1];

        // Check if point (x, y) is inside unit circle
        if x * x + y * y <= 1.0 {
            inside_circle += 1;
        }
    }

    // π/4 ≈ inside_circle / total_points
    // π ≈ 4 * inside_circle / total_points
    4.0 * (inside_circle as f64) / (pairs as f64)
}

/// POST /push - Receive entropy packets (push mode only)
async fn receive_push(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> StatusCode {
    let user_agent = extract_user_agent(&headers);
    
    let signer = match &state.signer {
        Some(s) => s,
        None => {
            warn!(
                client_ip = %addr,
                user_agent = %user_agent,
                endpoint = "/push",
                "Push endpoint called but HMAC signer not configured"
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    // Deserialize packet
    let packet = match EntropyPacket::from_msgpack(&body) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                client_ip = %addr,
                user_agent = %user_agent,
                endpoint = "/push",
                error = %e,
                "Failed to deserialize entropy packet"
            );
            return StatusCode::BAD_REQUEST;
        }
    };

    // Verify signature
    match signer.verify_packet(&packet) {
        Ok(true) => {}
        Ok(false) => {
            warn!(
                client_ip = %addr,
                user_agent = %user_agent,
                endpoint = "/push",
                sequence = packet.sequence,
                "Invalid packet signature"
            );
            return StatusCode::UNAUTHORIZED;
        }
        Err(e) => {
            error!(
                client_ip = %addr,
                user_agent = %user_agent,
                endpoint = "/push",
                sequence = packet.sequence,
                error = %e,
                "Signature verification error"
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    // Verify checksum if present
    if !packet.verify_checksum() {
        warn!(
            client_ip = %addr,
            user_agent = %user_agent,
            endpoint = "/push",
            sequence = packet.sequence,
            "Checksum mismatch"
        );
        return StatusCode::BAD_REQUEST;
    }

    // Check freshness
    if let Some(ttl) = state.config.buffer_ttl() {
        if packet.is_stale(ttl) {
            warn!(
                client_ip = %addr,
                user_agent = %user_agent,
                endpoint = "/push",
                sequence = packet.sequence,
                "Packet is stale"
            );
            return StatusCode::BAD_REQUEST;
        }
    }

    // Push to buffer
    match state.buffer.push(packet.data.clone()) {
        Ok(bytes) => {
            if bytes == 0 {
                warn!(
                    client_ip = %addr,
                    user_agent = %user_agent,
                    endpoint = "/push",
                    sequence = packet.sequence,
                    buffer_fill_percent = state.buffer.fill_percent(),
                    "Discarded packet, buffer full"
                );
                StatusCode::INSUFFICIENT_STORAGE
            } else if bytes < packet.data.len() {
                info!(
                    client_ip = %addr,
                    user_agent = %user_agent,
                    endpoint = "/push",
                    sequence = packet.sequence,
                    bytes_stored = bytes,
                    bytes_total = packet.data.len(),
                    buffer_fill_percent = state.buffer.fill_percent(),
                    "Received packet (partial)"
                );
                StatusCode::OK
            } else {
                info!(
                    client_ip = %addr,
                    user_agent = %user_agent,
                    endpoint = "/push",
                    sequence = packet.sequence,
                    bytes = bytes,
                    buffer_fill_percent = state.buffer.fill_percent(),
                    "Received packet"
                );
                StatusCode::OK
            }
        }
        Err(e) => {
            error!(
                client_ip = %addr,
                user_agent = %user_agent,
                endpoint = "/push",
                sequence = packet.sequence,
                error = %e,
                "Failed to push to buffer"
            );
            StatusCode::INSUFFICIENT_STORAGE
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();

    // Initialize tracing
    let log_level = args.log_level.parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .json()
        .init();

    info!("QRNG Gateway v{}", env!("CARGO_PKG_VERSION"));
    info!("The gateway acts as a data diode for the Quantis Appliance and receives pushed data from the collector.");
    info!("Developed by Valer BOCAN, PhD, CSSLP - www.bocan.ro");

    // Load configuration from environment variables
    info!("Loading configuration from environment variables");
    let config = GatewayConfig::from_env()
        .context("Failed to load configuration from environment")?;
    
    info!("Listen address: {}", config.listen_address);

    // Create buffer with overflow policy
    let buffer = if let Some(ttl) = config.buffer_ttl() {
        EntropyBuffer::with_ttl(config.buffer_size, ttl)
            .with_overflow_policy(config.overflow_policy())
    } else {
        EntropyBuffer::new(config.buffer_size)
            .with_overflow_policy(config.overflow_policy())
    };

    info!("Buffer overflow policy: {:?}", config.overflow_policy());

    // Create signer for push mode
    let signer = if let Some(key) = config.hmac_secret_key.as_ref() {
        let key_bytes = hex::decode(key)
            .context("Invalid HMAC key (must be hex-encoded)")?;
        Some(PacketSigner::new(key_bytes))
    } else {
        None
    };

    // Create application state
    let state = AppState {
        config: config.clone(),
        buffer: buffer.clone(),
        metrics: Metrics::new(),
        signer,
        start_time: Instant::now(),
        rate_limiter: Arc::new(RateLimiter::new(config.rate_limit_per_second)),
    };

    // Parse listen address
    let addr: SocketAddr = config.listen_address.parse()
        .context("Invalid listen address")?;

    // Create cancellation token for graceful shutdown
    let cancel_token = CancellationToken::new();
    let cancel_token_signal = cancel_token.clone();

    // Build HTTP router for gateway API
    let app = Router::new()
        .route("/api/random", get(serve_random))
        .route("/api/integers", get(serve_integers))
        .route("/api/floats", get(serve_floats))
        .route("/api/uuid", get(serve_uuid))
        .route("/api/status", get(get_status))
        .route("/api/test/monte-carlo", get(monte_carlo_test))
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/push", post(receive_push))
        .layer(CorsLayer::permissive())
        .with_state(state);

    info!("Gateway server starting on {}", addr);

    // Handle Ctrl+C for graceful shutdown
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down");
                cancel_token_signal.cancel();
            }
            Err(e) => error!("Failed to listen for Ctrl+C: {}", e),
        }
    });

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        cancel_token.cancelled().await;
        info!("Server is shutting down");
    });

    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }

    Ok(())
}
