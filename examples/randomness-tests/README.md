# Randomness Quality Tests

Statistical tests to validate the quality of quantum random data.

## Usage

```bash
# Run tests on 100,000 bytes
cargo run --release

# Test with more samples
cargo run --release -- --samples 1000000

# Quick test
cargo run --release -- --samples 10000
```

## Tests

### Frequency Test (Monobit)
Tests if the proportion of 0s and 1s is approximately equal. Expected ratio: 0.5.

### Runs Test
Counts sequences of consecutive identical bits. Tests for independence between bits.

### Chi-Square Test
Tests uniform distribution of byte values (0-255). Checks if all byte values appear with equal probability.

## Interpretation

Each test reports:
- Test statistic
- Expected value
- PASS/FAIL result

Quantum random data should pass all tests, demonstrating statistical randomness superior to pseudo-random generators.

## Options

- `--samples, -s`: Number of bytes to test (default: 100000)
