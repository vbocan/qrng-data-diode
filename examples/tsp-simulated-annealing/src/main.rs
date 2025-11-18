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
#[command(about = "Solve TSP using simulated annealing with quantum randomness")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "10")]
    cities: usize,

    #[arg(short, long, default_value = "10000")]
    iterations: usize,
}

fn main() {
    let args = Args::parse();

    let coordinates = generate_cities(args.cities, &args.gateway_url, &args.api_key);
    let mut tour: Vec<usize> = (0..args.cities).collect();
    let mut best_tour = tour.clone();
    let mut best_distance = calculate_distance(&tour, &coordinates);

    println!("Solving TSP for {} cities using simulated annealing", args.cities);
    println!("Initial tour distance: {:.2}", best_distance);

    let batch_size = 1000;
    let num_batches = (args.iterations + batch_size - 1) / batch_size;

    for batch in 0..num_batches {
        let current_batch_size = if batch == num_batches - 1 {
            args.iterations - batch * batch_size
        } else {
            batch_size
        };

        let random_data = get_random_floats(&args.gateway_url, &args.api_key, current_batch_size * 3);

        for i in 0..current_batch_size {
            let iteration = batch * batch_size + i;
            let t = 1.0 - (iteration as f64 / args.iterations as f64);
            let temperature = t * 100.0;

            let city_i = (random_data[i * 3] * args.cities as f64) as usize % args.cities;
            let city_j = (random_data[i * 3 + 1] * args.cities as f64) as usize % args.cities;

            tour.swap(city_i, city_j);
            let new_distance = calculate_distance(&tour, &coordinates);

            let accept = if new_distance < best_distance {
                true
            } else {
                let delta = new_distance - best_distance;
                let probability = (-delta / temperature).exp();
                random_data[i * 3 + 2] < probability
            };

            if accept {
                best_distance = new_distance;
                best_tour = tour.clone();
            } else {
                tour.swap(city_i, city_j);
            }

            if (iteration + 1) % 5000 == 0 {
                println!("Iteration {}: distance = {:.2}, temp = {:.2}", 
                         iteration + 1, best_distance, temperature);
            }
        }
    }

    println!("\nFinal Results:");
    println!("Best tour: {:?}", best_tour);
    println!("Best distance: {:.2}", best_distance);
}

fn generate_cities(count: usize, gateway_url: &str, api_key: &str) -> Vec<(f64, f64)> {
    let random_data = get_random_floats(gateway_url, api_key, count * 2);
    
    random_data.chunks(2)
        .map(|chunk| (chunk[0] * 100.0, chunk[1] * 100.0))
        .collect()
}

fn calculate_distance(tour: &[usize], coordinates: &[(f64, f64)]) -> f64 {
    let mut distance = 0.0;
    
    for i in 0..tour.len() {
        let current = coordinates[tour[i]];
        let next = coordinates[tour[(i + 1) % tour.len()]];
        
        let dx = current.0 - next.0;
        let dy = current.1 - next.1;
        distance += (dx * dx + dy * dy).sqrt();
    }
    
    distance
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
