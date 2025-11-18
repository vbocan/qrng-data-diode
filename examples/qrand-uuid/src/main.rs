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
use uuid::Uuid;

#[derive(Parser)]
#[command(about = "Generate UUIDs using quantum random entropy")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "1")]
    count: usize,

    #[arg(long)]
    no_hyphens: bool,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        let url = format!(
            "{}/api/random?bytes=16&encoding=hex&api_key={}",
            args.gateway_url, args.api_key
        );

        let response = reqwest::blocking::get(&url)
            .expect("Failed to contact gateway");
        
        let hex_data = response.text().expect("Failed to read response");
        let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");
        
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&bytes);
        
        uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40;
        uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;
        
        let uuid = Uuid::from_bytes(uuid_bytes);
        
        if args.no_hyphens {
            println!("{}", uuid.simple());
        } else {
            println!("{}", uuid);
        }
    }
}
