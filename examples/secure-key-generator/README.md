# Secure Key Generator

A simple example demonstrating how to generate cryptographic keys using quantum random data from the QRNG Gateway.

## Features

- Generate AES-256, AES-128, or custom-length keys
- Output in hex or base64 format
- Uses true quantum randomness from the gateway

## Usage

```bash
# Generate a default AES-256 key (32 bytes) in hex format
cargo run

# Generate an AES-128 key (16 bytes) in base64 format
cargo run -- --key-size 16 --format base64

# Generate a custom 64-byte key
cargo run -- --key-size 64

# Specify gateway URL and API key
cargo run -- --gateway-url http://localhost:7764 --api-key your-api-key-here
```

## Configuration

The tool can be configured via command-line arguments:
- `--gateway-url`: Gateway API endpoint (default: http://localhost:7764)
- `--api-key`: API key for authentication (default: test-key-1234567890)
- `--key-size`: Key size in bytes (default: 32 for AES-256)
- `--format`: Output format - hex or base64 (default: hex)

## Why Quantum Random Keys?

Cryptographic key strength depends on entropy quality. Traditional pseudo-random number generators (PRNGs) are deterministic and potentially vulnerable to prediction attacks. Quantum random number generators provide true randomness from quantum processes, ensuring maximum entropy for cryptographic keys.
