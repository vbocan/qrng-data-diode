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

//! Secure Key Generator using Quantum Random Data
//!
//! This example demonstrates how to generate cryptographic keys using the QRNG Gateway API.
//! It requests quantum random bytes and formats them as cryptographic keys.

use base64::Engine;
use clap::Parser;

/// CLI arguments for the key generator
#[derive(Parser, Debug)]
#[command(name = "secure-key-generator")]
#[command(about = "Generate cryptographic keys using quantum random data")]
struct Args {
    /// Gateway API URL
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    /// API key for authentication
    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    /// Key size in bytes (32 = AES-256, 16 = AES-128)
    #[arg(short, long, default_value = "32")]
    key_size: usize,

    /// Output format: hex or base64
    #[arg(short, long, default_value = "hex")]
    format: String,
}

fn main() {
    let args = Args::parse();

    println!("Quantum Secure Key Generator\n");

    // Display key type based on size
    let key_type = match args.key_size {
        16 => "AES-128",
        24 => "AES-192",
        32 => "AES-256",
        _ => "Custom",
    };
    println!("Generating {} key ({} bytes)...", key_type, args.key_size);

    // Request quantum random bytes from the gateway
    // The gateway returns plain text (hex or base64), not JSON
    let url = format!(
        "{}/api/random?bytes={}&encoding={}&api_key={}",
        args.gateway_url, args.key_size, args.format, args.api_key
    );

    // Make HTTP request to the gateway
    // Architecture: We use blocking I/O here to keep the example simple
    // In production, you'd want async/await for better performance
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .send()
        .expect("Failed to contact gateway");

    // Gateway returns plain text (the encoded random data)
    let random_key = response
        .text()
        .expect("Failed to read gateway response");

    // Display the generated key
    println!("\nGenerated key from quantum entropy:\n");
    println!("{}\n", random_key);
    
    println!("Key details:");
    println!("  Size: {} bytes", args.key_size);
    println!("  Format: {}", args.format);
    println!("  Entropy source: Quantum Random Number Generator\n");

    // Educational note about key usage
    println!("Usage Notes:");
    println!("  Store this key securely (e.g., in a password manager or hardware security module)");
    println!("  Never share or transmit keys over insecure channels");
    println!("  This key has maximum entropy from quantum processes");
    
    // Optional: Show alternative encoding if hex was requested
    if args.format == "hex" {
        let bytes = hex::decode(&random_key).expect("Invalid hex data");
        let base64_key = base64::engine::general_purpose::STANDARD.encode(&bytes);
        println!("\nAlternative encoding (base64): {}", base64_key);
    } else if args.format == "base64" {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&random_key)
            .expect("Invalid base64 data");
        let hex_key = hex::encode(&bytes);
        println!("\nAlternative encoding (hex): {}", hex_key);
    }
}
