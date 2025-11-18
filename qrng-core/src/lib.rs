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

//! QRNG Core Library
//!
//! This crate provides the foundational types, traits, and utilities for the QRNG Data Diode system.
//! It implements a software-based data diode emulation for securely bridging quantum random number
//! generators to external networks.
//!
//! # Architecture
//!
//! The library is organized into modules representing core concerns:
//! - `protocol`: Data packet format and serialization
//! - `config`: Configuration management with validation
//! - `buffer`: High-performance entropy buffer with FIFO semantics
//! - `crypto`: Cryptographic primitives (HMAC, CRC32)
//! - `fetcher`: Resilient HTTPS client for QRNG appliance
//! - `error`: Unified error types
//!
//! # Design Principles
//!
//! 1. **Zero-cost abstractions**: Compile-time dispatch where possible
//! 2. **Type safety**: Leverage Rust's type system to prevent bugs
//! 3. **Composability**: Small, focused modules with clear interfaces
//! 4. **Testability**: Mock-friendly designs with dependency injection
//! 5. **Performance**: Lock-free data structures, zero-copy operations

pub mod buffer;
pub mod config;
pub mod crypto;
pub mod error;
pub mod fetcher;
pub mod mixer;
pub mod protocol;
pub mod metrics;
pub mod retry;

pub use error::{Error, Result};
pub use buffer::OverflowPolicy;

/// Library version for protocol compatibility
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum single request size to prevent OOM
pub const MAX_REQUEST_SIZE: usize = 65_536; // 64 KiB

/// Default buffer capacity (10 MiB)
pub const DEFAULT_BUFFER_SIZE: usize = 10 * 1024 * 1024;

/// Default fetch chunk size (1 KiB)
pub const DEFAULT_CHUNK_SIZE: usize = 1024;
