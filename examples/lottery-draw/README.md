# Quantum Lottery Draw

Perform provably fair lottery draws using quantum random selection.

## Usage

```bash
# Standard 6/49 lottery
cargo run --release

# 5/35 lottery
cargo run --release -- --draw 5 --pool 35

# Multiple draws
cargo run --release -- --count 10

# Mega Millions style (5/70)
cargo run --release -- --draw 5 --pool 70
```

## Options

- `--draw, -d`: Number of balls to draw (default: 6)
- `--pool, -p`: Size of the number pool (default: 49)
- `--count, -c`: Number of independent draws to perform (default: 1)

## Implementation

Uses quantum random selection to draw unique numbers from the pool. Each number has equal probability of being selected, and the quantum source ensures the draw cannot be predicted or manipulated.

Applications include gaming, raffles, statistical sampling, and any scenario requiring verifiable randomness.
