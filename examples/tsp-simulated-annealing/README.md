# Traveling Salesman Problem - Simulated Annealing

Solve the Traveling Salesman Problem using simulated annealing with quantum random decisions.

## Usage

```bash
# Solve for 10 cities
cargo run --release

# Solve for 20 cities with more iterations
cargo run --release -- --cities 20 --iterations 50000

# Quick test with 5 cities
cargo run --release -- --cities 5 --iterations 5000
```

## Algorithm

Simulated annealing is a probabilistic optimization technique that:
1. Starts with a random tour
2. Randomly swaps two cities
3. Accepts improvements, and sometimes accepts worse solutions based on temperature
4. Temperature decreases over time, reducing acceptance of worse solutions

Quantum randomness is used for:
- City swap selection
- Acceptance probability decisions

## Options

- `--cities, -c`: Number of cities (default: 10)
- `--iterations, -i`: Number of iterations (default: 10000)

## Output

Shows progress every 1000 iterations and final best tour with total distance.
