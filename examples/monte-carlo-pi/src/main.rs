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

use clap::Parser;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(about = "Estimate π using Monte Carlo method with quantum randomness")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "1000000")]
    samples: usize,
}

fn main() {
    let args = Args::parse();
    
    println!("Estimating π using {} samples", args.samples);
    
    let mut inside_circle = 0;
    let mut total_processed = 0;
    
    // Use smaller batches to avoid overwhelming the gateway
    let batch_size = 1000;
    let num_batches = (args.samples + batch_size - 1) / batch_size;
    
    for batch in 0..num_batches {
        let current_batch_size = if batch == num_batches - 1 {
            args.samples - batch * batch_size
        } else {
            batch_size
        };
        
        let bytes_needed = current_batch_size * 16;
        let random_data = get_random_bytes(&args.gateway_url, &args.api_key, bytes_needed);
        
        if random_data.len() < bytes_needed {
            eprintln!("Warning: Requested {} bytes but got {} - stopping early", 
                     bytes_needed, random_data.len());
            break;
        }
        
        for i in 0..current_batch_size {
            let offset = i * 16;
            
            let mut x_bytes = [0u8; 8];
            x_bytes.copy_from_slice(&random_data[offset..offset + 8]);
            let x = bytes_to_float(&x_bytes);
            
            let mut y_bytes = [0u8; 8];
            y_bytes.copy_from_slice(&random_data[offset + 8..offset + 16]);
            let y = bytes_to_float(&y_bytes);
            
            if x * x + y * y <= 1.0 {
                inside_circle += 1;
            }
            total_processed += 1;
        }
        
        // Small delay to avoid overwhelming the gateway
        if batch < num_batches - 1 {
            thread::sleep(Duration::from_millis(10));
        }
        
        if (batch + 1) % 100 == 0 || batch == num_batches - 1 {
            let pi_estimate = 4.0 * inside_circle as f64 / total_processed as f64;
            let error = (pi_estimate - std::f64::consts::PI).abs();
            println!("Processed: {} | π estimate: {:.6} | Error: {:.6}", 
                     total_processed, pi_estimate, error);
        }
    }
    
    let pi_estimate = 4.0 * inside_circle as f64 / total_processed as f64;
    let error = (pi_estimate - std::f64::consts::PI).abs();
    
    println!("\nFinal Results:");
    println!("Samples: {}", total_processed);
    println!("Inside circle: {}", inside_circle);
    println!("π estimate: {:.10}", pi_estimate);
    println!("Actual π: {:.10}", std::f64::consts::PI);
    println!("Absolute error: {:.10}", error);
    println!("Relative error: {:.6}%", (error / std::f64::consts::PI) * 100.0);
}

fn get_random_bytes(gateway_url: &str, api_key: &str, count: usize) -> Vec<u8> {
    let url = format!(
        "{}/api/random?bytes={}&encoding=hex&api_key={}",
        gateway_url, count, api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let hex_data = response.text().expect("Failed to read response");
    hex::decode(hex_data.trim()).expect("Invalid hex data")
}

fn bytes_to_float(bytes: &[u8; 8]) -> f64 {
    let random_u64 = u64::from_le_bytes(*bytes);
    (random_u64 as f64) / (u64::MAX as f64)
}
