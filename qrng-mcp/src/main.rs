//! QRNG MCP Server Binary
//!
//! Runs the MCP server with SSE transport for integration with
//! Claude Desktop and LM Studio.

use qrng_core::buffer::EntropyBuffer;
use qrng_mcp::QrngMcpServer;
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    tracing::info!("Starting QRNG MCP Server with SSE transport");

    // Parse bind address from environment or use default
    let bind_addr: SocketAddr = std::env::var("MCP_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
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

    // Create SSE server with configuration
    let config = SseServerConfig {
        bind: bind_addr,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: CancellationToken::new(),
        sse_keep_alive: Some(std::time::Duration::from_secs(30)),
    };

    let sse_server = SseServer::serve_with_config(config).await?;
    
    tracing::info!("QRNG MCP Server listening on {}", bind_addr);
    tracing::info!("SSE endpoint: http://{}/sse", bind_addr);
    tracing::info!("POST endpoint: http://{}/message", bind_addr);

    // Serve with the handler - creates a new handler for each connection
    let ct = sse_server.with_service(move || {
        QrngMcpServer::new(buffer.clone())
    });

    // Wait for cancellation
    ct.cancelled().await;
    
    tracing::info!("QRNG MCP Server shutting down");

    Ok(())
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
