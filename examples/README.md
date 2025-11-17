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

You need access to a running QRNG Gateway. Choose **one** of the following options based on your setup:

### Option 1: Local Gateway with Quantis Appliance (Recommended for Production)

If you have access to a Quantis QRNG appliance on your network, run both the collector and gateway locally using Docker Compose:

```bash
# From the project root directory
docker-compose up -d qrng-gateway qrng-collector
```

This starts:
- **qrng-collector**: Fetches quantum random data from your Quantis appliance and pushes it to the gateway
- **qrng-gateway**: Serves the random data via REST API on `localhost:7764`

The collector will automatically connect to the Quantis appliance URL configured in the docker-compose.yml (default: `https://random.cs.upt.ro/api/2.0/streambytes`). Update this URL to match your appliance's address if needed.

Once running, examples use the default settings (no additional arguments needed):
```bash
cargo run --release
```

### Option 2: Public Gateway (No Hardware Required)

If you **do not** have access to a Quantis QRNG appliance, you can use the public QRNG gateway:

- **URL**: `https://qrng.dataman.ro`
- **API Key**: `test-key-1234567890`

This public gateway provides genuine quantum random numbers from a remote Quantis appliance. To use it, specify the gateway URL when running examples:

```bash
cargo run --release -- --gateway-url https://qrng.dataman.ro
```

Since the default API key (`test-key-1234567890`) matches the public gateway's key, you only need to specify the gateway URL.

**Note**: The public gateway is suitable for testing, development, and educational purposes. For production use with high-volume requirements, consider setting up your own local gateway with a Quantis appliance.

**Note**: Without access to a QRNG Gateway (local or remote), these examples will not function.

## Implementation Philosophy

These examples prioritize:
- **Simplicity**: Straightforward code with minimal error handling
- **Clarity**: Clear demonstration of the gateway API
- **Educational Value**: Show practical applications of quantum randomness

Code is intentionally kept simple to focus on API usage patterns rather than production-ready error handling.
