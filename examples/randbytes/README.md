# Random Bytes Generator

Generate arbitrary amounts of random bytes using quantum entropy.

## Usage

```bash
# Generate 32 bytes in hex format
cargo run --release

# Generate 1024 bytes in base64 format
cargo run --release -- --bytes 1024 --format base64

# Generate binary data (redirect to file)
cargo run --release -- --bytes 1024 --format binary > random.bin
```

## Options

- `--gateway-url`: Gateway endpoint (default: http://localhost:7764)
- `--api-key`: Authentication key (default: test-key-1234567890)
- `--bytes, -b`: Number of bytes to generate (default: 32)
- `--format`: Output format - hex, base64, or binary (default: hex)

## Implementation

Directly requests the specified number of bytes from the gateway in the desired encoding format. Demonstrates the simplest possible use of the /api/random endpoint.
