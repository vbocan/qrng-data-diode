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
    fetcher: EntropyFetcher,
    buffer: EntropyBuffer,
    signer: PacketSigner,
    http_client: reqwest::Client,
    metrics: Metrics,
    sequence: Arc<std::sync::atomic::AtomicU64>,
}

impl Collector {
    fn new(config: CollectorConfig) -> Result<Self> {
        // Parse HMAC secret key
        let hmac_key = hex::decode(&config.hmac_secret_key)
            .context("Failed to decode HMAC secret key")?;
        let signer = PacketSigner::new(hmac_key);

        // Create fetcher
        let fetcher_config = FetcherConfig::new(
            config.appliance_url.parse()?,
            config.fetch_chunk_size,
        );
        let fetcher = EntropyFetcher::new(fetcher_config)?;

        // Create buffer
        let buffer = EntropyBuffer::new(config.buffer_size);

        // Create HTTP client for pushing
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            fetcher,
            buffer,
            signer,
            http_client,
            metrics: Metrics::new(),
            sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Main run loop
    async fn run(self: Arc<Self>) -> Result<()> {
        info!("Starting Entropy Collector");
        info!("Appliance URL: {}", self.config.appliance_url);
        info!("Push URL: {}", self.config.push_url);
        info!("Buffer size: {} bytes", self.config.buffer_size);

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

    /// Fetch loop: continuously fetch data from appliance
    async fn fetch_loop(self: Arc<Self>) {
        let mut ticker = interval(self.config.fetch_interval());

        loop {
            ticker.tick().await;

            match self.fetcher.fetch().await {
                Ok(data) => {
                    self.metrics.record_fetch(data.len());
                    
                    if let Err(e) = self.buffer.push(data) {
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
                Err(e) => {
                    self.metrics.record_fetch_failure();
                    error!("Fetch failed: {}", e);
                }
            }
        }
    }

    /// Push loop: periodically push buffered data to gateway
    async fn push_loop(self: Arc<Self>) {
        let mut ticker = interval(self.config.push_interval());

        loop {
            ticker.tick().await;

            if self.buffer.is_empty() {
                info!("Buffer empty, skipping push");
                continue;
            }

            if let Err(e) = self.push_buffer().await {
                error!("Push failed: {}", e);
            }
        }
    }

    /// Push accumulated data to gateway
    async fn push_buffer(&self) -> Result<()> {
        // Extract data from buffer
        let batch_size = self.buffer.len().min(self.config.fetch_chunk_size * 10);
        let data = match self.buffer.pop(batch_size) {
            Some(d) => d,
            None => {
                warn!("No data available to push");
                return Ok(());
            }
        };

        // Create packet
        let sequence = self.sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
            Ok(())
        } else {
            self.metrics.record_push_failure();
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
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
            use signal_hook::consts::signal::*;
            use signal_hook_tokio::Signals;
            use futures::stream::StreamExt;

            let mut signals = Signals::new(&[SIGINT, SIGTERM])
                .expect("Failed to register signal handlers");

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
    let log_level = args.log_level.parse::<tracing::Level>()
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
    let config = CollectorConfig::from_env()
        .context("Failed to load configuration from environment")?;

    // Create and run collector
    let collector = Arc::new(Collector::new(config)?);
    collector.run().await
}
