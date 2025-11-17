# Monte Carlo π Estimation

Estimate the value of π using Monte Carlo method with quantum random sampling.

## Usage

```bash
# Run with default 1 million samples
cargo run --release

# Run with custom sample count
cargo run --release -- --samples 10000000
```

## Method

The algorithm randomly samples points in a unit square and counts how many fall inside the inscribed unit circle. The ratio of points inside the circle to total points approximates π/4.

For a point (x, y) where both x and y are in [0, 1]:
- If x² + y² ≤ 1, the point is inside the quarter circle
- π ≈ 4 × (points inside circle) / (total points)

## Implementation

Uses quantum random numbers for point coordinates to ensure unbiased sampling. Processes samples in batches to reduce API calls while maintaining statistical properties.

## Output

Shows incremental progress and final statistics including the estimated value of π, absolute and relative error compared to the true value.
