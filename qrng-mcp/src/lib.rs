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
        }
    }

    /// Fetch random bytes from quantum entropy source
    #[tool(description = "Fetch random bytes from quantum entropy source")]
    async fn get_random_bytes(&self, Parameters(args): Parameters<GetRandomBytesArgs>) -> Result<String, ErrorData> {
        let mut buffer = self.buffer.lock().await;

        // Validate count
        if args.count == 0 || args.count > 65536 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 65536", None));
        }

        // Get random bytes
        let mut bytes = vec![0u8; args.count];
        
        if let Some(data) = buffer.pop(bytes.len()) {
            bytes.copy_from_slice(&data);
        } else {
            return Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None));
        }

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
        let mut buffer = self.buffer.lock().await;

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

        for _ in 0..args.count {
            let mut bytes = [0u8; 8];
            
        if let Some(data) = buffer.pop(bytes.len()) {
            bytes.copy_from_slice(&data);
        } else {
            return Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None));
        }
            let value = u64::from_le_bytes(bytes);
            let result = min + (value % range) as i64;
            integers.push(result);
        }

        serde_json::to_string(&integers).map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None))
    }

    /// Generate random floats in range [0, 1)
    #[tool(description = "Generate random floats in range [0, 1)")]
    async fn get_random_floats(&self, Parameters(args): Parameters<GetRandomFloatsArgs>) -> Result<String, ErrorData> {
        let mut buffer = self.buffer.lock().await;

        // Validate count
        if args.count == 0 || args.count > 1000 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 1000", None));
        }

        let mut floats = Vec::new();

        for _ in 0..args.count {
            let mut bytes = [0u8; 8];
            
        if let Some(data) = buffer.pop(bytes.len()) {
            bytes.copy_from_slice(&data);
        } else {
            return Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None));
        }
            let value = u64::from_le_bytes(bytes);
            let float_val = (value as f64) / (u64::MAX as f64);
            floats.push(float_val);
        }

        serde_json::to_string(&floats).map_err(|e| ErrorData::new(ErrorCode::INTERNAL_ERROR, format!("Failed to serialize result: {}", e), None))
    }

    /// Generate random UUID v4
    #[tool(description = "Generate random UUID v4")]
    async fn get_random_uuid(&self, Parameters(args): Parameters<GetRandomUuidArgs>) -> Result<String, ErrorData> {
        let mut buffer = self.buffer.lock().await;

        let count = args.count.unwrap_or(1);

        // Validate count
        if count == 0 || count > 100 {
            return Err(ErrorData::new(ErrorCode::INVALID_PARAMS, "Count must be between 1 and 100", None));
        }

        let mut uuids = Vec::new();

        for _ in 0..count {
            let mut bytes = [0u8; 16];
            
        if let Some(data) = buffer.pop(bytes.len()) {
            bytes.copy_from_slice(&data);
        } else {
            return Err(ErrorData::new(ErrorCode::INTERNAL_ERROR, "Not enough entropy available", None));
        }
            
            // Set version (4) and variant (RFC 4122)
            bytes[6] = (bytes[6] & 0x0f) | 0x40;
            bytes[8] = (bytes[8] & 0x3f) | 0x80;
            
            let uuid = uuid::Uuid::from_bytes(bytes);
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









