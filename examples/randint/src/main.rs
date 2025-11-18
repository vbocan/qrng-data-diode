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

#[derive(Parser)]
#[command(about = "Generate random integers using quantum entropy")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "0")]
    min: i64,

    #[arg(short = 'M', long, default_value = "100")]
    max: i64,

    #[arg(short, long, default_value = "1")]
    count: usize,
}

fn main() {
    let args = Args::parse();

    if args.min >= args.max {
        eprintln!("Error: min must be less than max");
        std::process::exit(1);
    }

    let range = (args.max - args.min) as u64;
    
    for _ in 0..args.count {
        let url = format!(
            "{}/api/random?bytes=8&encoding=hex&api_key={}",
            args.gateway_url, args.api_key
        );

        let response = reqwest::blocking::get(&url)
            .expect("Failed to contact gateway");
        
        let hex_data = response.text().expect("Failed to read response");
        let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");
        
        let mut array = [0u8; 8];
        array.copy_from_slice(&bytes[0..8]);
        let random_u64 = u64::from_le_bytes(array);
        
        let result = args.min + (random_u64 % range) as i64;
        
        println!("{}", result);
    }
}
