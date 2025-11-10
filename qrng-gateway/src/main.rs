//! Entropy Gateway - External Component for QRNG Data Diode
//!
//! The Entropy Gateway serves as the public-facing component that receives entropy
//! from the Collector (push mode) or fetches directly from the appliance (direct mode).
//!
//! # Features
//!
//! - REST API for entropy distribution
//! - Two deployment modes: push-based and direct access
//! - API key authentication
//! - Rate limiting per client
//! - Prometheus metrics
//! - Health monitoring

use anyhow::{Context, Result};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use qrng_core::{
    buffer::EntropyBuffer,
    config::{DeploymentMode, GatewayConfig},
    crypto::{encode_base64, encode_hex, PacketSigner},
    fetcher::{EntropyFetcher, FetcherConfig},
    metrics::Metrics,
    protocol::{EncodingFormat, EntropyPacket, GatewayStatus, HealthStatus},
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "qrng-gateway")]
#[command(about = "QRNG Gateway - Serves quantum random data via REST API", long_about = None)]
struct Args {
    /// Path to configuration file (ignored if --env-mode is set)
    #[arg(short, long, default_value = "config/gateway-push.yaml")]
    config: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Load configuration from environment variables instead of file
    #[arg(long, default_value = "false")]
    env_mode: bool,
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

/// GET /api/random - Serve random entropy
async fn serve_random(
    State(state): State<AppState>,
    Query(params): Query<RandomQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let start = Instant::now();

    // Extract API key (from header or query param)
    let api_key = if let Some(key) = params.api_key {
        if state.config.api_keys.contains(&key) {
            key
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    } else {
        extract_api_key(&headers, &state.config)?
    };

    // Rate limiting
    if !state.rate_limiter.check(&api_key) {
        state.metrics.record_request_failure();
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Validate request size
    if params.bytes == 0 || params.bytes > qrng_core::MAX_REQUEST_SIZE {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Parse encoding
    let encoding = EncodingFormat::parse(&params.encoding)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Get entropy from buffer
    let data = state.buffer.pop(params.bytes)
        .ok_or_else(|| {
            state.metrics.record_request_failure();
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
    headers: HeaderMap,
) -> Result<Json<GatewayStatus>, StatusCode> {
    extract_api_key(&headers, &state.config)?;

    let mode = match state.config.deployment_mode {
        DeploymentMode::PushBased => "push_based",
        DeploymentMode::DirectAccess => "direct_access",
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

    Ok(Json(GatewayStatus {
        status,
        deployment_mode: mode.to_string(),
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

/// GET /metrics - Prometheus metrics
async fn get_metrics(State(state): State<AppState>) -> String {
    state.metrics.prometheus_format()
}

/// POST /push - Receive entropy packets (push mode only)
async fn receive_push(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> StatusCode {
    // Only available in push mode
    if state.config.deployment_mode != DeploymentMode::PushBased {
        return StatusCode::METHOD_NOT_ALLOWED;
    }

    let signer = match &state.signer {
        Some(s) => s,
        None => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Deserialize packet
    let packet = match EntropyPacket::from_msgpack(&body) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to deserialize packet: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Verify signature
    match signer.verify_packet(&packet) {
        Ok(true) => {}
        Ok(false) => {
            warn!("Invalid packet signature");
            return StatusCode::UNAUTHORIZED;
        }
        Err(e) => {
            error!("Signature verification error: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    // Verify checksum if present
    if !packet.verify_checksum() {
        warn!("Checksum mismatch");
        return StatusCode::BAD_REQUEST;
    }

    // Check freshness
    if let Some(ttl) = state.config.buffer_ttl() {
        if packet.is_stale(ttl) {
            warn!("Packet is stale");
            return StatusCode::BAD_REQUEST;
        }
    }

    // Push to buffer
    match state.buffer.push(packet.data) {
        Ok(bytes) => {
            info!(
                "Received packet #{}, {} bytes, buffer: {:.1}%",
                packet.sequence,
                bytes,
                state.buffer.fill_percent()
            );
            StatusCode::OK
        }
        Err(e) => {
            error!("Failed to push to buffer: {}", e);
            StatusCode::INSUFFICIENT_STORAGE
        }
    }
}

/// Direct access mode: fetch loop
async fn direct_fetch_loop(state: AppState) {
    let direct_config = match &state.config.direct_mode {
        Some(c) => c,
        None => {
            error!("Direct mode config missing");
            return;
        }
    };

    let fetcher_config = FetcherConfig::new(
        direct_config.appliance_url.parse().expect("Invalid URL"),
        direct_config.fetch_chunk_size,
    );

    let fetcher = match EntropyFetcher::new(fetcher_config) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to initialize fetcher: {}", e);
            return;
        }
    };

    let mut ticker = interval(Duration::from_secs(direct_config.fetch_interval_secs));

    info!("Starting direct access fetch loop");

    loop {
        ticker.tick().await;

        match fetcher.fetch().await {
            Ok(data) => {
                state.metrics.record_fetch(data.len());
                if let Err(e) = state.buffer.push(data) {
                    error!("Failed to push to buffer: {}", e);
                } else {
                    info!(
                        "Fetched data, buffer: {}/{} bytes ({:.1}%)",
                        state.buffer.len(),
                        state.buffer.capacity(),
                        state.buffer.fill_percent()
                    );
                }
            }
            Err(e) => {
                state.metrics.record_fetch_failure();
                error!("Fetch failed: {}", e);
            }
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

    // Load configuration
    let config = if args.env_mode {
        info!("Loading configuration from environment variables");
        GatewayConfig::from_env()
            .context("Failed to load configuration from environment")?
    } else {
        info!("Loading configuration from file: {:?}", args.config);
        GatewayConfig::from_file(&args.config)
            .context("Failed to load configuration from file")?
    };

    info!("Deployment mode: {:?}", config.deployment_mode);
    info!("Listen address: {}", config.listen_address);

    // Create buffer
    let buffer = if let Some(ttl) = config.buffer_ttl() {
        EntropyBuffer::with_ttl(config.buffer_size, ttl)
    } else {
        EntropyBuffer::new(config.buffer_size)
    };

    // Create signer for push mode
    let signer = if config.deployment_mode == DeploymentMode::PushBased {
        let key = config.hmac_secret_key.as_ref()
            .context("HMAC key required for push mode")?;
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

    // Start direct fetch loop if in direct mode
    if config.deployment_mode == DeploymentMode::DirectAccess {
        let fetch_state = state.clone();
        tokio::spawn(async move {
            direct_fetch_loop(fetch_state).await;
        });
    }

    // Build router
    let app = Router::new()
        .route("/api/random", get(serve_random))
        .route("/api/status", get(get_status))
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/push", post(receive_push))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Parse listen address
    let addr: std::net::SocketAddr = config.listen_address.parse()
        .context("Invalid listen address")?;

    info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
