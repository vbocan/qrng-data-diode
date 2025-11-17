# Random Float Generator

Generate random floating-point numbers in the range [0, 1) using quantum entropy.

## Usage

```bash
# Generate a single random float
cargo run --release

# Generate 10 random floats
cargo run --release -- --count 10
```

## Options

- `--gateway-url`: Gateway endpoint (default: http://localhost:7764)
- `--api-key`: Authentication key (default: test-key-1234567890)
- `--count, -c`: Number of floats to generate (default: 1)

## Implementation

Requests 8 bytes of quantum random data, converts to u64, then normalizes to [0, 1) by dividing by 2^64. This provides uniform distribution across the entire floating-point range with quantum randomness.
