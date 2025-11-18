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
#[command(about = "Simulate random walk using quantum randomness")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "1000")]
    steps: usize,

    #[arg(long, default_value = "2")]
    dimensions: usize,

    #[arg(long)]
    csv: bool,
}

fn main() {
    let args = Args::parse();

    if args.dimensions < 1 || args.dimensions > 3 {
        eprintln!("Error: Dimensions must be 1, 2, or 3");
        std::process::exit(1);
    }

    let mut position = vec![0.0; args.dimensions];
    let mut trajectory = Vec::new();
    trajectory.push(position.clone());

    let batch_size = 2000;
    let num_batches = (args.steps + batch_size - 1) / batch_size;

    for batch in 0..num_batches {
        let current_batch_size = if batch == num_batches - 1 {
            args.steps - batch * batch_size
        } else {
            batch_size
        };

        let random_data = get_random_floats(&args.gateway_url, &args.api_key, current_batch_size * args.dimensions);

        for step in 0..current_batch_size {
            for dim in 0..args.dimensions {
                let angle = random_data[step * args.dimensions + dim] * 2.0 * std::f64::consts::PI;
                position[dim] += angle.cos();
            }
            trajectory.push(position.clone());
        }
    }

    if args.csv {
        if args.dimensions == 2 {
            println!("step,x,y");
        } else if args.dimensions == 3 {
            println!("step,x,y,z");
        } else {
            println!("step,x");
        }
        
        for (i, pos) in trajectory.iter().enumerate() {
            print!("{}", i);
            for val in pos {
                print!(",{:.6}", val);
            }
            println!();
        }
    } else {
        let final_pos = &trajectory[trajectory.len() - 1];
        let distance = final_pos.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        println!("Random Walk Simulation");
        println!("Steps: {}", args.steps);
        println!("Dimensions: {}", args.dimensions);
        println!("Final position: {:?}", final_pos);
        println!("Distance from origin: {:.6}", distance);
        println!("Mean squared displacement: {:.6}", distance * distance / args.steps as f64);
    }
}

fn get_random_floats(gateway_url: &str, api_key: &str, count: usize) -> Vec<f64> {
    let bytes_needed = count * 8;
    let url = format!(
        "{}/api/random?bytes={}&encoding=hex&api_key={}",
        gateway_url, bytes_needed, api_key
    );

    let response = reqwest::blocking::get(&url)
        .expect("Failed to contact gateway");
    
    let hex_data = response.text().expect("Failed to read response");
    let bytes = hex::decode(hex_data.trim()).expect("Invalid hex data");

    bytes.chunks(8)
        .map(|chunk| {
            let mut array = [0u8; 8];
            array.copy_from_slice(chunk);
            let random_u64 = u64::from_le_bytes(array);
            (random_u64 as f64) / (u64::MAX as f64)
        })
        .collect()
}
