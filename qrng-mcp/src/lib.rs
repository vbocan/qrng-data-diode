//! Model Context Protocol (MCP) server for QRNG
//!
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
use serde::{Deserialize, Serialize};

/// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP server for entropy distribution
pub struct McpServer {
    buffer: EntropyBuffer,
}

impl McpServer {
    pub fn new(buffer: EntropyBuffer) -> Self {
        Self { buffer }
    }

    /// Handle MCP request
    pub fn handle_request(&self, request: &str) -> anyhow::Result<String> {
        let req: McpRequest = serde_json::from_str(request)?;

        let response = match req.method.as_str() {
            "tools/list" => self.list_tools(),
            "tools/call" => self.call_tool(&req.params)?,
            "initialize" => self.initialize(),
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
            },
        };

        Ok(serde_json::to_string(&response)?)
    }

    fn initialize(&self) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: Some(serde_json::json!({
                "protocolVersion": MCP_VERSION,
                "serverInfo": {
                    "name": "qrng-entropy-gateway",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
        }
    }

    fn list_tools(&self) -> McpResponse {
        let tools = vec![
            Tool {
                name: "get_random_bytes".to_string(),
                description: "Fetch random bytes from quantum entropy source".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "count": {
                            "type": "integer",
                            "description": "Number of bytes to fetch (1-65536)",
                            "minimum": 1,
                            "maximum": 65536
                        },
                        "encoding": {
                            "type": "string",
                            "enum": ["hex", "base64"],
                            "default": "hex",
                            "description": "Output encoding format"
                        }
                    },
                    "required": ["count"]
                }),
            },
            Tool {
                name: "get_random_integers".to_string(),
                description: "Generate random integers in specified range".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "count": {
                            "type": "integer",
                            "description": "Number of integers to generate",
                            "minimum": 1,
                            "maximum": 1000
                        },
                        "min": {
                            "type": "integer",
                            "description": "Minimum value (inclusive)",
                            "default": 0
                        },
                        "max": {
                            "type": "integer",
                            "description": "Maximum value (inclusive)",
                            "default": 100
                        }
                    },
                    "required": ["count"]
                }),
            },
            Tool {
                name: "get_random_floats".to_string(),
                description: "Generate random floats in range [0, 1)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "count": {
                            "type": "integer",
                            "description": "Number of floats to generate",
                            "minimum": 1,
                            "maximum": 1000
                        }
                    },
                    "required": ["count"]
                }),
            },
            Tool {
                name: "get_random_uuid".to_string(),
                description: "Generate random UUID v4".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "count": {
                            "type": "integer",
                            "description": "Number of UUIDs to generate",
                            "minimum": 1,
                            "maximum": 100,
                            "default": 1
                        }
                    }
                }),
            },
            Tool {
                name: "get_status".to_string(),
                description: "Get entropy buffer status and health".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        McpResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    fn call_tool(&self, params: &Option<serde_json::Value>) -> anyhow::Result<McpResponse> {
        let params = params.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing params"))?;

        let name = params["name"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

        let arguments = &params["arguments"];

        let result = match name {
            "get_random_bytes" => self.tool_get_random_bytes(arguments)?,
            "get_random_integers" => self.tool_get_random_integers(arguments)?,
            "get_random_floats" => self.tool_get_random_floats(arguments)?,
            "get_random_uuid" => self.tool_get_random_uuid(arguments)?,
            "get_status" => self.tool_get_status()?,
            _ => return Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(1),
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: format!("Unknown tool: {}", name),
                }),
            }),
        };

        Ok(McpResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: Some(result),
            error: None,
        })
    }

    fn tool_get_random_bytes(&self, args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let count = args["count"].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing count"))?;
        let encoding = args["encoding"].as_str().unwrap_or("hex");

        if count == 0 || count > 65536 {
            anyhow::bail!("Invalid count: must be 1-65536");
        }

        let data = self.buffer.pop(count as usize)
            .ok_or_else(|| anyhow::anyhow!("Insufficient entropy"))?;

        let encoded = match encoding {
            "hex" => qrng_core::crypto::encode_hex(&data),
            "base64" => qrng_core::crypto::encode_base64(&data),
            _ => anyhow::bail!("Invalid encoding"),
        };

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": encoded
            }]
        }))
    }

    fn tool_get_random_integers(&self, args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let count = args["count"].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing count"))? as usize;
        let min = args["min"].as_i64().unwrap_or(0);
        let max = args["max"].as_i64().unwrap_or(100);

        if count == 0 || count > 1000 {
            anyhow::bail!("Invalid count: must be 1-1000");
        }

        if min >= max {
            anyhow::bail!("min must be < max");
        }

        let range = (max - min + 1) as u64;
        let bytes_needed = count * 8;

        let data = self.buffer.pop(bytes_needed)
            .ok_or_else(|| anyhow::anyhow!("Insufficient entropy"))?;

        let mut integers = Vec::new();
        for chunk in data.chunks(8) {
            if chunk.len() == 8 {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(chunk);
                let random = u64::from_be_bytes(bytes);
                let value = min + (random % range) as i64;
                integers.push(value);
            }
        }

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("{:?}", integers)
            }]
        }))
    }

    fn tool_get_random_floats(&self, args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let count = args["count"].as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing count"))? as usize;

        if count == 0 || count > 1000 {
            anyhow::bail!("Invalid count: must be 1-1000");
        }

        let bytes_needed = count * 8;
        let data = self.buffer.pop(bytes_needed)
            .ok_or_else(|| anyhow::anyhow!("Insufficient entropy"))?;

        let mut floats = Vec::new();
        for chunk in data.chunks(8) {
            if chunk.len() == 8 {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(chunk);
                let random = u64::from_be_bytes(bytes);
                // Convert to [0, 1) float
                let value = (random as f64) / (u64::MAX as f64);
                floats.push(value);
            }
        }

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("{:?}", floats)
            }]
        }))
    }

    fn tool_get_random_uuid(&self, args: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let count = args["count"].as_u64().unwrap_or(1) as usize;

        if count == 0 || count > 100 {
            anyhow::bail!("Invalid count: must be 1-100");
        }

        let bytes_needed = count * 16;
        let data = self.buffer.pop(bytes_needed)
            .ok_or_else(|| anyhow::anyhow!("Insufficient entropy"))?;

        let mut uuids = Vec::new();
        for chunk in data.chunks(16) {
            if chunk.len() == 16 {
                let uuid = uuid::Uuid::from_slice(chunk)?;
                uuids.push(uuid.to_string());
            }
        }

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": uuids.join("\n")
            }]
        }))
    }

    fn tool_get_status(&self) -> anyhow::Result<serde_json::Value> {
        let fill_percent = self.buffer.fill_percent();
        let status = if fill_percent < 10.0 {
            "unhealthy"
        } else if fill_percent < 30.0 {
            "degraded"
        } else {
            "healthy"
        };

        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Status: {}\nBuffer: {:.1}% ({}/{} bytes)\nFreshness: {} seconds",
                    status,
                    fill_percent,
                    self.buffer.len(),
                    self.buffer.capacity(),
                    self.buffer.freshness_seconds().unwrap_or(0)
                )
            }]
        }))
    }
}

// MCP protocol types
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct McpRequest {
    jsonrpc: String,
    id: Option<u64>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_list_tools() {
        let buffer = EntropyBuffer::new(1024);
        let server = McpServer::new(buffer);

        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let response = server.handle_request(request).unwrap();

        assert!(response.contains("get_random_bytes"));
        assert!(response.contains("get_random_integers"));
    }

    #[test]
    fn test_mcp_get_status() {
        let buffer = EntropyBuffer::new(1024);
        buffer.push(vec![0u8; 500]).unwrap();
        let server = McpServer::new(buffer);

        let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}"#;
        let response = server.handle_request(request).unwrap();

        assert!(response.contains("Status"));
    }
}
