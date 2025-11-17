# Random Integer Generator

Generate random integers in a specified range using quantum entropy from the gateway.

## Usage

```bash
# Generate a single random integer between 0 and 100
cargo run --release

# Generate 10 random integers between 1 and 6 (dice roll)
cargo run --release -- --min 1 --max 6 --count 10

# Generate random integers in custom range
cargo run --release -- --min -100 --max 100 --count 5
```

## Options

- `--gateway-url`: Gateway endpoint (default: http://localhost:7764)
- `--api-key`: Authentication key (default: test-key-1234567890)
- `--min, -m`: Minimum value (default: 0)
- `--max, -M`: Maximum value (default: 100)
- `--count, -c`: Number of integers to generate (default: 1)

## Implementation

Requests 8 bytes of quantum random data from the gateway, converts to u64, and maps to the desired range using modulo operation. Each request generates a fresh random number from the quantum source.
