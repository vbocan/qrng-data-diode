# Quantum Password Generator

Generate secure passwords and passphrases using quantum random entropy.

## Usage

```bash
# Generate a 16-character password (default)
cargo run --release

# Generate a 24-character password
cargo run --release -- --length 24

# Generate 5 passwords
cargo run --release -- --count 5

# Generate password with only letters and digits
cargo run --release -- --no-symbols

# Generate a passphrase (4 words)
cargo run --release -- --passphrase --length 4

# Generate a 6-word passphrase
cargo run --release -- --passphrase --length 6
```

## Options

- `--length, -l`: Password length or number of words in passphrase (default: 16)
- `--count, -c`: Number of passwords to generate (default: 1)
- `--no-uppercase`: Exclude uppercase letters
- `--no-lowercase`: Exclude lowercase letters
- `--no-digits`: Exclude digits
- `--no-symbols`: Exclude special symbols
- `--passphrase`: Generate word-based passphrase instead of random characters

## Implementation

Uses quantum random bytes to select characters from the enabled character sets or words from a word list. Provides cryptographically secure passwords with maximum entropy.
