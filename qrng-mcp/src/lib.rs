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

//! Model Context Protocol (MCP) server for QRNG
//!
//! Official MCP implementation using the rmcp SDK.
//! Provides AI agent integration for quantum random number generation.
//!
//! This is a thin wrapper around the QRNG Gateway API, translating
//! MCP protocol requests into Gateway HTTP API calls.
//!
//! # Tools Provided
//!
//! - `get_random_bytes`: Fetch random bytes
//! - `get_random_integers`: Generate random integers in range
//! - `get_random_floats`: Generate random floats
//! - `get_random_uuid`: Generate UUID v4
//! - `get_status`: Query gateway status
//! - `get_data_quality`: Test random data quality using Monte Carlo simulation

use rmcp::{
    ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    tool, tool_handler, tool_router,
    schemars::JsonSchema,
};
use serde::{Deserialize, Serialize};

/// QRNG MCP Server implementation
/// 
/// This server acts as a thin AI-friendly wrapper around the QRNG Gateway API.
/// It has no local buffer or QRNG logic - all operations are delegated to the gateway.
#[derive(Clone)]
pub struct QrngMcpServer {
    tool_router: ToolRouter<Self>,
    gateway_url: String,
    gateway_api_key: String,
    http_client: reqwest::Client,
}

/// Arguments for get_random_bytes tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetRandomBytesArgs {
    #[schemars(description = "Number of bytes to fetch (1-65536)")]
    pub count: usize,
    #[schemars(description = "Output encoding format: hex or base64")]
    pub encoding: Option<String>,
}

/// Arguments for get_random_integers tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetRandomIntegersArgs {
    #[schemars(description = "Number of integers to generate")]
    pub count: usize,
    #[schemars(description = "Minimum value (inclusive)")]
    pub min: Option<i64>,
    #[schemars(description = "Maximum value (inclusive)")]
    pub max: Option<i64>,
}

/// Arguments for get_random_floats tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetRandomFloatsArgs {
    #[schemars(description = "Number of floats to generate")]
    pub count: usize,
}

/// Arguments for get_random_uuid tool
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetRandomUuidArgs {
    #[schemars(description = "Number of UUIDs to generate")]
    pub count: Option<usize>,
}



#[tool_router]
impl QrngMcpServer {
    /// Create a new QRNG MCP server with gateway connection
    pub fn new(gateway_url: String, gateway_api_key: String) -> Self {
        Self {
            tool_router: Self::tool_router(),
            gateway_url,
            gateway_api_key,
            http_client: reqwest::Client::new(),
        }
    }

    /// Fetch random bytes from quantum entropy source via gateway
    #[tool(description = "Fetch random bytes from quantum entropy source")]
    async fn get_random_bytes(&self, Parameters(args): Parameters<GetRandomBytesArgs>) -> Result<String, ErrorData> {
        // Validate count
        if args.count == 0 || args.count > 65536 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 65536", None));
        }

        let encoding = args.encoding.as_deref().unwrap_or("hex");
        if encoding != "hex" && encoding != "base64" {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Invalid encoding. Must be 'hex' or 'base64'", None));
        }

        // Call gateway API
        let url = format!("{}/api/random?bytes={}&encoding={}", self.gateway_url, args.count, encoding);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))
    }

    /// Generate random integers in specified range via gateway
    #[tool(description = "Generate random integers in specified range")]
    async fn get_random_integers(&self, Parameters(args): Parameters<GetRandomIntegersArgs>) -> Result<String, ErrorData> {
        // Validate count
        if args.count == 0 || args.count > 1000 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 1000", None));
        }

        let min = args.min.unwrap_or(0);
        let max = args.max.unwrap_or(100);

        if min >= max {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Min must be less than max", None));
        }

        // Call gateway API
        let url = format!("{}/api/integers?count={}&min={}&max={}", self.gateway_url, args.count, min, max);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))
    }

    /// Generate random floats in range [0, 1) via gateway
    #[tool(description = "Generate random floats in range [0, 1)")]
    async fn get_random_floats(&self, Parameters(args): Parameters<GetRandomFloatsArgs>) -> Result<String, ErrorData> {
        // Validate count
        if args.count == 0 || args.count > 1000 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 1000", None));
        }

        // Call gateway API
        let url = format!("{}/api/floats?count={}", self.gateway_url, args.count);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))
    }

    /// Generate random UUID v4 via gateway
    #[tool(description = "Generate random UUID v4")]
    async fn get_random_uuid(&self, Parameters(args): Parameters<GetRandomUuidArgs>) -> Result<String, ErrorData> {
        let count = args.count.unwrap_or(1);

        // Validate count
        if count == 0 || count > 100 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 100", None));
        }

        // Call gateway API
        let url = format!("{}/api/uuid?count={}", self.gateway_url, count);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))
    }

    /// Get entropy buffer status and health from gateway
    #[tool(description = "Get entropy buffer status and health")]
    async fn get_status(&self) -> Result<String, ErrorData> {
        // Call gateway API
        let url = format!("{}/api/status", self.gateway_url);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))
    }

    /// Test random data quality using Monte Carlo π estimation (via gateway)
    #[tool(description = "Test the quality of quantum random data using Monte Carlo π estimation. Returns statistical metrics about randomness quality.")]
    async fn get_data_quality(&self) -> Result<String, ErrorData> {
        // Use default iterations (500k) for quality testing
        const ITERATIONS: u64 = 500_000;
        
        // Call gateway's Monte Carlo endpoint
        let url = format!("{}/api/test/monte-carlo?iterations={}", self.gateway_url, ITERATIONS);
        
        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.gateway_api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::INSUFFICIENT_STORAGE {
                return Ok(serde_json::json!({
                    "status": "unavailable",
                    "message": "Insufficient entropy in gateway buffer. Test will be available soon as the buffer fills."
                }).to_string());
            }
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))
    }
}

#[tool_handler]
impl ServerHandler for QrngMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            ..Default::default()
        }
    }
}










