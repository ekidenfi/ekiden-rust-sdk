use crate::error::{EkidenError, Result};
use aptos_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    PrivateKey, Signature, SigningKey, Uniform, ValidCryptoMaterialStringExt,
};
use hex;
use sha3::{Digest, Keccak256};

/// Cryptographic utilities for Ekiden SDK
pub struct Crypto;

impl Crypto {
    /// Generate an Ed25519 signature for the given message using the private key
    pub fn sign_message(message: &[u8], private_key: Ed25519PrivateKey) -> anyhow::Result<String> {
        let signature = private_key.sign_arbitrary_message(message);
        signature.to_encoded_string()
    }

    /// Verify an Ed25519 signature
    pub fn verify_signature(message: &[u8], signature: &str, public_key: &str) -> Result<bool> {
        let signature_bytes = hex::decode(signature.strip_prefix("0x").unwrap_or(signature))
            .map_err(|_| EkidenError::crypto("Invalid signature hex format"))?;

        let public_key_bytes = hex::decode(public_key.strip_prefix("0x").unwrap_or(public_key))
            .map_err(|_| EkidenError::crypto("Invalid public key hex format"))?;

        if signature_bytes.len() != 64 {
            return Err(EkidenError::crypto("Signature must be 64 bytes"));
        }

        if public_key_bytes.len() != 32 {
            return Err(EkidenError::crypto("Public key must be 32 bytes"));
        }

        let signature = Ed25519Signature::try_from(signature_bytes.as_slice())
            .map_err(|_| EkidenError::crypto("Invalid signature format"))?;

        let public_key = Ed25519PublicKey::try_from(public_key_bytes.as_slice())
            .map_err(|_| EkidenError::crypto("Invalid public key format"))?;

        match signature.verify_arbitrary_msg(message, &public_key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generate a private key from a private key hex string
    pub fn private_key_from_hex(private_key_hex: &str) -> Result<Ed25519PrivateKey> {
        let private_key = Ed25519PrivateKey::from_encoded_string(&private_key_hex)
            .map_err(|e| EkidenError::crypto(&format!("Invalid private key hex: {}", e)))?;
        Ok(private_key)
    }

    /// Get the public key from a private key as hex string
    pub fn public_key_from_private_key(private_key: &Ed25519PrivateKey) -> String {
        let public_key = private_key.public_key();
        format!("0x{}", hex::encode(public_key.to_bytes()))
    }

    /// Generate an address from a public key (using Keccak256)
    pub fn generate_address_from_public_key(public_key: &str) -> Result<String> {
        let public_key = public_key.strip_prefix("0x").unwrap_or(public_key);
        let public_key_bytes = hex::decode(public_key)
            .map_err(|_| EkidenError::crypto("Invalid public key hex format"))?;

        if public_key_bytes.len() != 32 {
            return Err(EkidenError::crypto("Public key must be 32 bytes"));
        }

        // Hash the public key using Keccak256
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes);
        let hash = hasher.finalize();

        // Take the last 20 bytes as the address
        let address = &hash[12..];
        Ok(format!("0x{}", hex::encode(address)))
    }

    /// Hash a message using Keccak256
    pub fn keccak256(data: &[u8]) -> Vec<u8> {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Hash a message using Keccak256 and return as hex string
    pub fn keccak256_hex(data: &[u8]) -> String {
        let hash = Self::keccak256(data);
        hex::encode(hash)
    }
}

/// Key pair for signing operations
#[derive(Debug, Clone)]
pub struct KeyPair {
    private_key: Ed25519PrivateKey,
}

impl KeyPair {
    /// Create a new key pair from a private key hex string
    pub fn from_private_key(private_key_hex: &str) -> Result<Self> {
        let private_key = Crypto::private_key_from_hex(private_key_hex)?;
        Ok(Self { private_key })
    }

    /// Generate a random key pair
    pub fn generate() -> Self {
        let private_key = Ed25519PrivateKey::generate_for_testing();
        Self { private_key }
    }

    /// Get the private key as hex string
    pub fn private_key(&self) -> String {
        format!("0x{}", hex::encode(self.private_key.to_bytes()))
    }

    /// Get the public key as hex string
    pub fn public_key(&self) -> String {
        self.private_key.public_key().to_encoded_string().unwrap()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> String {
        let signature = self.private_key.sign_arbitrary_message(message);
        // hex::encode(signature.to_bytes())
        signature.to_encoded_string().unwrap()
    }

    /// Sign the authorization message "AUTHORIZE"
    pub fn sign_authorize(&self) -> String {
        self.sign(b"AUTHORIZE")
    }

    /// Get the private key reference
    pub fn get_private_key(&self) -> &Ed25519PrivateKey {
        &self.private_key
    }
}

/// Utility functions for working with hex strings and addresses
pub mod format {
    use crate::error::{EkidenError, Result};

    /// Ensure a hex string has the "0x" prefix
    pub fn ensure_hex_prefix(hex_str: &str) -> String {
        if hex_str.starts_with("0x") {
            hex_str.to_string()
        } else {
            format!("0x{}", hex_str)
        }
    }

    /// Remove the "0x" prefix from a hex string
    pub fn strip_hex_prefix(hex_str: &str) -> &str {
        hex_str.strip_prefix("0x").unwrap_or(hex_str)
    }

    /// Validate that a string is a valid hex public key
    pub fn validate_public_key(public_key: &str) -> Result<()> {
        let public_key = strip_hex_prefix(public_key);

        if public_key.len() != 64 {
            return Err(EkidenError::validation(
                "Public key must be 64 hex characters (32 bytes)",
            ));
        }

        hex::decode(public_key)
            .map_err(|_| EkidenError::validation("Invalid hex characters in public key"))?;

        Ok(())
    }

    /// Validate that a string is a valid hex signature
    pub fn validate_signature(signature: &str) -> Result<()> {
        let signature = strip_hex_prefix(signature);

        if signature.len() != 128 {
            return Err(EkidenError::validation(
                "Signature must be 128 hex characters (64 bytes)",
            ));
        }

        hex::decode(signature)
            .map_err(|_| EkidenError::validation("Invalid hex characters in signature"))?;

        Ok(())
    }

    /// Normalize a public key (lowercase, with 0x prefix)
    pub fn normalize_public_key(public_key: &str) -> Result<String> {
        validate_public_key(public_key)?;
        Ok(ensure_hex_prefix(
            &strip_hex_prefix(public_key).to_lowercase(),
        ))
    }

    /// Normalize a signature (lowercase, with 0x prefix)
    pub fn normalize_signature(signature: &str) -> Result<String> {
        validate_signature(signature)?;
        Ok(ensure_hex_prefix(
            &strip_hex_prefix(signature).to_lowercase(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_pair_generation() {
        let key_pair = KeyPair::generate();
        let public_key = key_pair.public_key();
        let private_key = key_pair.private_key();

        // Test that keys are valid hex
        assert!(public_key.starts_with("0x"));
        assert_eq!(public_key.len(), 66); // 0x + 64 hex chars
        assert!(private_key.starts_with("0x"));
        assert_eq!(private_key.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_signing_and_verification() {
        let key_pair = KeyPair::generate();
        let message = b"test message";
        let signature = key_pair.sign(message);
        let public_key = key_pair.public_key();

        let is_valid = Crypto::verify_signature(message, &signature, &public_key).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_authorize_signature() {
        let key_pair = KeyPair::generate();
        let signature = key_pair.sign_authorize();
        let public_key = key_pair.public_key();

        let is_valid = Crypto::verify_signature(b"AUTHORIZE", &signature, &public_key).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_hex_prefix_handling() {
        assert_eq!(format::ensure_hex_prefix("123"), "0x123");
        assert_eq!(format::ensure_hex_prefix("0x123"), "0x123");
        assert_eq!(format::strip_hex_prefix("0x123"), "123");
        assert_eq!(format::strip_hex_prefix("123"), "123");
    }
}
