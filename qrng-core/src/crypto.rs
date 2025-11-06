//! Cryptographic utilities for packet signing and verification

use crate::{Error, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// HMAC signer for entropy packets
#[derive(Clone)]
pub struct PacketSigner {
    key: Vec<u8>,
}

impl PacketSigner {
    /// Create a new signer with the given secret key
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }

    /// Generate a random secret key
    pub fn generate_key() -> Vec<u8> {
        use rand::Rng;
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill(&mut key[..]);
        key
    }

    /// Sign data and return HMAC-SHA256 signature
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|e| Error::Crypto(format!("Invalid key length: {}", e)))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    /// Verify HMAC-SHA256 signature using constant-time comparison
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|e| Error::Crypto(format!("Invalid key length: {}", e)))?;
        mac.update(data);
        
        Ok(mac.verify_slice(signature).is_ok())
    }

    /// Sign an entropy packet by computing HMAC over canonical representation
    pub fn sign_packet(&self, packet: &mut crate::protocol::EntropyPacket) -> Result<()> {
        let canonical = self.canonical_packet_bytes(packet)?;
        packet.signature = self.sign(&canonical)?;
        Ok(())
    }

    /// Verify an entropy packet's signature
    pub fn verify_packet(&self, packet: &crate::protocol::EntropyPacket) -> Result<bool> {
        let canonical = self.canonical_packet_bytes(packet)?;
        self.verify(&canonical, &packet.signature)
    }

    /// Create canonical byte representation for signing
    /// Format: version || sequence || data || timestamp_nanos
    fn canonical_packet_bytes(&self, packet: &crate::protocol::EntropyPacket) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(packet.version);
        bytes.extend_from_slice(&packet.sequence.to_be_bytes());
        bytes.extend_from_slice(&packet.data);
        bytes.extend_from_slice(&packet.timestamp.timestamp_nanos_opt()
            .ok_or_else(|| Error::Crypto("Invalid timestamp".to_string()))?
            .to_be_bytes());
        Ok(bytes)
    }
}

/// Encode bytes to hexadecimal string
pub fn encode_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Decode hexadecimal string to bytes
pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| Error::Crypto(format!("Invalid hex: {}", e)))
        })
        .collect()
}

/// Encode bytes to base64 string
pub fn encode_base64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Decode base64 string to bytes
pub fn decode_base64(s: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| Error::Crypto(format!("Invalid base64: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::EntropyPacket;

    #[test]
    fn test_signing() {
        let signer = PacketSigner::new(b"test-secret-key");
        let data = b"hello world";
        let sig = signer.sign(data).unwrap();
        assert!(signer.verify(data, &sig).unwrap());
        assert!(!signer.verify(b"different data", &sig).unwrap());
    }

    #[test]
    fn test_packet_signing() {
        let signer = PacketSigner::new(b"test-secret-key");
        let mut packet = EntropyPacket::new(1, vec![1, 2, 3, 4]);
        signer.sign_packet(&mut packet).unwrap();
        assert!(!packet.signature.is_empty());
        assert!(signer.verify_packet(&packet).unwrap());
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"hello";
        let hex = encode_hex(data);
        assert_eq!(hex, "68656c6c6f");
        let decoded = decode_hex(&hex).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"hello world";
        let b64 = encode_base64(data);
        let decoded = decode_base64(&b64).unwrap();
        assert_eq!(decoded, data);
    }
}
