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

//! Protocol data structures for entropy transmission
//!
//! Defines the wire format for entropy packets transmitted from Collector to Gateway.
//! Uses MessagePack for efficient binary serialization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entropy packet transmitted from Collector to Gateway
///
/// This structure is optimized for:
/// - Integrity verification (HMAC + CRC32)
/// - Ordering and gap detection (sequence numbers)
/// - Freshness tracking (timestamps)
/// - Efficient serialization (MessagePack)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyPacket {
    /// Protocol version for forward compatibility
    pub version: u8,

    /// Unique packet identifier
    pub id: Uuid,

    /// Monotonically increasing sequence number
    pub sequence: u64,

    /// Random entropy payload
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,

    /// UTC timestamp when packet was created
    pub timestamp: DateTime<Utc>,

    /// HMAC-SHA256 signature over (version || sequence || data || timestamp)
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,

    /// Optional CRC32 checksum for additional integrity
    pub checksum: Option<u32>,
}

impl EntropyPacket {
    /// Current protocol version
    pub const VERSION: u8 = 1;

    /// Create a new entropy packet
    pub fn new(sequence: u64, data: Vec<u8>) -> Self {
        Self {
            version: Self::VERSION,
            id: Uuid::new_v4(),
            sequence,
            data,
            timestamp: Utc::now(),
            signature: Vec::new(),
            checksum: None,
        }
    }

    /// Calculate CRC32 checksum of payload
    pub fn calculate_checksum(&self) -> u32 {
        crc32fast::hash(&self.data)
    }

    /// Verify CRC32 checksum if present
    pub fn verify_checksum(&self) -> bool {
        match self.checksum {
            Some(expected) => expected == self.calculate_checksum(),
            None => true, // No checksum to verify
        }
    }

    /// Get payload size in bytes
    pub fn payload_size(&self) -> usize {
        self.data.len()
    }

    /// Check if packet is stale (older than threshold)
    pub fn is_stale(&self, threshold: chrono::Duration) -> bool {
        Utc::now().signed_duration_since(self.timestamp) > threshold
    }

    /// Serialize to MessagePack
    pub fn to_msgpack(&self) -> crate::Result<Vec<u8>> {
        rmp_serde::to_vec(self).map_err(Into::into)
    }

    /// Deserialize from MessagePack
    pub fn from_msgpack(bytes: &[u8]) -> crate::Result<Self> {
        rmp_serde::from_slice(bytes).map_err(Into::into)
    }
}

/// Health status for system monitoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Gateway status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    /// Overall health status
    pub status: HealthStatus,

    /// Buffer fill percentage (0-100)
    pub buffer_fill_percent: f64,

    /// Available bytes in buffer
    pub buffer_bytes_available: usize,

    /// Timestamp of last data received/fetched
    pub last_data_received: Option<DateTime<Utc>>,

    /// Age of oldest data in seconds
    pub data_freshness_seconds: Option<u64>,

    /// Service uptime in seconds
    pub uptime_seconds: u64,

    /// Total requests served
    pub total_requests_served: u64,

    /// Total bytes served
    pub total_bytes_served: u64,

    /// Current requests per second
    pub requests_per_second: f64,

    /// Any warnings or issues
    pub warnings: Vec<String>,
}

/// Encoding format for served entropy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    /// Raw binary data
    Binary,
    /// Hexadecimal encoding
    Hex,
    /// Base64 encoding
    Base64,
}

impl EncodingFormat {
    /// Parse from string (case-insensitive)
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "binary" | "raw" => Some(Self::Binary),
            "hex" | "hexadecimal" => Some(Self::Hex),
            "base64" | "b64" => Some(Self::Base64),
            _ => None,
        }
    }

    /// Get MIME type for this encoding
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Binary => "application/octet-stream",
            Self::Hex => "text/plain; charset=utf-8",
            Self::Base64 => "text/plain; charset=utf-8",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let packet = EntropyPacket::new(42, vec![1, 2, 3, 4, 5]);
        let bytes = packet.to_msgpack().unwrap();
        let decoded = EntropyPacket::from_msgpack(&bytes).unwrap();
        assert_eq!(packet.sequence, decoded.sequence);
        assert_eq!(packet.data, decoded.data);
    }

    #[test]
    fn test_checksum() {
        let mut packet = EntropyPacket::new(1, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        packet.checksum = Some(packet.calculate_checksum());
        assert!(packet.verify_checksum());
    }

    #[test]
    fn test_encoding_format() {
        assert_eq!(EncodingFormat::parse("hex"), Some(EncodingFormat::Hex));
        assert_eq!(EncodingFormat::parse("HEX"), Some(EncodingFormat::Hex));
        assert_eq!(EncodingFormat::parse("base64"), Some(EncodingFormat::Base64));
        assert_eq!(EncodingFormat::parse("invalid"), None);
    }
}
