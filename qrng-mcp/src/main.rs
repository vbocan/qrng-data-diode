// SPDX-License-Identifier: MIT
//
// QRNG Data Diode
// Copyright (c) 2025 Valer Bocan, PhD, CSSLP
// Email: valer.bocan@upt.ro
//
// Department of Computer and Information Technology
// Politehnica University of Timisoara
//
// https://github.com/vbocan/qrng-data-diode

//! QRNG MCP Server Binary
//!
//! Runs the MCP server with both SSE and Streamable HTTP transports
//! for integration with Claude Desktop and LM Studio.
//! 
//! This is a thin AI-friendly wrapper around the QRNG Gateway API.
//! It has no local buffer or QRNG logic - all operations are delegated to the gateway.

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

    // Parse configuration from environment
    let bind_addr: SocketAddr = std::env::var("MCP_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;
    
    let gateway_url = std::env::var("QRNG_GATEWAY_URL")
        .unwrap_or_else(|_| "http://qrng-gateway:7764".to_string());
    
    let gateway_api_key = std::env::var("QRNG_GATEWAY_API_KEY")
        .expect("QRNG_GATEWAY_API_KEY must be set");

    tracing::info!("Gateway URL: {}", gateway_url);
    tracing::info!("MCP server will forward all requests to the gateway");

    // Create the service factory for both transports
    let gateway_url_clone = gateway_url.clone();
    let gateway_api_key_clone = gateway_api_key.clone();
    let service_factory = move || {
        Ok::<_, std::io::Error>(QrngMcpServer::new(
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
