//! QRNG MCP Server Binary
//!
//! Runs the MCP server with both SSE and Streamable HTTP transports
//! for integration with Claude Desktop and LM Studio.
//! 
//! Fetches quantum entropy from the QRNG Gateway's REST API.

use qrng_core::buffer::EntropyBuffer;
use qrng_mcp::QrngMcpServer;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, tower::StreamableHttpService,
    tower::StreamableHttpServerConfig,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::{Router, routing::{get, post, delete}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qrng_mcp=info,rmcp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting QRNG MCP Server with SSE and Streamable HTTP transports");

    // Parse configuration from environment
    let bind_addr: SocketAddr = std::env::var("MCP_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;
    
    let gateway_url = std::env::var("QRNG_GATEWAY_URL")
        .unwrap_or_else(|_| "http://qrng-gateway:7764".to_string());
    
    let gateway_api_key = std::env::var("QRNG_GATEWAY_API_KEY")
        .expect("QRNG_GATEWAY_API_KEY must be set");

    tracing::info!("Gateway URL: {}", gateway_url);

    // Create entropy buffer (10 MB capacity)
    let buffer = EntropyBuffer::new(10 * 1024 * 1024);
    
    // Start background task to fetch entropy from gateway
    tracing::info!("Starting entropy fetcher from gateway...");
    let buffer_fetcher = buffer.clone();
    let gateway_url_fetcher = gateway_url.clone();
    let api_key_fetcher = gateway_api_key.clone();
    
    tokio::spawn(async move {
        fetch_entropy_loop(buffer_fetcher, gateway_url_fetcher, api_key_fetcher).await;
    });
    
    // Wait a moment for initial buffer fill
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    tracing::info!(
        "Buffer initialized: {} bytes available",
        buffer.len()
    );

    // Create the service factory for both transports
    let buffer_clone = buffer.clone();
    let gateway_url_clone = gateway_url.clone();
    let gateway_api_key_clone = gateway_api_key.clone();
    let service_factory = move || {
        Ok::<_, std::io::Error>(QrngMcpServer::with_gateway(
            buffer_clone.clone(),
            gateway_url_clone.clone(),
            gateway_api_key_clone.clone(),
        ))
    };

    // Create Streamable HTTP service
    let session_manager = Arc::new(LocalSessionManager::default());
    let streamable_config = StreamableHttpServerConfig {
        sse_keep_alive: Some(std::time::Duration::from_secs(30)),
        stateful_mode: true,
    };
    let streamable_service = StreamableHttpService::new(
        service_factory.clone(),
        session_manager,
        streamable_config,
    );

    // Create router with both Streamable HTTP and legacy SSE endpoints
    let app = Router::new()
        // Streamable HTTP endpoints (2025-06-18 spec)
        .route("/", post({
            let svc = streamable_service.clone();
            move |req| async move {
                let mut svc = svc.clone();
                tower::Service::call(&mut svc, req).await
            }
        }))
        .route("/", get({
            let svc = streamable_service.clone();
            move |req| async move {
                let mut svc = svc.clone();
                tower::Service::call(&mut svc, req).await
            }
        }))
        .route("/", delete({
            let svc = streamable_service.clone();
            move |req| async move {
                let mut svc = svc.clone();
                tower::Service::call(&mut svc, req).await
            }
        }))
        // Legacy SSE endpoints (2024-11-05 spec) for backward compatibility
        .route("/sse", get(legacy_sse_handler))
        .route("/message", post(legacy_message_handler));

    tracing::info!("QRNG MCP Server listening on {}", bind_addr);
    tracing::info!("Streamable HTTP endpoints:");
    tracing::info!("  POST   http://{}/", bind_addr);
    tracing::info!("  GET    http://{}/", bind_addr);
    tracing::info!("  DELETE http://{}/", bind_addr);
    tracing::info!("Legacy SSE endpoints:");
    tracing::info!("  GET    http://{}/sse", bind_addr);
    tracing::info!("  POST   http://{}/message", bind_addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Legacy SSE endpoint handler
async fn legacy_sse_handler(
    _req: axum::http::Request<axum::body::Body>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    tracing::warn!("Legacy /sse endpoint called - not fully implemented");
    Err(axum::http::StatusCode::NOT_IMPLEMENTED)
}

/// Legacy message endpoint handler
async fn legacy_message_handler(
    _req: axum::http::Request<axum::body::Body>,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    tracing::warn!("Legacy /message endpoint called - not fully implemented");
    Err(axum::http::StatusCode::NOT_IMPLEMENTED)
}

/// Continuously fetch entropy from gateway and fill buffer
async fn fetch_entropy_loop(buffer: EntropyBuffer, gateway_url: String, api_key: String) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");
    
    let mut fetch_interval = tokio::time::interval(Duration::from_millis(100));
    let mut backoff_duration = Duration::from_secs(1);
    
    loop {
        fetch_interval.tick().await;
        
        // Check if buffer needs filling
        let fill_percent = buffer.fill_percent();
        if fill_percent > 80.0 {
            continue; // Buffer sufficiently full
        }
        
        // Calculate how much to fetch
        let available = buffer.len();
        let capacity = buffer.capacity();
        let needed = capacity - available;
        let fetch_size = needed.min(65536); // Fetch up to 64KB at a time
        
        if fetch_size < 1024 {
            continue; // Not worth fetching
        }
        
        // Fetch from gateway
        let url = format!("{}/api/random?bytes={}&encoding=hex", gateway_url, fetch_size);
        
        match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(hex_data) => {
                            match hex::decode(&hex_data) {
                                Ok(bytes) => {
                                    if let Err(e) = buffer.push(bytes.clone()) {
                                        tracing::error!("Failed to push to buffer: {}", e);
                                    } else {
                                        tracing::debug!(
                                            "Fetched {} bytes from gateway, buffer: {:.1}%",
                                            bytes.len(),
                                            buffer.fill_percent()
                                        );
                                        // Reset backoff on success
                                        backoff_duration = Duration::from_secs(1);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to decode hex data: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to read response: {}", e);
                        }
                    }
                } else {
                    tracing::warn!(
                        "Gateway returned error: {} - backing off for {:?}",
                        response.status(),
                        backoff_duration
                    );
                    tokio::time::sleep(backoff_duration).await;
                    backoff_duration = (backoff_duration * 2).min(Duration::from_secs(60));
                }
            }
            Err(e) => {
                tracing::error!(
                    "Failed to fetch from gateway: {} - backing off for {:?}",
                    e,
                    backoff_duration
                );
                tokio::time::sleep(backoff_duration).await;
                backoff_duration = (backoff_duration * 2).min(Duration::from_secs(60));
            }
        }
    }
}
