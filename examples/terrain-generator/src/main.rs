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
use noise::{NoiseFn, Perlin};

#[derive(Parser)]
#[command(about = "Generate procedural terrain using quantum-seeded Perlin noise")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "256")]
    width: usize,

    #[arg(short, long, default_value = "256")]
    height: usize,

    #[arg(short, long, default_value = "0.05")]
    scale: f64,
}

fn main() {
    let args = Args::parse();

    let seed = get_quantum_seed(&args.gateway_url, &args.api_key);
    let perlin = Perlin::new(seed);

    println!("P2");
    println!("{} {}", args.width, args.height);
    println!("255");

    for y in 0..args.height {
        for x in 0..args.width {
            let nx = x as f64 * args.scale;
            let ny = y as f64 * args.scale;
            
            let value = perlin.get([nx, ny]);
            let normalized = ((value + 1.0) / 2.0 * 255.0) as u8;
            
            println!("{}", normalized);
        }
    }

    eprintln!("Generated {}x{} terrain heightmap with quantum seed {}", 
              args.width, args.height, seed);
}

fn get_quantum_seed(gateway_url: &str, api_key: &str) -> u32 {
    let url = format!(
        "{}/api/random?bytes=4&encoding=hex&api_key={}",
        gateway_url, api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let hex_data = response.text().expect("Failed to read response");
    let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");

    let mut array = [0u8; 4];
    array.copy_from_slice(&bytes);
    u32::from_le_bytes(array)
}
