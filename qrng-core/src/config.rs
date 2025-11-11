//! Configuration management for QRNG components

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Entropy mixing strategy
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MixingStrategy {
    /// No mixing - use single source only
    None,
    /// XOR all sources together
    Xor,
    /// Use HKDF (HMAC-based Key Derivation Function) for mixing
    Hkdf,
}

impl Default for MixingStrategy {
    fn default() -> Self {
        Self::None
    }
}

/// Entropy Collector configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectorConfig {
    /// URLs of QRNG appliances (supports single or multiple sources)
    #[serde(default)]
    pub appliance_urls: Vec<String>,

    /// Entropy mixing strategy for multiple sources
    #[serde(default)]
    pub mixing_strategy: MixingStrategy,

    /// Bytes to fetch per request
    #[serde(default = "default_chunk_size")]
    pub fetch_chunk_size: usize,

    /// Fetch interval in milliseconds
    #[serde(default = "default_fetch_interval_ms")]
    pub fetch_interval_ms: u64,

    /// Internal buffer size in bytes
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    /// URL of Entropy Gateway push endpoint
    pub push_url: String,

    /// Push interval in milliseconds
    #[serde(default = "default_push_interval_ms")]
    pub push_interval_ms: u64,

    /// HMAC secret key (hex-encoded)
    pub hmac_secret_key: String,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial backoff in milliseconds
    #[serde(default = "default_initial_backoff_ms")]
    pub initial_backoff_ms: u64,
}

impl CollectorConfig {
    /// Get all appliance URLs
    pub fn get_appliance_urls(&self) -> Vec<String> {
        self.appliance_urls.clone()
    }

    /// Returns true if multiple sources are configured
    pub fn has_multiple_sources(&self) -> bool {
        self.appliance_urls.len() > 1
    }
}

impl CollectorConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config: Self = envy::prefixed("QRNG_")
            .from_env()
            .map_err(|e| Error::Config(format!("Failed to parse environment variables: {}", e)))?;
        
        // Handle comma-separated APPLIANCE_URLS if provided as single string
        if config.appliance_urls.is_empty() {
            if let Ok(urls_str) = std::env::var("QRNG_APPLIANCE_URLS") {
                config.appliance_urls = urls_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate appliance URLs
        if self.appliance_urls.is_empty() {
            return Err(Error::Config(
                "Must provide at least one appliance URL via QRNG_APPLIANCE_URLS".to_string()
            ));
        }

        for url in &self.appliance_urls {
            Url::parse(url)
                .map_err(|e| Error::Config(format!("Invalid appliance URL '{}': {}", url, e)))?;
        }

        // Validate push URL
        Url::parse(&self.push_url)
            .map_err(|e| Error::Config(format!("Invalid push_url: {}", e)))?;

        // Validate mixing strategy
        if self.has_multiple_sources() && self.mixing_strategy == MixingStrategy::None {
            return Err(Error::Config(
                "Multiple sources configured but mixing_strategy is 'none'. Set to 'xor' or 'hkdf'".to_string()
            ));
        }

        // Validate sizes
        if self.fetch_chunk_size == 0 || self.fetch_chunk_size > crate::MAX_REQUEST_SIZE {
            return Err(Error::Config(format!(
                "fetch_chunk_size must be between 1 and {}",
                crate::MAX_REQUEST_SIZE
            )));
        }

        if self.buffer_size < self.fetch_chunk_size {
            return Err(Error::Config(
                "buffer_size must be >= fetch_chunk_size".to_string()
            ));
        }

        // Validate secret key
        if self.hmac_secret_key.is_empty() {
            return Err(Error::Config("hmac_secret_key cannot be empty".to_string()));
        }

        Ok(())
    }

    pub fn fetch_interval(&self) -> Duration {
        Duration::from_millis(self.fetch_interval_ms)
    }

    pub fn push_interval(&self) -> Duration {
        Duration::from_millis(self.push_interval_ms)
    }
}

/// Entropy Gateway configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    /// Bind address for HTTP server
    #[serde(default = "default_listen_address")]
    pub listen_address: String,
    
    /// Buffer size in bytes
    #[serde(default = "default_gateway_buffer_size")]
    pub buffer_size: usize,
    
    /// Buffer TTL in seconds (0 = no TTL)
    #[serde(default)]
    pub buffer_ttl_secs: u64,
    
    /// Valid API keys for authentication
    pub api_keys: Vec<String>,
    
    /// Rate limit: requests per second per key
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_second: u32,
    
    /// HMAC secret key for push mode (hex-encoded)
    #[serde(default)]
    pub hmac_secret_key: Option<String>,
    
    /// Direct mode configuration (only used if deployment_mode = DirectAccess)
    pub direct_mode: Option<DirectModeConfig>,
    
    /// Enable MCP server
    #[serde(default)]
    pub mcp_enabled: bool,
    
    /// Enable Prometheus metrics
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,
}

/// Direct access mode configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DirectModeConfig {
    /// QRNG appliance URL
    pub appliance_url: String,
    
    /// Fetch chunk size
    #[serde(default = "default_chunk_size")]
    pub fetch_chunk_size: usize,
    
    /// Fetch interval in milliseconds
    #[serde(default = "default_fetch_interval_ms")]
    pub fetch_interval_ms: u64,
}

impl GatewayConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Parse basic config from env
        let mut config: Self = envy::prefixed("QRNG_")
            .from_env()
            .map_err(|e| Error::Config(format!("Failed to parse environment variables: {}", e)))?;

        // Parse API keys from comma-separated string
        if let Ok(keys) = std::env::var("QRNG_API_KEYS") {
            config.api_keys = keys.split(',').map(|s| s.trim().to_string()).collect();
        }
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate buffer size
        if self.buffer_size == 0 {
            return Err(Error::Config("buffer_size must be > 0".to_string()));
        }

        // Validate API keys
        if self.api_keys.is_empty() {
            return Err(Error::Config("At least one API key required".to_string()));
        }
        Ok(())
    }

    pub fn buffer_ttl(&self) -> Option<chrono::Duration> {
        if self.buffer_ttl_secs > 0 {
            Some(chrono::Duration::seconds(self.buffer_ttl_secs as i64))
        } else {
            None
        }
    }
}

// Default value functions
fn default_chunk_size() -> usize {
    crate::DEFAULT_CHUNK_SIZE
}

fn default_buffer_size() -> usize {
    1024 * 1024 // 1 MB for collector
}

fn default_gateway_buffer_size() -> usize {
    crate::DEFAULT_BUFFER_SIZE
}

fn default_fetch_interval_ms() -> u64 {
    100  // 100ms = 10 fetches per second
}

fn default_push_interval_ms() -> u64 {
    500  // 500ms = 2 pushes per second
}

fn default_max_retries() -> u32 {
    5
}

fn default_initial_backoff_ms() -> u64 {
    100
}

fn default_listen_address() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_rate_limit() -> u32 {
    100
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_config_validation() {
        let config = CollectorConfig {
            appliance_urls: vec!["https://example.com/random".to_string()],
            mixing_strategy: MixingStrategy::None,
            fetch_chunk_size: 1024,
            fetch_interval_ms: 100,
            buffer_size: 10240,
            push_url: "https://gateway.com/push".to_string(),
            push_interval_ms: 500,
            hmac_secret_key: "secret123".to_string(),
            max_retries: 5,
            initial_backoff_ms: 100,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_multi_source_config() {
        let config = CollectorConfig {
            appliance_urls: vec![
                "https://source1.com/random".to_string(),
                "https://source2.com/random".to_string(),
            ],
            mixing_strategy: MixingStrategy::Xor,
            fetch_chunk_size: 1024,
            fetch_interval_ms: 100,
            buffer_size: 10240,
            push_url: "https://gateway.com/push".to_string(),
            push_interval_ms: 500,
            hmac_secret_key: "secret123".to_string(),
            max_retries: 5,
            initial_backoff_ms: 100,
        };
        assert!(config.validate().is_ok());
        assert!(config.has_multiple_sources());
        assert_eq!(config.get_appliance_urls().len(), 2);
    }

    #[test]
    fn test_gateway_config_validation() {
        let config = GatewayConfig {            
            listen_address: "0.0.0.0:8080".to_string(),
            buffer_size: 10240,
            buffer_ttl_secs: 3600,
            api_keys: vec!["key1".to_string()],
            rate_limit_per_second: 100,
            hmac_secret_key: Some("secret".to_string()),
            direct_mode: None,
            mcp_enabled: false,
            metrics_enabled: true,
        };
        assert!(config.validate().is_ok());
    }
}
