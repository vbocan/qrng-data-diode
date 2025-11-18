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

//! Error types for the QRNG system
//!
//! Provides a unified error taxonomy using `thiserror` for ergonomic error handling.

pub type Result<T> = std::result::Result<T, Error>;

/// Core error type for QRNG operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration validation failed
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network communication failed
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Cryptographic operation failed
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Buffer operation failed
    #[error("Buffer error: {0}")]
    Buffer(String),

    /// Data validation failed
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication failed
    #[error("Authentication failed")]
    Authentication,

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimit,

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Operation timed out
    #[error("Operation timed out")]
    Timeout,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Check if error is transient and retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network(_) | Error::Timeout | Error::RateLimit
        )
    }

    /// Check if error indicates authentication failure
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Authentication)
    }
}

// Conversions for common error types
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(e: rmp_serde::encode::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(e: rmp_serde::decode::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}
