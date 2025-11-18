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

//! Metrics collection and reporting

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;

/// Global metrics collector
#[derive(Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    start_time: Instant,
    
    // Request metrics
    requests_total: AtomicU64,
    requests_failed: AtomicU64,
    bytes_served: AtomicU64,
    
    // Push metrics (for collector)
    pushes_total: AtomicU64,
    pushes_failed: AtomicU64,
    bytes_pushed: AtomicU64,
    
    // Fetch metrics
    fetches_total: AtomicU64,
    fetches_failed: AtomicU64,
    bytes_fetched: AtomicU64,
    
    // Latency tracking (microseconds)
    request_latencies: RwLock<Vec<u64>>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                start_time: Instant::now(),
                requests_total: AtomicU64::new(0),
                requests_failed: AtomicU64::new(0),
                bytes_served: AtomicU64::new(0),
                pushes_total: AtomicU64::new(0),
                pushes_failed: AtomicU64::new(0),
                bytes_pushed: AtomicU64::new(0),
                fetches_total: AtomicU64::new(0),
                fetches_failed: AtomicU64::new(0),
                bytes_fetched: AtomicU64::new(0),
                request_latencies: RwLock::new(Vec::with_capacity(10000)),
            }),
        }
    }

    // Request metrics
    pub fn record_request(&self, bytes: usize, latency_micros: u64) {
        self.inner.requests_total.fetch_add(1, Ordering::Relaxed);
        self.inner.bytes_served.fetch_add(bytes as u64, Ordering::Relaxed);
        
        let mut latencies = self.inner.request_latencies.write();
        latencies.push(latency_micros);
        if latencies.len() > 10000 {
            latencies.drain(0..5000);
        }
    }

    pub fn record_request_failure(&self) {
        self.inner.requests_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn requests_total(&self) -> u64 {
        self.inner.requests_total.load(Ordering::Relaxed)
    }

    pub fn requests_failed(&self) -> u64 {
        self.inner.requests_failed.load(Ordering::Relaxed)
    }

    pub fn bytes_served(&self) -> u64 {
        self.inner.bytes_served.load(Ordering::Relaxed)
    }

    // Push metrics
    pub fn record_push(&self, bytes: usize) {
        self.inner.pushes_total.fetch_add(1, Ordering::Relaxed);
        self.inner.bytes_pushed.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_push_failure(&self) {
        self.inner.pushes_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn pushes_total(&self) -> u64 {
        self.inner.pushes_total.load(Ordering::Relaxed)
    }

    // Fetch metrics
    pub fn record_fetch(&self, bytes: usize) {
        self.inner.fetches_total.fetch_add(1, Ordering::Relaxed);
        self.inner.bytes_fetched.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn record_fetch_failure(&self) {
        self.inner.fetches_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn fetches_total(&self) -> u64 {
        self.inner.fetches_total.load(Ordering::Relaxed)
    }

    // Derived metrics
    pub fn uptime_seconds(&self) -> u64 {
        self.inner.start_time.elapsed().as_secs()
    }

    pub fn requests_per_second(&self) -> f64 {
        let uptime = self.uptime_seconds() as f64;
        if uptime > 0.0 {
            self.requests_total() as f64 / uptime
        } else {
            0.0
        }
    }

    pub fn latency_percentile(&self, percentile: f64) -> Option<u64> {
        let latencies = self.inner.request_latencies.read();
        if latencies.is_empty() {
            return None;
        }

        let mut sorted = latencies.clone();
        sorted.sort_unstable();
        let index = ((sorted.len() as f64 * percentile).ceil() as usize).min(sorted.len() - 1);
        Some(sorted[index])
    }

    pub fn latency_p50(&self) -> Option<u64> {
        self.latency_percentile(0.50)
    }

    pub fn latency_p95(&self) -> Option<u64> {
        self.latency_percentile(0.95)
    }

    pub fn latency_p99(&self) -> Option<u64> {
        self.latency_percentile(0.99)
    }

    /// Generate Prometheus-compatible metrics output
    pub fn prometheus_format(&self) -> String {
        let mut output = String::new();
        
        output.push_str("# HELP qrng_requests_total Total number of requests\n");
        output.push_str("# TYPE qrng_requests_total counter\n");
        output.push_str(&format!("qrng_requests_total {}\n", self.requests_total()));
        
        output.push_str("# HELP qrng_requests_failed Total number of failed requests\n");
        output.push_str("# TYPE qrng_requests_failed counter\n");
        output.push_str(&format!("qrng_requests_failed {}\n", self.requests_failed()));
        
        output.push_str("# HELP qrng_bytes_served Total bytes served\n");
        output.push_str("# TYPE qrng_bytes_served counter\n");
        output.push_str(&format!("qrng_bytes_served {}\n", self.bytes_served()));
        
        output.push_str("# HELP qrng_uptime_seconds Service uptime in seconds\n");
        output.push_str("# TYPE qrng_uptime_seconds gauge\n");
        output.push_str(&format!("qrng_uptime_seconds {}\n", self.uptime_seconds()));
        
        if let Some(p50) = self.latency_p50() {
            output.push_str("# HELP qrng_latency_p50_microseconds Request latency 50th percentile\n");
            output.push_str("# TYPE qrng_latency_p50_microseconds gauge\n");
            output.push_str(&format!("qrng_latency_p50_microseconds {}\n", p50));
        }
        
        if let Some(p99) = self.latency_p99() {
            output.push_str("# HELP qrng_latency_p99_microseconds Request latency 99th percentile\n");
            output.push_str("# TYPE qrng_latency_p99_microseconds gauge\n");
            output.push_str(&format!("qrng_latency_p99_microseconds {}\n", p99));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics() {
        let metrics = Metrics::new();
        
        metrics.record_request(1024, 100);
        metrics.record_request(2048, 200);
        metrics.record_request_failure();
        
        assert_eq!(metrics.requests_total(), 2);
        assert_eq!(metrics.requests_failed(), 1);
        assert_eq!(metrics.bytes_served(), 3072);
    }

    #[test]
    fn test_latency_percentiles() {
        let metrics = Metrics::new();
        
        for i in 1..=100 {
            metrics.record_request(100, i);
        }
        
        let p50 = metrics.latency_p50().unwrap();
        assert!((45..=55).contains(&p50));
        
        let p99 = metrics.latency_p99().unwrap();
        assert!((95..=100).contains(&p99));
    }
}
