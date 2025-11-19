# Model Context Protocol (MCP) Integration Guide

## Table of Contents

1. [Overview](#overview)
2. [What is MCP?](#what-is-mcp)
3. [Quick Start](#quick-start)
4. [Available Tools](#available-tools)
5. [Usage Examples](#usage-examples)
6. [Troubleshooting](#troubleshooting)

## Overview

QRNG-DD provides Model Context Protocol (MCP) integration for quantum random number generation, enabling AI agents to seamlessly access quantum entropy for cryptographic operations, simulations, and research experiments.

The public MCP server at **https://qrng-mcp.datamana.ro** provides zero-configuration access to quantum randomness.

### Key Features

- **Zero Configuration**: No setup required - just add the server URL
- **No Authentication**: Public access (MCP server handles Gateway authentication internally)
- **Multiple Data Formats**: Bytes, integers, floats, UUIDs
- **Quality Validation**: Built-in randomness testing via Monte Carlo
- **Standardized Interface**: Works with Claude Desktop, LM Studio, and any MCP client

## Quick Start

### Claude Desktop

1. Open Claude Desktop
2. Go to **Settings** → **Connectors**
3. Click **Add Connector**
4. Enter server URL: `https://qrng-mcp.datamana.ro`
5. Save and restart Claude Desktop

The QRNG tools are now available in your conversations!

### LM Studio

1. Open LM Studio
2. Go to **Integrations** dialog
3. Add MCP server URL: `https://qrng-mcp.datamana.ro`
4. The QRNG tools will be available to your LLM

### Other MCP Clients

Any MCP-compatible client can connect to `https://qrng-mcp.datamana.ro` using the MCP HTTP transport.

## Usage Examples

### Example 1: Generate Cryptographic Key

```
You: Generate a 256-bit cryptographic key using quantum randomness

Claude: I'll use the quantum RNG to generate a secure 256-bit key...
[Uses get_random_bytes tool with length=32, encoding="hex"]

Result: a3f5b2c8d9e1f4a7b6c3d8e9f2a5b4c7d6e1f8a3b2c5d4e7f6a9b8c1d2e3f4a5

This is a cryptographically secure 256-bit key generated from quantum entropy.
```

### Example 2: Roll Dice

```
You: Roll 10 six-sided dice using quantum randomness

Claude: I'll use quantum random numbers to roll 10 dice...
[Uses get_random_integers tool with count=10, min=1, max=6]

Result: [4, 2, 6, 1, 5, 3, 6, 2, 4, 5]
```

### Example 3: Generate UUIDs

```
You: Create 5 quantum-random UUIDs for database records

Claude: I'll generate 5 UUIDs using quantum entropy...
[Uses get_random_uuid tool with count=5]

Result:
- a3b8c2d5-4e6f-4a1b-9c3d-5e7f8a9b0c1d
- f1e2d3c4-b5a6-4978-8869-7a5b4c3d2e1f
- 9b8a7c6d-5e4f-4321-ba09-8c7d6e5f4a3b
- 2c3d4e5f-6a7b-4890-a1b2-c3d4e5f6a7b8
- 8e7f6a5b-4c3d-4210-9a8b-7c6d5e4f3a2b
```

### Example 4: Monte Carlo Simulation

```
You: Test the quality of quantum randomness using Monte Carlo method

Claude: I'll run a Monte Carlo π estimation test...
[Uses validate_randomness tool with iterations=1000000]

Result:
π estimate: 3.141598
Actual π: 3.141592653589793
Error: 0.000005 (0.0002%)
Quality: ★★★★★ Excellent

The quantum RNG is producing high-quality random numbers!
```

### Example 5: Random Floats for Simulation

```
You: Generate 100 random probability values for a Monte Carlo simulation

Claude: I'll generate quantum random probabilities...
[Uses get_random_floats tool with count=100]

Result: [0.742816, 0.391057, 0.629384, 0.185209, ...]
(100 uniform random floats in [0.0, 1.0))
```

## Additional Information

The public MCP server at https://qrng-mcp.datamana.ro is provided for demonstration and testing purposes and has reasonable rate limits to ensure fair access for all users. It may go down for no reason, so for production applications consider deploying your own QRNG infrastructure.

### Data Privacy

- No logging of random data generated
- No tracking or analytics
- Quantum entropy is used immediately and not stored

### Support

- **Documentation**: [Main README](../README.md)
- **Issues**: [GitHub Issues](https://github.com/vbocan/qrng-data-diode/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vbocan/qrng-data-diode/discussions)