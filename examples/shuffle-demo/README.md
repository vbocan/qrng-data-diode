# Quantum Shuffle Demonstrator

Shuffle items using the Fisher-Yates algorithm with quantum random numbers.

## Usage

```bash
# Shuffle numbers 1-10 (default)
cargo run --release

# Shuffle a deck of cards
cargo run --release -- --cards

# Shuffle custom items
cargo run --release -- apple banana cherry date elderberry

# Shuffle a playlist
cargo run --release -- "Song1" "Song2" "Song3" "Song4" "Song5"
```

## Algorithm

Uses the Fisher-Yates shuffle algorithm, which produces an unbiased permutation of the input items. Each possible permutation has equal probability when using true random numbers.

The algorithm:
1. Start from the last item
2. Pick a random item from the remaining unshuffled items
3. Swap them
4. Repeat for all items

## Implementation

Quantum randomness ensures the shuffle is truly unbiased, which is critical for applications in cryptography, gaming, and statistical sampling where predictable patterns could be exploited.
