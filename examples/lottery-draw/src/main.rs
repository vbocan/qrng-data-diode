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
use std::collections::HashSet;

#[derive(Parser)]
#[command(about = "Perform lottery draws using quantum random selection")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "6", help = "Numbers to draw")]
    draw: usize,

    #[arg(short = 'p', long, default_value = "49", help = "Pool size")]
    pool: usize,

    #[arg(short, long, default_value = "1", help = "Number of draws")]
    count: usize,
}

fn main() {
    let args = Args::parse();

    if args.draw > args.pool {
        eprintln!("Error: Cannot draw more numbers than pool size");
        std::process::exit(1);
    }

    println!("Lottery Draw: {} numbers from pool of {}\n", args.draw, args.pool);

    for i in 1..=args.count {
        let numbers = draw_lottery(&args.gateway_url, &args.api_key, args.draw, args.pool);
        
        if args.count > 1 {
            println!("Draw {}: {}", i, format_numbers(&numbers));
        } else {
            println!("Winning numbers: {}", format_numbers(&numbers));
        }
    }
}

fn draw_lottery(gateway_url: &str, api_key: &str, draw_count: usize, pool_size: usize) -> Vec<usize> {
    let mut drawn = HashSet::new();
    
    while drawn.len() < draw_count {
        let num = get_random_int(gateway_url, api_key, 1, pool_size + 1);
        drawn.insert(num);
    }
    
    let mut numbers: Vec<usize> = drawn.into_iter().collect();
    numbers.sort();
    numbers
}

fn get_random_int(gateway_url: &str, api_key: &str, min: usize, max: usize) -> usize {
    let range = (max - min) as u64;
    
    let url = format!(
        "{}/api/random?bytes=8&encoding=hex&api_key={}",
        gateway_url, api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let hex_data = response.text().expect("Failed to read response");
    let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");
    
    let mut array = [0u8; 8];
    array.copy_from_slice(&bytes[0..8]);
    let random_u64 = u64::from_le_bytes(array);
    
    min + (random_u64 % range) as usize
}

fn format_numbers(numbers: &[usize]) -> String {
    numbers.iter()
        .map(|n| format!("{:2}", n))
        .collect::<Vec<_>>()
        .join(" ")
}
