//! Cryptographic key types for the handshake protocol
//!
//! Based on X25519 for key exchange and Ed25519 for signatures.

use serde::{Deserialize, Serialize};

/// Ephemeral X25519 public key for key exchange
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EphemeralPublicKey(pub [u8; 32]);

impl EphemeralPublicKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Ephemeral X25519 secret key for key exchange
#[derive(Clone)]
pub struct EphemeralSecretKey(pub [u8; 32]);

impl EphemeralSecretKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Perform X25519 key exchange with a public key
    pub fn diffie_hellman(&self, _public_key: &EphemeralPublicKey) -> SharedSecret {
        // TODO: Implement actual X25519 DH
        // This would use x25519-dalek or similar
        SharedSecret([0u8; 32])
    }
}

/// Ephemeral key pair for handshake
#[derive(Clone)]
pub struct EphemeralKeyPair {
    pub public: EphemeralPublicKey,
    pub secret: EphemeralSecretKey,
}

impl EphemeralKeyPair {
    /// Generate a new random ephemeral key pair
    pub fn generate() -> Self {
        // TODO: Use proper RNG (rand crate)
        Self {
            public: EphemeralPublicKey([0u8; 32]),
            secret: EphemeralSecretKey([0u8; 32]),
        }
    }

    pub fn public_key(&self) -> &EphemeralPublicKey {
        &self.public
    }

    pub fn diffie_hellman(&self, other_public: &EphemeralPublicKey) -> SharedSecret {
        self.secret.diffie_hellman(other_public)
    }
}

/// Static Ed25519 public key for long-term identity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StaticPublicKey(pub [u8; 32]);

impl StaticPublicKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Verify a signature against this public key
    pub fn verify(&self, _message: &[u8], _signature: &StaticSignature) -> bool {
        // TODO: Implement actual Ed25519 verification
        true
    }
}

/// Static Ed25519 secret key for long-term identity
#[derive(Clone)]
pub struct StaticSecretKey(pub [u8; 32]);

impl StaticSecretKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Sign a message with this key
    pub fn sign(&self, _message: &[u8]) -> StaticSignature {
        // TODO: Implement actual Ed25519 signing
        StaticSignature([0u8; 64])
    }

    /// Derive public key from secret key
    pub fn public_key(&self) -> StaticPublicKey {
        // TODO: Implement actual key derivation
        StaticPublicKey([0u8; 32])
    }
}

/// Static key pair for node identity
#[derive(Clone)]
pub struct StaticKeyPair {
    pub public: StaticPublicKey,
    pub secret: StaticSecretKey,
}

impl StaticKeyPair {
    /// Generate a new random static key pair
    pub fn generate() -> Self {
        // TODO: Use proper RNG
        Self {
            public: StaticPublicKey([0u8; 32]),
            secret: StaticSecretKey([0u8; 32]),
        }
    }

    pub fn public_key(&self) -> &StaticPublicKey {
        &self.public
    }

    pub fn sign(&self, message: &[u8]) -> StaticSignature {
        self.secret.sign(message)
    }
}

/// Ed25519 signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticSignature(pub [u8; 64]);

impl StaticSignature {
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

// Manual serde implementation for [u8; 64]
impl serde::Serialize for StaticSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for StaticSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StaticSignature;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("64 bytes")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() != 64 {
                    return Err(E::invalid_length(v.len(), &"64 bytes"));
                }
                let mut arr = [0u8; 64];
                arr.copy_from_slice(v);
                Ok(StaticSignature(arr))
            }
        }

        deserializer.deserialize_bytes(Visitor)
    }
}

/// Shared secret from X25519 key exchange
pub struct SharedSecret(pub [u8; 32]);

impl SharedSecret {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
