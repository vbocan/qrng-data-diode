# Quantum-Backed UUID Generator

Generate version 4 UUIDs using quantum random entropy for maximum uniqueness.

## Usage

```bash
# Generate a single UUID
cargo run --release

# Generate 10 UUIDs
cargo run --release -- --count 10

# Generate UUIDs without hyphens
cargo run --release -- --count 5 --no-hyphens
```

## Options

- `--gateway-url`: Gateway endpoint (default: http://localhost:7764)
- `--api-key`: Authentication key (default: test-key-1234567890)
- `--count, -c`: Number of UUIDs to generate (default: 1)
- `--no-hyphens`: Output UUIDs without hyphens

## Implementation

Requests 16 bytes of quantum random data and constructs a valid v4 UUID by setting the appropriate version and variant bits. Quantum randomness ensures global uniqueness even in distributed systems with high UUID generation rates.
