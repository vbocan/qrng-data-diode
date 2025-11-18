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
#[derive(Clone)]
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
            let reason = status.canonical_reason().unwrap_or("Unknown");
            warn!("HTTP error {}: {}", status, reason);
            return Err(Error::Validation(format!("HTTP {} {}", status, reason)));
        }

        // Read response body
        let data = response.bytes().await.map_err(Error::Network)?;
        
        // Try to parse as JSON array of integers first (Quantis API v2.0 format)
        // If that fails, treat as raw binary data
        let data_vec = match serde_json::from_slice::<Vec<u8>>(&data) {
            Ok(json_array) => {
                debug!("Parsed JSON array of {} bytes", json_array.len());
                json_array
            }
            Err(_) => {
                // Not JSON, use as raw binary
                debug!("Using raw binary data");
                data.to_vec()
            }
        };

        // Validate response
        self.validate_response(&data_vec)?;

        debug!("Successfully fetched {} bytes", data_vec.len());
        Ok(data_vec)
    }

    /// Build request URL with proper query parameters
    fn build_request_url(&self) -> Result<Url> {
        let mut url = self.config.base_url.clone();
        
        // Add query parameter for byte count
        // Quantis Appliance API v2.0 uses "size" parameter
        url.query_pairs_mut()
            .append_pair("size", &self.config.chunk_size.to_string());
        
        Ok(url)
    }

    /// Validate fetched data
    fn validate_response(&self, data: &[u8]) -> Result<()> {
        // Check if we got expected amount of data
        if data.len() != self.config.chunk_size {
            return Err(Error::Validation(format!(
                "Received {} bytes, expected {}",
                data.len(),
                self.config.chunk_size
            )));
        }

        // Basic sanity check: ensure we got some data
        if data.is_empty() {
            return Err(Error::Validation("Received empty response".to_string()));
        }

        // Check for HTML content (error pages)
        if data.len() > 15 {
            let prefix = &data[0..15];
            if prefix.starts_with(b"<!doctype html>") ||
               prefix.starts_with(b"<!DOCTYPE html>") ||
               prefix.starts_with(b"<html>") {
                return Err(Error::Validation(
                    "Received HTML content instead of binary random data".to_string()
                ));
            }
        }

        // Check for obvious non-random patterns (all zeros, all same byte)
        if data.iter().all(|&b| b == data[0]) {
            return Err(Error::Validation(format!(
                "All bytes have the same value (0x{:02X}), not random data",
                data[0]
            )));
        }

        // Check for low entropy (too many repeated bytes)
        let mut byte_counts = [0u32; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }
        
        // If any single byte appears more than 90% of the time, it's not random
        let max_count = byte_counts.iter().max().unwrap();
        let threshold = (data.len() as f64 * 0.9) as u32;
        if *max_count > threshold {
            return Err(Error::Validation(format!(
                "Low entropy detected: one byte value appears {} times out of {} (>90%)",
                max_count, data.len()
            )));
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
        assert!(url.to_string().contains("size=1024"));
    }

    #[test]
    fn test_validation() {
        let config = FetcherConfig::new(
            Url::parse("https://example.com/random").unwrap(),
            100,
        );
        let fetcher = EntropyFetcher::new(config).unwrap();
        
        // Valid random-looking data with correct size
        let mut valid_data = vec![0u8; 100];
        for i in 0..100 {
            valid_data[i] = (i % 256) as u8; // Varied data
        }
        assert!(fetcher.validate_response(&valid_data).is_ok());
        
        // Empty data
        assert!(fetcher.validate_response(&[]).is_err());
        
        // Wrong size
        let wrong_size = vec![1, 2, 3, 4, 5];
        assert!(fetcher.validate_response(&wrong_size).is_err());
        
        // HTML content
        let html = b"<!doctype html><html><body>Error</body></html>".to_vec();
        assert!(fetcher.validate_response(&html).is_err());
        
        // All same byte (non-random)
        let all_zeros = vec![0u8; 100];
        assert!(fetcher.validate_response(&all_zeros).is_err());
        
        // Low entropy (90% same byte)
        let mut low_entropy = vec![42u8; 95];
        low_entropy.extend_from_slice(&[1, 2, 3, 4, 5]);
        assert!(fetcher.validate_response(&low_entropy).is_err());
    }
}
