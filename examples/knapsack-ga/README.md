# Knapsack Problem - Genetic Algorithm

Solve the 0/1 knapsack problem using a genetic algorithm with quantum random operations.

## Usage

```bash
# Run with default parameters
cargo run --release

# Larger population and more generations
cargo run --release -- --population 100 --generations 200

# Quick test
cargo run --release -- --population 20 --generations 50
```

## Problem

Given items with weights and values, select items to maximize total value while staying within weight limit.

## Algorithm

Genetic algorithm steps:
1. Initialize random population of solutions
2. Evaluate fitness (total value if within weight limit, 0 otherwise)
3. Select parents based on fitness
4. Create offspring via crossover
5. Apply random mutations
6. Repeat for multiple generations

Quantum randomness is used for:
- Initial population generation
- Parent selection (roulette wheel)
- Crossover points
- Mutation decisions

## Options

- `--population, -p`: Population size (default: 50)
- `--generations, -g`: Number of generations (default: 100)

## Output

Shows progress every 10 generations and final best solution with total value and weight.
