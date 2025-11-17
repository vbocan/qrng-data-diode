# Procedural Terrain Generator

Generate height maps using quantum-seeded Perlin noise for terrain generation.

## Usage

```bash
# Generate 256x256 terrain
cargo run --release > terrain.pgm

# Custom size and scale
cargo run --release -- --width 512 --height 512 --scale 0.03 > terrain.pgm

# Adjust noise frequency
cargo run --release -- --scale 0.1 > terrain.pgm
```

## Output Format

Outputs a PGM (Portable Gray Map) image to stdout. Redirect to a file and view with any image viewer that supports PGM format, or convert to PNG:

```bash
cargo run --release > terrain.pgm
convert terrain.pgm terrain.png  # requires ImageMagick
```

## Options

- `--width, -w`: Image width in pixels (default: 256)
- `--height, -h`: Image height in pixels (default: 256)
- `--scale, -s`: Noise scale/frequency (default: 0.05)

## Implementation

Uses quantum random seed for Perlin noise initialization, ensuring unpredictable and unique terrain patterns. Each run produces different landscapes due to quantum entropy.
