//! Entropy mixing for multiple randomness sources
//!
//! Provides algorithms to combine entropy from multiple quantum sources.

use crate::{config::MixingStrategy, Error, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Entropy mixer for combining multiple randomness sources
pub struct EntropyMixer {
    strategy: MixingStrategy,
}

impl EntropyMixer {
    /// Create a new entropy mixer with the specified strategy
    pub fn new(strategy: MixingStrategy) -> Self {
        Self { strategy }
    }

    /// Mix multiple entropy chunks into a single output
    ///
    /// All chunks must have the same length. Returns error if chunks are empty
    /// or have different lengths.
    pub fn mix(&self, chunks: &[Vec<u8>]) -> Result<Vec<u8>> {
        if chunks.is_empty() {
            return Err(Error::Validation("No chunks to mix".to_string()));
        }

        if chunks.len() == 1 {
            return Ok(chunks[0].clone());
        }

        // Verify all chunks have the same length
        let expected_len = chunks[0].len();
        if chunks.iter().any(|chunk| chunk.len() != expected_len) {
            return Err(Error::Validation(
                "All chunks must have the same length for mixing".to_string(),
            ));
        }

        match self.strategy {
            MixingStrategy::None => Ok(chunks[0].clone()),
            MixingStrategy::Xor => Ok(self.xor_mix(chunks)),
            MixingStrategy::Hkdf => self.hkdf_mix(chunks),
        }
    }

    /// XOR all chunks together
    fn xor_mix(&self, chunks: &[Vec<u8>]) -> Vec<u8> {
        let len = chunks[0].len();
        let mut result = vec![0u8; len];

        for chunk in chunks {
            for (i, &byte) in chunk.iter().enumerate() {
                result[i] ^= byte;
            }
        }

        result
    }

    /// Mix using HKDF (HMAC-based Key Derivation Function)
    ///
    /// This provides better mixing properties than simple XOR, especially
    /// if the sources have any correlation or bias.
    fn hkdf_mix(&self, chunks: &[Vec<u8>]) -> Result<Vec<u8>> {
        let len = chunks[0].len();

        // Concatenate all chunks
        let mut input = Vec::new();
        for chunk in chunks {
            input.extend_from_slice(chunk);
        }

        // Use HKDF-Extract: HMAC(salt, input_key_material)
        // We use a fixed salt derived from the number of sources
        let salt = format!("qrng-entropy-mix-{}-sources", chunks.len());
        let mut mac = HmacSha256::new_from_slice(salt.as_bytes())
            .map_err(|e| Error::Crypto(format!("HMAC init failed: {}", e)))?;
        mac.update(&input);
        let prk = mac.finalize().into_bytes();

        // HKDF-Expand: derive output of desired length
        let mut output = Vec::with_capacity(len);
        let mut counter = 1u8;
        let mut t = Vec::new();

        while output.len() < len {
            let mut mac = HmacSha256::new_from_slice(&prk)
                .map_err(|e| Error::Crypto(format!("HMAC init failed: {}", e)))?;
            mac.update(&t);
            mac.update(&[counter]);
            t = mac.finalize().into_bytes().to_vec();
            output.extend_from_slice(&t);
            counter += 1;
        }

        output.truncate(len);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_mixing() {
        let mixer = EntropyMixer::new(MixingStrategy::Xor);

        let chunk1 = vec![0b11110000, 0b10101010];
        let chunk2 = vec![0b00001111, 0b01010101];

        let result = mixer.mix(&[chunk1, chunk2]).unwrap();
        assert_eq!(result, vec![0b11111111, 0b11111111]);
    }

    #[test]
    fn test_xor_three_sources() {
        let mixer = EntropyMixer::new(MixingStrategy::Xor);

        let chunk1 = vec![0xFF, 0x00];
        let chunk2 = vec![0x0F, 0xF0];
        let chunk3 = vec![0xA5, 0x5A];

        let result = mixer.mix(&[chunk1, chunk2, chunk3]).unwrap();
        // 0xFF ^ 0x0F ^ 0xA5 = 0x55
        // 0x00 ^ 0xF0 ^ 0x5A = 0xAA
        assert_eq!(result, vec![0x55, 0xAA]);
    }

    #[test]
    fn test_hkdf_mixing() {
        let mixer = EntropyMixer::new(MixingStrategy::Hkdf);

        let chunk1 = vec![0x01, 0x02, 0x03, 0x04];
        let chunk2 = vec![0x05, 0x06, 0x07, 0x08];

        let result = mixer.mix(&[chunk1.clone(), chunk2.clone()]).unwrap();
        assert_eq!(result.len(), 4);

        // HKDF should produce deterministic output for same input
        let result2 = mixer.mix(&[chunk1, chunk2]).unwrap();
        assert_eq!(result, result2);
    }

    #[test]
    fn test_different_lengths_error() {
        let mixer = EntropyMixer::new(MixingStrategy::Xor);

        let chunk1 = vec![0x01, 0x02];
        let chunk2 = vec![0x03, 0x04, 0x05];

        assert!(mixer.mix(&[chunk1, chunk2]).is_err());
    }

    #[test]
    fn test_single_chunk() {
        let mixer = EntropyMixer::new(MixingStrategy::Xor);
        let chunk = vec![0x01, 0x02, 0x03];

        let result = mixer.mix(&[chunk.clone()]).unwrap();
        assert_eq!(result, chunk);
    }

    #[test]
    fn test_empty_chunks_error() {
        let mixer = EntropyMixer::new(MixingStrategy::Xor);
        assert!(mixer.mix(&[]).is_err());
    }
}
