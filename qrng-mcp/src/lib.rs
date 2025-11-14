//! Model Context Protocol (MCP) server for QRNG
//!
//! Official MCP implementation using the rmcp SDK.
//! Provides AI agent integration for quantum random number generation.
//!
//! # Tools Provided
//!
//! - `get_random_bytes`: Fetch random bytes
//! - `get_random_integers`: Generate random integers in range
//! - `get_random_floats`: Generate random floats
//! - `get_random_uuid`: Generate UUID v4
//! - `get_status`: Query buffer status
//! - `get_data_quality`: Test random data quality using Monte Carlo simulation

use qrng_core::buffer::EntropyBuffer;
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
use std::sync::Arc;
use tokio::sync::Mutex;

/// QRNG MCP Server implementation
#[derive(Clone)]
pub struct QrngMcpServer {
    buffer: Arc<Mutex<EntropyBuffer>>,
    tool_router: ToolRouter<Self>,
    gateway_url: Option<String>,
    gateway_api_key: Option<String>,
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
    /// Create a new QRNG MCP server
    pub fn new(buffer: EntropyBuffer) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(buffer)),
            tool_router: Self::tool_router(),
            gateway_url: None,
            gateway_api_key: None,
        }
    }

    /// Create a new QRNG MCP server with gateway access
    pub fn with_gateway(buffer: EntropyBuffer, gateway_url: String, gateway_api_key: String) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(buffer)),
            tool_router: Self::tool_router(),
            gateway_url: Some(gateway_url),
            gateway_api_key: Some(gateway_api_key),
        }
    }

    /// Fetch random bytes from quantum entropy source
    #[tool(description = "Fetch random bytes from quantum entropy source")]
    async fn get_random_bytes(&self, Parameters(args): Parameters<GetRandomBytesArgs>) -> Result<String, ErrorData> {
        // Validate count
        if args.count == 0 || args.count > 65536 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 65536", None));
        }

        // Get random bytes - buffer.pop() is called on the buffer directly
        let bytes = self.buffer.lock().await.pop(args.count)
            .ok_or_else(|| ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None))?;

        // Encode based on format
        let encoding = args.encoding.as_deref().unwrap_or("hex");
        let encoded = match encoding {
            "hex" => hex::encode(&bytes),
            "base64" => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(&bytes)
            }
            _ => {
                return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Invalid encoding. Must be 'hex' or 'base64'", None));
            }
        };

        Ok(encoded)
    }

    /// Generate random integers in specified range
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

        let range = (max - min + 1) as u64;
        let mut integers = Vec::new();

        // Get all required bytes at once
        let total_bytes = args.count * 8;
        let bytes = self.buffer.lock().await.pop(total_bytes)
            .ok_or_else(|| ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None))?;

        // Convert bytes to integers
        for i in 0..args.count {
            let offset = i * 8;
            let mut chunk = [0u8; 8];
            chunk.copy_from_slice(&bytes[offset..offset + 8]);
            let value = u64::from_le_bytes(chunk);
            let result = min + (value % range) as i64;
            integers.push(result);
        }

        serde_json::to_string(&integers).map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None))
    }

    /// Generate random floats in range [0, 1)
    #[tool(description = "Generate random floats in range [0, 1)")]
    async fn get_random_floats(&self, Parameters(args): Parameters<GetRandomFloatsArgs>) -> Result<String, ErrorData> {
        // Validate count
        if args.count == 0 || args.count > 1000 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 1000", None));
        }

        let mut floats = Vec::new();

        // Get all required bytes at once
        let total_bytes = args.count * 8;
        let bytes = self.buffer.lock().await.pop(total_bytes)
            .ok_or_else(|| ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None))?;

        // Convert bytes to floats
        for i in 0..args.count {
            let offset = i * 8;
            let mut chunk = [0u8; 8];
            chunk.copy_from_slice(&bytes[offset..offset + 8]);
            let value = u64::from_le_bytes(chunk);
            let float_val = (value as f64) / (u64::MAX as f64);
            floats.push(float_val);
        }

        serde_json::to_string(&floats).map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None))
    }

    /// Generate random UUID v4
    #[tool(description = "Generate random UUID v4")]
    async fn get_random_uuid(&self, Parameters(args): Parameters<GetRandomUuidArgs>) -> Result<String, ErrorData> {
        let count = args.count.unwrap_or(1);

        // Validate count
        if count == 0 || count > 100 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 100", None));
        }

        let mut uuids = Vec::new();

        // Get all required bytes at once
        let total_bytes = count * 16;
        let bytes = self.buffer.lock().await.pop(total_bytes)
            .ok_or_else(|| ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None))?;

        // Convert bytes to UUIDs
        for i in 0..count {
            let offset = i * 16;
            let mut chunk = [0u8; 16];
            chunk.copy_from_slice(&bytes[offset..offset + 16]);
            
            // Set version (4) and variant (RFC 4122)
            chunk[6] = (chunk[6] & 0x0f) | 0x40;
            chunk[8] = (chunk[8] & 0x3f) | 0x80;
            
            let uuid = uuid::Uuid::from_bytes(chunk);
            uuids.push(uuid.to_string());
        }

        if uuids.len() == 1 {
            Ok(uuids[0].clone())
        } else {
            serde_json::to_string(&uuids).map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None))
        }
    }

    /// Get entropy buffer status and health
    #[tool(description = "Get entropy buffer status and health")]
    async fn get_status(&self) -> Result<String, ErrorData> {
        let buffer = self.buffer.lock().await;

        let status = serde_json::json!({
            "available_bytes": buffer.len(),
            "capacity_bytes": (*buffer).capacity(),
            "fill_percentage": (buffer.len() as f64 / (*buffer).capacity() as f64 * 100.0).round(),
            "status": if buffer.len() > (*buffer).capacity() / 2 { "healthy" } else { "low" }
        });

        serde_json::to_string(&status).map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize status: {}", e), None))
    }

    /// Test random data quality using Monte Carlo π estimation (via gateway)
    #[tool(description = "Test the quality of quantum random data using Monte Carlo π estimation. Returns statistical metrics about randomness quality.")]
    async fn get_data_quality(&self) -> Result<String, ErrorData> {
        // Check if gateway is configured
        let (gateway_url, api_key) = match (&self.gateway_url, &self.gateway_api_key) {
            (Some(url), Some(key)) => (url, key),
            _ => {
                return Err(ErrorData::new(
                    ErrorCode::INTERNAL_ERROR,
                    "Gateway not configured for quality testing",
                    None
                ));
            }
        };

        // Use 5 million iterations for high-quality statistical testing
        const ITERATIONS: u64 = 5_000_000;
        
        // Check if buffer has enough data (need 16 bytes per iteration)
        let bytes_needed = (ITERATIONS * 16) as usize;
        let buffer = self.buffer.lock().await;
        let available = buffer.len();
        drop(buffer);
        
        if available < bytes_needed {
            let result = serde_json::json!({
                "status": "unavailable",
                "message": format!(
                    "Quality test requires {} MB of entropy. Currently {} MB available. Test will be available soon as the buffer fills.",
                    bytes_needed / 1_048_576,
                    available / 1_048_576
                ),
                "bytes_needed": bytes_needed,
                "bytes_available": available,
                "fill_percentage": (available as f64 / bytes_needed as f64 * 100.0).round()
            });
            return serde_json::to_string(&result)
                .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None));
        }

        // Call gateway's Monte Carlo endpoint
        let client = reqwest::Client::new();
        let url = format!("{}/api/test/monte-carlo?iterations={}", gateway_url, ITERATIONS);
        
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to contact gateway: {}", e), None))?;

        if !response.status().is_success() {
            let status = response.status();
            if status == reqwest::StatusCode::INSUFFICIENT_STORAGE {
                let result = serde_json::json!({
                    "status": "unavailable",
                    "message": "Insufficient entropy in gateway buffer. Test will be available soon as the buffer fills."
                });
                return serde_json::to_string(&result)
                    .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None));
            }
            return Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                format!("Gateway returned error: {}", status),
                None
            ));
        }

        let result_text = response.text().await
            .map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to read response: {}", e), None))?;

        Ok(result_text)
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









