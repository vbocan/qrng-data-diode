//! High-performance entropy buffer with FIFO semantics and TTL management
//!
//! This module implements a thread-safe, efficient circular buffer for storing
//! random entropy with automatic age-based eviction and watermark monitoring.

use crate::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

/// Entry in the entropy buffer with timestamp tracking
#[derive(Debug, Clone)]
struct BufferEntry {
    data: Bytes,
    timestamp: DateTime<Utc>,
}

/// Thread-safe entropy buffer with FIFO semantics
///
/// # Design
///
/// - Uses `parking_lot::RwLock` for efficient concurrent access
/// - Stores data in chunks with timestamps for TTL enforcement
/// - Implements automatic eviction policies (age-based, overflow)
/// - Provides watermark-based monitoring
///
/// # Performance
///
/// - Lock-free reads when possible
/// - Zero-copy operations using `Bytes`
/// - O(1) push and pop operations
#[derive(Clone)]
pub struct EntropyBuffer {
    inner: Arc<RwLock<BufferInner>>,
}

struct BufferInner {
    entries: VecDeque<BufferEntry>,
    max_size: usize,
    current_size: usize,
    ttl: Option<Duration>,
    stats: BufferStats,
}

#[derive(Debug, Clone, Default)]
pub struct BufferStats {
    pub total_pushes: u64,
    pub total_pops: u64,
    pub bytes_pushed: u64,
    pub bytes_popped: u64,
    pub evictions_overflow: u64,
    pub evictions_ttl: u64,
}

/// Buffer watermark levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatermarkLevel {
    Low,      // < 10%
    Medium,   // 10-80%
    High,     // 80-95%
    Critical, // > 95%
}

impl EntropyBuffer {
    /// Create a new buffer with specified capacity
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BufferInner {
                entries: VecDeque::new(),
                max_size,
                current_size: 0,
                ttl: None,
                stats: BufferStats::default(),
            })),
        }
    }

    /// Create buffer with TTL for automatic age-based eviction
    pub fn with_ttl(max_size: usize, ttl: Duration) -> Self {
        let mut buffer = Self::new(max_size);
        buffer.inner.write().ttl = Some(ttl);
        buffer
    }

    /// Push entropy data into buffer
    ///
    /// Automatically evicts stale or overflow data as needed.
    /// Returns the number of bytes actually stored.
    pub fn push(&self, data: impl Into<Bytes>) -> Result<usize> {
        let data = data.into();
        let data_len = data.len();

        if data_len == 0 {
            return Ok(0);
        }

        let mut inner = self.inner.write();

        // Evict stale data based on TTL
        if let Some(ttl) = inner.ttl {
            inner.evict_stale(ttl);
        }

        // Evict old data if buffer would overflow
        while inner.current_size + data_len > inner.max_size && !inner.entries.is_empty() {
            if let Some(entry) = inner.entries.pop_front() {
                inner.current_size -= entry.data.len();
                inner.stats.evictions_overflow += 1;
            }
        }

        // Don't push if single entry exceeds capacity
        if data_len > inner.max_size {
            return Err(Error::Buffer(format!(
                "Data size {} exceeds buffer capacity {}",
                data_len, inner.max_size
            )));
        }

        // Push new entry
        inner.entries.push_back(BufferEntry {
            data: data.clone(),
            timestamp: Utc::now(),
        });
        inner.current_size += data_len;
        inner.stats.total_pushes += 1;
        inner.stats.bytes_pushed += data_len as u64;

        Ok(data_len)
    }

    /// Pop exactly N bytes from buffer (FIFO)
    ///
    /// Returns None if insufficient data available.
    pub fn pop(&self, n: usize) -> Option<Bytes> {
        if n == 0 {
            return Some(Bytes::new());
        }

        let mut inner = self.inner.write();

        if inner.current_size < n {
            return None;
        }

        let mut result = BytesMut::with_capacity(n);
        let mut remaining = n;

        while remaining > 0 {
            let entry = inner.entries.front_mut()?;
            let available = entry.data.len();

            if available <= remaining {
                // Consume entire entry
                let consumed = inner.entries.pop_front()?;
                result.put(consumed.data);
                remaining -= available;
                inner.current_size -= available;
            } else {
                // Partial consumption
                let chunk = entry.data.split_to(remaining);
                result.put(chunk);
                inner.current_size -= remaining;
                remaining = 0;
            }
        }

        inner.stats.total_pops += 1;
        inner.stats.bytes_popped += n as u64;

        Some(result.freeze())
    }

    /// Peek at N bytes without consuming
    pub fn peek(&self, n: usize) -> Option<Bytes> {
        let inner = self.inner.read();

        if inner.current_size < n {
            return None;
        }

        let mut result = BytesMut::with_capacity(n);
        let mut remaining = n;
        let mut iter = inner.entries.iter();

        while remaining > 0 {
            let entry = iter.next()?;
            let available = entry.data.len();

            if available <= remaining {
                result.put(entry.data.clone());
                remaining -= available;
            } else {
                result.put(entry.data.slice(0..remaining));
                remaining = 0;
            }
        }

        Some(result.freeze())
    }

    /// Get current buffer utilization (bytes)
    pub fn len(&self) -> usize {
        self.inner.read().current_size
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get buffer capacity (bytes)
    pub fn capacity(&self) -> usize {
        self.inner.read().max_size
    }

    /// Get fill percentage (0.0 - 100.0)
    pub fn fill_percent(&self) -> f64 {
        let inner = self.inner.read();
        (inner.current_size as f64 / inner.max_size as f64) * 100.0
    }

    /// Get current watermark level
    pub fn watermark(&self) -> WatermarkLevel {
        let percent = self.fill_percent();
        match percent {
            p if p < 10.0 => WatermarkLevel::Low,
            p if p < 80.0 => WatermarkLevel::Medium,
            p if p < 95.0 => WatermarkLevel::High,
            _ => WatermarkLevel::Critical,
        }
    }

    /// Get timestamp of oldest data
    pub fn oldest_timestamp(&self) -> Option<DateTime<Utc>> {
        self.inner.read().entries.front().map(|e| e.timestamp)
    }

    /// Get age of oldest data in seconds
    pub fn freshness_seconds(&self) -> Option<u64> {
        self.oldest_timestamp().map(|ts| {
            Utc::now()
                .signed_duration_since(ts)
                .num_seconds()
                .max(0) as u64
        })
    }

    /// Get buffer statistics
    pub fn stats(&self) -> BufferStats {
        self.inner.read().stats.clone()
    }

    /// Clear all data from buffer
    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.entries.clear();
        inner.current_size = 0;
    }
}

impl BufferInner {
    fn evict_stale(&mut self, ttl: Duration) {
        let cutoff = Utc::now() - ttl;
        
        while let Some(entry) = self.entries.front() {
            if entry.timestamp < cutoff {
                let removed = self.entries.pop_front().unwrap();
                self.current_size -= removed.data.len();
                self.stats.evictions_ttl += 1;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let buffer = EntropyBuffer::new(1024);
        buffer.push(vec![1, 2, 3, 4]).unwrap();
        assert_eq!(buffer.len(), 4);

        let data = buffer.pop(4).unwrap();
        assert_eq!(data.as_ref(), &[1, 2, 3, 4]);
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_overflow_eviction() {
        let buffer = EntropyBuffer::new(10);
        buffer.push(vec![1; 8]).unwrap();
        buffer.push(vec![2; 8]).unwrap(); // Should evict first entry
        assert_eq!(buffer.len(), 8);
        
        let data = buffer.pop(8).unwrap();
        assert_eq!(data.as_ref(), &[2; 8]);
    }

    #[test]
    fn test_watermark() {
        let buffer = EntropyBuffer::new(100);
        assert_eq!(buffer.watermark(), WatermarkLevel::Low);

        buffer.push(vec![0; 50]).unwrap();
        assert_eq!(buffer.watermark(), WatermarkLevel::Medium);

        buffer.push(vec![0; 40]).unwrap();
        assert_eq!(buffer.watermark(), WatermarkLevel::High);
    }

    #[test]
    fn test_peek() {
        let buffer = EntropyBuffer::new(100);
        buffer.push(vec![1, 2, 3, 4, 5]).unwrap();
        
        let peeked = buffer.peek(3).unwrap();
        assert_eq!(peeked.as_ref(), &[1, 2, 3]);
        assert_eq!(buffer.len(), 5); // Not consumed
    }
}
