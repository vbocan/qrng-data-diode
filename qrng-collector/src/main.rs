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

//! Entropy Collector - Internal Component for QRNG Data Diode
//!
//! The Entropy Collector runs within the restricted network containing the QRNG appliance.
//! It fetches random data, accumulates it in a buffer, and periodically pushes signed packets
//! to the Entropy Gateway via unidirectional data flow (emulating a data diode).
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐     fetch     ┌──────────────┐     push       ┌──────────────┐
//! │   Quantis    │ ─────────────>│   Collector  │ ──────────────>│   Gateway    │
//! │  Appliance   │    (HTTPS)    │   (Buffer)   │   (HTTPS)      │  (External)  │
//! └──────────────┘               └──────────────┘                └──────────────┘
//!       │                               │
//!       │                               │
//!    Internal                      Unidirectional
//!    Network                          Flow
//! ```
//!
//! # Features
//!
//! - Resilient fetching with exponential backoff
//! - High-performance in-memory buffering
//! - Cryptographic packet signing (HMAC-SHA256)
//! - Graceful shutdown with buffer flushing
//! - Comprehensive metrics and logging

use anyhow::{Context, Result};
use clap::Parser;
use qrng_core::{
    buffer::EntropyBuffer,
    config::CollectorConfig,
    crypto::PacketSigner,
    fetcher::{EntropyFetcher, FetcherConfig},
    metrics::Metrics,
    mixer::EntropyMixer,
    protocol::EntropyPacket,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "qrng-collector")]
#[command(about = "QRNG Collector - Fetches and pushes quantum random data", long_about = None)]
struct Args {
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

/// Main collector application state
struct Collector {
    config: CollectorConfig,
    fetchers: Vec<EntropyFetcher>,
    mixer: Option<EntropyMixer>,
    buffer: EntropyBuffer,
    signer: PacketSigner,
    http_client: reqwest::Client,
    metrics: Metrics,
    sequence: Arc<std::sync::atomic::AtomicU64>,
    backoff_until: Arc<tokio::sync::RwLock<Option<std::time::Instant>>>,
    fetch_backoff_duration: Arc<tokio::sync::RwLock<Duration>>,
}

impl Collector {
    fn new(config: CollectorConfig) -> Result<Self> {
        // Parse HMAC secret key
        let hmac_key =
            hex::decode(&config.hmac_secret_key).context("Failed to decode HMAC secret key")?;
        let signer = PacketSigner::new(hmac_key);

        // Create fetchers for all sources
        let urls = config.get_appliance_urls();
        let mut fetchers = Vec::new();

        for url in &urls {
            let fetcher_config = FetcherConfig::new(url.parse()?, config.fetch_chunk_size);
            let fetcher = EntropyFetcher::new(fetcher_config)?;
            fetchers.push(fetcher);
        }

        // Create mixer if multiple sources
        let mixer = if config.has_multiple_sources() {
            Some(EntropyMixer::new(config.mixing_strategy))
        } else {
            None
        };

        // Create buffer
        let buffer = EntropyBuffer::new(config.buffer_size);

        // Create HTTP client for pushing
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            fetchers,
            mixer,
            buffer,
            signer,
            http_client,
            metrics: Metrics::new(),
            sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            backoff_until: Arc::new(tokio::sync::RwLock::new(None)),
            fetch_backoff_duration: Arc::new(tokio::sync::RwLock::new(Duration::from_secs(1))),
        })
    }

    /// Main run loop
    async fn run(self: Arc<Self>) -> Result<()> {
        info!("QRNG Collector v{}", env!("CARGO_PKG_VERSION"));
        info!("The collector runs in the same network as the Quantis Appliance and pushes data to the gateway via unidirectional flow.");
        info!("Developed by Valer BOCAN, PhD, CSSLP - www.bocan.ro");

        let urls = self.config.get_appliance_urls();
        info!("Configured {} source(s)", urls.len());
        for (i, url) in urls.iter().enumerate() {
            info!("  Source {}: {}", i + 1, url);
        }
        
        if urls.len() > 1 {
            info!("Mixing strategy: {:?}", self.config.mixing_strategy);
        }

        info!("Random data is pushed to URL: {}", self.config.push_url);
        info!("Buffer size: {} bytes", self.config.buffer_size);
        info!("Fetch interval: {:?} sec.", self.config.fetch_interval());
        info!("Push interval: {:?} sec.", self.config.push_interval());

        // Spawn fetch task
        let fetch_handle = {
            let collector = Arc::clone(&self);
            tokio::spawn(async move { collector.fetch_loop().await })
        };

        // Spawn push task
        let push_handle = {
            let collector = Arc::clone(&self);
            tokio::spawn(async move { collector.push_loop().await })
        };

        // Wait for shutdown signal
        Self::wait_for_shutdown().await;

        info!("Shutdown signal received, flushing buffer...");

        // Attempt final push
        if let Err(e) = self.push_buffer().await {
            error!("Failed to flush buffer on shutdown: {}", e);
        }

        // Clean up
        fetch_handle.abort();
        push_handle.abort();

        info!("Collector shut down gracefully");
        Ok(())
    }

    /// Fetch loop: continuously fetch data from appliances
    async fn fetch_loop(self: Arc<Self>) {
        let mut ticker = interval(self.config.fetch_interval());
        const HIGH_WATER_MARK: f64 = 98.0;

        loop {
            ticker.tick().await;

            // Check if we're in backoff period
            let backoff = self.backoff_until.read().await;
            if let Some(until) = *backoff {
                if std::time::Instant::now() < until {
                    drop(backoff);
                    continue;
                }
            }
            drop(backoff);

            // If buffer is critically full, trigger immediate push
            let fill_percent = self.buffer.fill_percent();
            if fill_percent >= HIGH_WATER_MARK {
                info!("Buffer at {:.1}%, triggering immediate push", fill_percent);
                let self_clone = Arc::clone(&self);
                tokio::spawn(async move {
                    if let Err(e) = self_clone.push_buffer().await {
                        error!("Emergency push failed: {}", e);
                    }
                });
            }

            // If buffer is completely full, skip fetching to avoid wasted work
            if fill_percent >= 100.0 {
                warn!("Buffer full, skipping fetch until space available");
                continue;
            }

            // Fetch from all sources in parallel
            let fetch_results = {
                let mut handles = Vec::new();
                for (i, fetcher) in self.fetchers.iter().enumerate() {
                    let fetcher = fetcher.clone();
                    let handle = tokio::spawn(async move {
                        (i, fetcher.fetch().await)
                    });
                    handles.push(handle);
                }

                let mut results = Vec::new();
                for handle in handles {
                    if let Ok(result) = handle.await {
                        results.push(result);
                    }
                }
                results
            };

            // Process results
            let mut chunks = Vec::new();
            let mut failed_sources = Vec::new();

            for (i, result) in fetch_results {
                match result {
                    Ok(data) => {
                        chunks.push(data);
                    }
                    Err(e) => {
                        failed_sources.push((i, e));
                    }
                }
            }

            // Log failures
            for (i, e) in &failed_sources {
                warn!("Source {} fetch failed: {}", i + 1, e);
            }

            // Mix if we have multiple chunks
            let final_data = if chunks.is_empty() {
                self.metrics.record_fetch_failure();
                
                // Apply exponential backoff when all sources fail
                let current_backoff = *self.fetch_backoff_duration.read().await;
                let next_backoff = (current_backoff * 2).min(Duration::from_secs(300)); // Cap at 5 minutes
                *self.fetch_backoff_duration.write().await = next_backoff;
                
                let backoff_until = std::time::Instant::now() + current_backoff;
                *self.backoff_until.write().await = Some(backoff_until);
                
                error!(
                    "All sources failed to fetch, backing off for {} seconds",
                    current_backoff.as_secs()
                );
                continue;
            } else if chunks.len() == 1 {
                // Reset backoff on successful fetch
                *self.fetch_backoff_duration.write().await = Duration::from_secs(1);
                *self.backoff_until.write().await = None;
                
                chunks.into_iter().next().unwrap()
            } else if let Some(mixer) = &self.mixer {
                // Reset backoff on successful fetch
                *self.fetch_backoff_duration.write().await = Duration::from_secs(1);
                *self.backoff_until.write().await = None;
                
                match mixer.mix(&chunks) {
                    Ok(mixed) => {
                        info!("Mixed {} sources into {} bytes", chunks.len(), mixed.len());
                        mixed
                    }
                    Err(e) => {
                        error!("Failed to mix entropy: {}", e);
                        self.metrics.record_fetch_failure();
                        continue;
                    }
                }
            } else {
                // Shouldn't happen, but fallback to first chunk
                chunks.into_iter().next().unwrap()
            };

            // Push to buffer
            let data_len = final_data.len();
            self.metrics.record_fetch(data_len);

            if let Err(e) = self.buffer.push(final_data) {
                error!("Failed to push to buffer: {}", e);
            } else {
                info!(
                    "Fetched data, buffer: {}/{} bytes ({:.1}%)",
                    self.buffer.len(),
                    self.buffer.capacity(),
                    self.buffer.fill_percent()
                );
            }
        }
    }

    /// Push loop: periodically push buffered data to gateway
    async fn push_loop(self: Arc<Self>) {
        let mut ticker = interval(self.config.push_interval());
        const MIN_PUSH_THRESHOLD: f64 = 1.0;

        loop {
            ticker.tick().await;

            // Check if we're in backoff period
            let backoff = self.backoff_until.read().await;
            if let Some(until) = *backoff {
                if std::time::Instant::now() < until {
                    drop(backoff);
                    continue;
                }
            }
            drop(backoff);

            let fill_percent = self.buffer.fill_percent();

            if self.buffer.is_empty() {
                continue;
            }

            if fill_percent >= MIN_PUSH_THRESHOLD {
                if let Err(e) = self.push_buffer().await {
                    error!("Push failed: {}", e);
                }
            }
        }
    }

    /// Push accumulated data to gateway
    async fn push_buffer(&self) -> Result<()> {
        // Calculate batch size dynamically to allow partial packet accumulation
        // This ensures the gateway buffer can reach 100% regardless of packet/buffer size ratios
        // Use available data up to 1MB, allowing any size (not constrained to fixed packets)
        let available = self.buffer.len();
        if available == 0 {
            warn!("No data available to push");
            return Ok(());
        }
        
        let batch_size = available.min(1024 * 1024);
        let data = match self.buffer.pop(batch_size) {
            Some(d) => d,
            None => {
                warn!("Failed to pop data from buffer");
                return Ok(());
            }
        };

        // Create packet
        let sequence = self
            .sequence
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut packet = EntropyPacket::new(sequence, data.to_vec());

        // Add checksum
        packet.checksum = Some(packet.calculate_checksum());

        // Sign packet
        self.signer.sign_packet(&mut packet)?;

        // Serialize
        let serialized = packet.to_msgpack()?;

        info!(
            "Pushing packet #{} ({} bytes, checksum: {:08x})",
            packet.sequence,
            packet.payload_size(),
            packet.checksum.unwrap()
        );

        // Send to gateway
        let response = self
            .http_client
            .post(&self.config.push_url)
            .header("Content-Type", "application/msgpack")
            .body(serialized.clone())
            .send()
            .await?;

        if response.status().is_success() {
            self.metrics.record_push(packet.payload_size());
            info!("Push successful ({})", response.status());
            
            // Clear backoff on success
            *self.backoff_until.write().await = None;
            Ok(())
        } else {
            self.metrics.record_push_failure();
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            
            // Apply exponential backoff for 507 Insufficient Storage
            if status == 507 {
                let backoff_read = self.backoff_until.read().await;
                let current_backoff = *backoff_read;
                drop(backoff_read);
                
                let backoff_duration = if current_backoff.is_none() {
                    Duration::from_secs(1)
                } else {
                    Duration::from_secs(5)
                };
                
                let backoff_until = std::time::Instant::now() + backoff_duration;
                *self.backoff_until.write().await = Some(backoff_until);
                
                warn!(
                    "Gateway buffer full (507), backing off for {} seconds",
                    backoff_duration.as_secs()
                );
            }
            
            error!("Push failed with status {}: {}", status, body);

            // Put data back in buffer
            self.buffer.push(packet.data)?;

            Err(anyhow::anyhow!("Push failed: {}", status))
        }
    }

    /// Wait for shutdown signal (SIGINT or SIGTERM)
    async fn wait_for_shutdown() {
        #[cfg(unix)]
        {
            use futures::stream::StreamExt;
            use signal_hook::consts::signal::*;
            use signal_hook_tokio::Signals;

            let mut signals =
                Signals::new(&[SIGINT, SIGTERM]).expect("Failed to register signal handlers");

            if let Some(signal) = signals.next().await {
                info!("Received signal: {:?}", signal);
            }
        }

        #[cfg(windows)]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C");
            info!("Received Ctrl+C signal");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize tracing
    let log_level = args
        .log_level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .json()
        .init();

    info!("QRNG Collector v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration from environment variables
    info!("Loading configuration from environment variables");
    let config =
        CollectorConfig::from_env().context("Failed to load configuration from environment")?;

    // Create and run collector
    let collector = Arc::new(Collector::new(config)?);
    collector.run().await
}
