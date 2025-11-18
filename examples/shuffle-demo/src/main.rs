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
#[command(about = "Shuffle items using Fisher-Yates algorithm with quantum randomness")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(trailing_var_arg = true, help = "Items to shuffle")]
    items: Vec<String>,

    #[arg(long)]
    cards: bool,
}

fn main() {
    let args = Args::parse();
    
    let mut items = if args.cards {
        generate_deck()
    } else if !args.items.is_empty() {
        args.items
    } else {
        (1..=10).map(|n| n.to_string()).collect()
    };
    
    println!("Original: {}", items.join(" "));
    
    fisher_yates_shuffle(&mut items, &args.gateway_url, &args.api_key);
    
    println!("Shuffled: {}", items.join(" "));
}

fn fisher_yates_shuffle(items: &mut [String], gateway_url: &str, api_key: &str) {
    let n = items.len();
    if n <= 1 {
        return;
    }
    
    let random_bytes = get_random_bytes(gateway_url, api_key, n - 1);
    
    for i in (1..n).rev() {
        let random_byte = random_bytes[n - 1 - i];
        let j = (random_byte as usize) % (i + 1);
        
        items.swap(i, j);
    }
}

fn generate_deck() -> Vec<String> {
    let suits = ["♠", "♥", "♦", "♣"];
    let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
    
    let mut deck = Vec::new();
    for suit in suits {
        for rank in ranks {
            deck.push(format!("{}{}", rank, suit));
        }
    }
    deck
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
