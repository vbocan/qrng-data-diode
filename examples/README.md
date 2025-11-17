# QRNG Gateway Examples

This directory contains example applications demonstrating the QRNG Gateway API. Each example is a standalone Rust application showcasing different use cases for quantum random numbers.

## Basic Examples

### Random Number Generators
- **randint**: Generate random integers in a specified range
- **randfloat**: Generate random floating-point numbers in [0, 1)
- **randbytes**: Generate arbitrary random bytes (hex, base64, or binary)
- **qrand-uuid**: Generate v4 UUIDs using quantum entropy

### Utilities
- **dice-roller**: Simulate dice rolls with standard notation (e.g., 3d6+2)
- **password-generator**: Create secure passwords and passphrases
- **shuffle-demo**: Demonstrate Fisher-Yates shuffling
- **lottery-draw**: Simulate fair lottery draws

## Scientific Computing

### Simulations
- **monte-carlo-pi**: Estimate π using Monte Carlo method
- **random-walk**: Simulate random walks in 1D, 2D, or 3D
- **terrain-generator**: Generate procedural terrain using Perlin noise

### Optimization Algorithms
- **tsp-simulated-annealing**: Solve Traveling Salesman Problem using simulated annealing
- **knapsack-ga**: Solve 0/1 Knapsack Problem using genetic algorithms

## Validation
- **randomness-tests**: Statistical tests for randomness quality (frequency, runs, chi-square)

## Running Examples

Each example is a standalone Cargo project. Navigate to the directory and run:

```bash
cd examples/randint
cargo run --release

cd examples/monte-carlo-pi
cargo run --release -- --samples 10000000
```

See each example's README.md for specific usage instructions and options.

## Common Options

All examples share these common options:
- `--gateway-url`: Gateway endpoint (default: http://localhost:7764)
- `--api-key`: Authentication key (default: test-key-1234567890)

## Prerequisites

You need access to a running QRNG Gateway. There are several options:

### Option 1: Local Gateway (requires QRNG hardware)
If you have a QRNG appliance, run the gateway locally:

```bash
docker-compose up -d qrng-gateway qrng-collector
```

### Option 2: Public Gateway
A public QRNG gateway is available at:
- **URL**: `http://qrng.dataman.ro:7764`
- **API Key**: Contact the service administrator for access

To use the public gateway, pass the URL to examples:
```bash
cargo run --release -- --gateway-url http://qrng.dataman.ro:7764 --api-key YOUR_KEY
```

### Option 3: Mock/Test Gateway
For development and testing without QRNG hardware, you can run the gateway in mock mode (see main project documentation).

**Note**: Without access to a QRNG Gateway (local or remote), these examples will not function.

## Implementation Philosophy

These examples prioritize:
- **Simplicity**: Straightforward code with minimal error handling
- **Clarity**: Clear demonstration of the gateway API
- **Educational Value**: Show practical applications of quantum randomness

Code is intentionally kept simple to focus on API usage patterns rather than production-ready error handling.
