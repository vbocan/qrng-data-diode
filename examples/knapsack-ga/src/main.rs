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
#[command(about = "Solve 0/1 knapsack using genetic algorithm with quantum randomness")]
struct Args {
    #[arg(long, default_value = "http://localhost:7764")]
    gateway_url: String,

    #[arg(long, default_value = "test-key-1234567890")]
    api_key: String,

    #[arg(short, long, default_value = "50")]
    population: usize,

    #[arg(short, long, default_value = "100")]
    generations: usize,
}

struct Item {
    weight: u32,
    value: u32,
}

const MAX_WEIGHT: u32 = 50;
const ITEMS: &[(u32, u32)] = &[
    (10, 60), (20, 100), (30, 120), (5, 30), (15, 80),
    (8, 50), (12, 70), (25, 110), (7, 40), (18, 90),
];

fn main() {
    let args = Args::parse();

    let items: Vec<Item> = ITEMS.iter()
        .map(|&(w, v)| Item { weight: w, value: v })
        .collect();

    println!("Solving 0/1 Knapsack Problem");
    println!("Items: {}, Max weight: {}", items.len(), MAX_WEIGHT);

    let mut population = initialize_population(args.population, items.len(), &args.gateway_url, &args.api_key);
    let mut best_solution = population[0].clone();
    let mut best_fitness = fitness(&best_solution, &items);

    for gen in 0..args.generations {
        let fitnesses: Vec<u32> = population.iter()
            .map(|chromosome| fitness(chromosome, &items))
            .collect();

        let max_fitness = *fitnesses.iter().max().unwrap();
        if max_fitness > best_fitness {
            best_fitness = max_fitness;
            let best_idx = fitnesses.iter().position(|&f| f == max_fitness).unwrap();
            best_solution = population[best_idx].clone();
        }

        let random_data = get_random_floats(&args.gateway_url, &args.api_key, args.population * 3);
        let mut new_population = Vec::new();

        for i in 0..args.population {
            let parent1 = select_parent(&population, &fitnesses, random_data[i * 3]);
            let parent2 = select_parent(&population, &fitnesses, random_data[i * 3 + 1]);
            
            let mut child = crossover(&parent1, &parent2, random_data[i * 3 + 2]);
            
            if random_data[i * 3 + 2] < 0.1 {
                mutate(&mut child, random_data[i * 3 + 2]);
            }
            
            new_population.push(child);
        }

        population = new_population;

        if (gen + 1) % 10 == 0 {
            println!("Generation {}: best fitness = {}", gen + 1, best_fitness);
        }
    }

    let total_weight: u32 = best_solution.iter().enumerate()
        .filter(|(_, &gene)| gene)
        .map(|(i, _)| items[i].weight)
        .sum();

    println!("\nFinal Results:");
    println!("Best solution: {:?}", best_solution);
    println!("Total value: {}", best_fitness);
    println!("Total weight: {}/{}", total_weight, MAX_WEIGHT);
}

fn initialize_population(size: usize, genes: usize, gateway_url: &str, api_key: &str) -> Vec<Vec<bool>> {
    let random_data = get_random_bytes(gateway_url, api_key, size * genes);
    
    (0..size)
        .map(|i| {
            (0..genes)
                .map(|j| random_data[i * genes + j] > 127)
                .collect()
        })
        .collect()
}

fn fitness(chromosome: &[bool], items: &[Item]) -> u32 {
    let mut total_weight = 0;
    let mut total_value = 0;

    for (i, &gene) in chromosome.iter().enumerate() {
        if gene {
            total_weight += items[i].weight;
            total_value += items[i].value;
        }
    }

    if total_weight > MAX_WEIGHT {
        0
    } else {
        total_value
    }
}

fn select_parent(population: &[Vec<bool>], fitnesses: &[u32], random: f64) -> Vec<bool> {
    let total_fitness: u32 = fitnesses.iter().sum();
    let mut cumulative = 0;
    let target = (random * total_fitness as f64) as u32;

    for (i, &fitness) in fitnesses.iter().enumerate() {
        cumulative += fitness;
        if cumulative >= target {
            return population[i].clone();
        }
    }

    population[population.len() - 1].clone()
}

fn crossover(parent1: &[bool], parent2: &[bool], random: f64) -> Vec<bool> {
    let point = (random * parent1.len() as f64) as usize;
    
    parent1.iter().take(point)
        .chain(parent2.iter().skip(point))
        .copied()
        .collect()
}

fn mutate(chromosome: &mut [bool], random: f64) {
    let index = (random * chromosome.len() as f64) as usize % chromosome.len();
    chromosome[index] = !chromosome[index];
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

fn get_random_floats(gateway_url: &str, api_key: &str, count: usize) -> Vec<f64> {
    let bytes_needed = count * 8;
    let bytes = get_random_bytes(gateway_url, api_key, bytes_needed);

    bytes.chunks(8)
        .map(|chunk| {
            let mut array = [0u8; 8];
            array.copy_from_slice(chunk);
            let random_u64 = u64::from_le_bytes(array);
            (random_u64 as f64) / (u64::MAX as f64)
        })
        .collect()
}
