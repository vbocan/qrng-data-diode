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
use std::io::Write;

#[derive(Parser)]
#[command(about = "Generate random bytes using quantum entropy")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "32")]
    bytes: usize,

    #[arg(long, default_value = "hex")]
    format: String,
}

fn main() {
    let args = Args::parse();

    let url = format!(
        "{}/api/random?bytes={}&encoding={}&api_key={}",
        args.gateway_url, args.bytes, args.format, args.api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let data = response.text().expect("Failed to read response");
    
    if args.format == "binary" {
        let bytes = hex::decode(data.trim()).expect("Invalid hex data");
        std::io::stdout().write_all(&bytes).expect("Failed to write bytes");
    } else {
        println!("{}", data);
    }
}
