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

use clap::Parser;

#[derive(Parser)]
#[command(about = "Statistical tests for randomness quality")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "100000")]
    samples: usize,
}

fn main() {
    let args = Args::parse();

    println!("Running randomness tests on {} samples", args.samples);
    println!();

    let data = get_random_bytes(&args.gateway_url, &args.api_key, args.samples);

    frequency_test(&data);
    runs_test(&data);
    chi_square_test(&data);
}

fn frequency_test(data: &[u8]) {
    let ones: usize = data.iter()
        .map(|&byte| byte.count_ones() as usize)
        .sum();
    
    let total_bits = data.len() * 8;
    let ones_ratio = ones as f64 / total_bits as f64;
    let expected = 0.5;
    let deviation = (ones_ratio - expected).abs();

    println!("Frequency Test (Monobit):");
    println!("  Total bits: {}", total_bits);
    println!("  Ones: {}", ones);
    println!("  Ones ratio: {:.6}", ones_ratio);
    println!("  Expected: {:.6}", expected);
    println!("  Deviation: {:.6}", deviation);
    println!("  Result: {}", if deviation < 0.01 { "PASS" } else { "FAIL" });
    println!();
}

fn runs_test(data: &[u8]) {
    let bits: Vec<bool> = data.iter()
        .flat_map(|&byte| (0..8).map(move |i| (byte >> i) & 1 == 1))
        .collect();

    let mut runs = 0;
    for i in 1..bits.len() {
        if bits[i] != bits[i - 1] {
            runs += 1;
        }
    }
    runs += 1;

    let n = bits.len() as f64;
    let expected_runs = (n / 2.0) + 1.0;
    let deviation = (runs as f64 - expected_runs).abs();

    println!("Runs Test:");
    println!("  Total runs: {}", runs);
    println!("  Expected runs: {:.0}", expected_runs);
    println!("  Deviation: {:.2}", deviation);
    println!("  Result: {}", if deviation < expected_runs * 0.1 { "PASS" } else { "FAIL" });
    println!();
}

fn chi_square_test(data: &[u8]) {
    let mut frequencies = [0usize; 256];
    for &byte in data {
        frequencies[byte as usize] += 1;
    }

    let expected = data.len() as f64 / 256.0;
    let chi_square: f64 = frequencies.iter()
        .map(|&observed| {
            let diff = observed as f64 - expected;
            (diff * diff) / expected
        })
        .sum();

    let degrees_of_freedom = 255;
    let critical_value = 293.25;

    println!("Chi-Square Test:");
    println!("  Chi-square value: {:.2}", chi_square);
    println!("  Degrees of freedom: {}", degrees_of_freedom);
    println!("  Critical value (α=0.05): {:.2}", critical_value);
    println!("  Result: {}", if chi_square < critical_value { "PASS" } else { "FAIL" });
    println!();
}

fn get_random_bytes(gateway_url: &str, api_key: &str, count: usize) -> Vec<u8> {
    println!("Fetching {} bytes of quantum random data...", count);
    
    let mut all_bytes = Vec::with_capacity(count);
    let max_chunk = 32768; // 32KB per request to stay well under 64KB limit
    let num_chunks = (count + max_chunk - 1) / max_chunk;
    
    for chunk_idx in 0..num_chunks {
        let chunk_size = if chunk_idx == num_chunks - 1 {
            count - chunk_idx * max_chunk
        } else {
            max_chunk
        };
        
        let url = format!(
            "{}/api/random?bytes={}&encoding=hex&api_key={}",
            gateway_url, chunk_size, api_key
        );

        let response = reqwest::blocking::get(&url)
            .expect("Failed to contact gateway");
        
        let hex_data = response.text().expect("Failed to read response");
        let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");
        
        all_bytes.extend_from_slice(&bytes);
        
        // Small delay between chunks
        if chunk_idx < num_chunks - 1 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    
    println!("Data retrieved successfully (fetched {} bytes in {} chunks)\n", 
             all_bytes.len(), num_chunks);
    all_bytes
}
