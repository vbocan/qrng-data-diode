//! QRNG MCP Server Binary
//!
//! Runs the MCP server with both SSE and Streamable HTTP transports
//! for integration with Claude Desktop and LM Studio.

use qrng_core::buffer::EntropyBuffer;
use qrng_mcp::QrngMcpServer;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, tower::StreamableHttpService,
    tower::StreamableHttpServerConfig,
};
use std::net::SocketAddr;
use std::sync::Arc;
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

    // Parse bind address from environment or use default
    let bind_addr: SocketAddr = std::env::var("MCP_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;

    // Create entropy buffer (10 MB capacity)
    let buffer = EntropyBuffer::new(10 * 1024 * 1024);
    
    // Pre-fill buffer with test entropy
    tracing::info!("Pre-filling buffer with test entropy...");
    fill_buffer_with_test_entropy(&buffer).await;
    
    tracing::info!(
        "Buffer initialized: {} bytes available",
        buffer.len()
    );

    // Create the service factory for both transports
    let buffer_clone = buffer.clone();
    let service_factory = move || Ok::<_, std::io::Error>(QrngMcpServer::new(buffer_clone.clone()));

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
    // For now, return a placeholder - the SSE server implementation
    // would go here if needed for strict backward compatibility
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

/// Fill buffer with test entropy for demonstration
/// In production, use a real QRNG collector
async fn fill_buffer_with_test_entropy(buffer: &EntropyBuffer) {
    use rand::RngCore;
    
    let mut rng = rand::thread_rng();
    
    // Fill with 5 MB of random data
    const CHUNK_SIZE: usize = 65536;
    const TOTAL_BYTES: usize = 5 * 1024 * 1024;
    
    for _ in 0..(TOTAL_BYTES / CHUNK_SIZE) {
        let mut chunk = vec![0u8; CHUNK_SIZE];
        rng.fill_bytes(&mut chunk);
        
        if let Err(e) = buffer.push(chunk) {
            tracing::error!("Failed to push entropy: {}", e);
            break;
        }
    }
}
