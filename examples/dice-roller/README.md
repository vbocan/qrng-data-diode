# Quantum Dice Roller

Roll dice using quantum random numbers for tabletop gaming and simulations.

## Usage

```bash
# Roll a single d6
cargo run --release d6

# Roll 3 six-sided dice
cargo run --release 3d6

# Roll 2 twenty-sided dice with +5 modifier
cargo run --release 2d20+5

# Roll percentile dice
cargo run --release d100

# Roll with negative modifier
cargo run --release 4d6-2
```

## Dice Notation

Format: `[count]d<sides>[+/-modifier]`
- `count`: Number of dice (optional, default 1)
- `sides`: Number of sides per die
- `modifier`: Value to add/subtract from total (optional)

Examples: `d6`, `3d8`, `2d10+3`, `4d6-1`

## Implementation

Parses standard dice notation and generates quantum random rolls for each die. Each roll is independent and uses fresh quantum entropy from the gateway.
