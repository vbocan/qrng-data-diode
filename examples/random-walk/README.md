# Random Walk Simulator

Simulate random walks in 1D, 2D, or 3D using quantum randomness.

## Usage

```bash
# 2D random walk with 1000 steps
cargo run --release

# 3D random walk with 5000 steps
cargo run --release -- --steps 5000 --dimensions 3

# Export trajectory as CSV
cargo run --release -- --steps 1000 --csv > walk.csv
```

## Options

- `--steps, -s`: Number of steps in the walk (default: 1000)
- `--dimensions`: Dimensionality (1, 2, or 3) (default: 2)
- `--csv`: Output trajectory as CSV format

## Applications

Random walks model Brownian motion, particle diffusion, stock prices, and polymer chains. Quantum randomness ensures unbiased sampling of directions.

## Output

In normal mode, shows final position and statistics. In CSV mode, outputs step-by-step positions for plotting and analysis.
