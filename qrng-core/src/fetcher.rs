//! HTTPS client for fetching entropy from Quantis appliance
//!
//! Implements resilient fetching with connection pooling, retry logic, and rate limiting.

use crate::{Error, Result, retry::RetryPolicy};
use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use tracing::{debug, instrument, warn};
use url::Url;

/// Configuration for the entropy fetcher
#[derive(Debug, Clone)]
pub struct FetcherConfig {
    /// Base URL of the QRNG appliance
    pub base_url: Url,
    /// Number of bytes to fetch per request
    pub chunk_size: usize,
    /// Request timeout
    pub timeout: Duration,
    /// Retry policy
    pub retry_policy: RetryPolicy,
}

impl FetcherConfig {
    pub fn new(base_url: Url, chunk_size: usize) -> Self {
        Self {
            base_url,
            chunk_size,
            timeout: Duration::from_secs(30),
            retry_policy: RetryPolicy::default(),
        }
    }
}

/// HTTP client for fetching entropy from QRNG appliance
pub struct EntropyFetcher {
    client: Client,
    config: FetcherConfig,
}

impl EntropyFetcher {
    /// Create a new fetcher with configuration
    pub fn new(config: FetcherConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(config.timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .use_rustls_tls()
            .https_only(true)
            .build()
            .map_err(Error::Network)?;

        Ok(Self { client, config })
    }

    /// Fetch entropy bytes from the appliance
    ///
    /// This method automatically retries transient failures according to the retry policy.
    #[instrument(skip(self), fields(chunk_size = self.config.chunk_size))]
    pub async fn fetch(&self) -> Result<Vec<u8>> {
        self.config.retry_policy.execute(|| self.fetch_once()).await
    }

    /// Fetch entropy once without retry
    async fn fetch_once(&self) -> Result<Vec<u8>> {
        // Construct request URL with query parameter for byte count
        let url = self.build_request_url()?;
        
        debug!("Fetching {} bytes from {}", self.config.chunk_size, url);

        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to fetch from {}: {}", url, e);
                Error::Network(e)
            })?;

        // Check HTTP status
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("HTTP error {}: {}", status, body);
            return Err(Error::Validation(format!("HTTP {}: {}", status, body)));
        }

        // Read response body
        let data = response.bytes().await.map_err(Error::Network)?;
        let data_vec = data.to_vec();

        // Validate response
        self.validate_response(&data_vec)?;

        debug!("Successfully fetched {} bytes", data_vec.len());
        Ok(data_vec)
    }

    /// Build request URL with proper query parameters
    fn build_request_url(&self) -> Result<Url> {
        let mut url = self.config.base_url.clone();
        
        // Add query parameter for byte count (adjust based on actual API)
        url.query_pairs_mut()
            .append_pair("bytes", &self.config.chunk_size.to_string());
        
        Ok(url)
    }

    /// Validate fetched data
    fn validate_response(&self, data: &[u8]) -> Result<()> {
        // Check if we got expected amount of data
        if data.len() != self.config.chunk_size {
            warn!(
                "Received {} bytes, expected {}",
                data.len(),
                self.config.chunk_size
            );
        }

        // Basic sanity check: ensure we got some data
        if data.is_empty() {
            return Err(Error::Validation("Received empty response".to_string()));
        }

        // Optional: Check for obvious non-random patterns (all zeros, all same byte)
        if data.iter().all(|&b| b == data[0]) {
            warn!("Warning: All bytes have the same value ({})", data[0]);
        }

        Ok(())
    }

    /// Get fetcher configuration
    pub fn config(&self) -> &FetcherConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let config = FetcherConfig::new(
            Url::parse("https://example.com/random").unwrap(),
            1024,
        );
        let fetcher = EntropyFetcher::new(config).unwrap();
        let url = fetcher.build_request_url().unwrap();
        assert!(url.to_string().contains("bytes=1024"));
    }

    #[test]
    fn test_validation() {
        let config = FetcherConfig::new(
            Url::parse("https://example.com/random").unwrap(),
            100,
        );
        let fetcher = EntropyFetcher::new(config).unwrap();
        
        // Valid data
        let data = vec![1, 2, 3, 4, 5];
        assert!(fetcher.validate_response(&data).is_ok());
        
        // Empty data
        assert!(fetcher.validate_response(&[]).is_err());
    }
}
