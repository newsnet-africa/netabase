//! Cryptographic types for capability and entry signing
//!
//! This module provides wrapper types for various signatures used in the
//! capability system and entry verification.

use serde::{Deserialize, Serialize};

/// Signature proving capability delegation authority
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilitySignature(pub Vec<u8>);

impl CapabilitySignature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Verify signature (placeholder for actual crypto implementation)
    pub fn verify(&self, _message: &[u8], _public_key: &[u8]) -> bool {
        // TODO: Implement actual signature verification
        // This would use ed25519, secp256k1, or similar
        true
    }
}

/// Signature proving ownership of a subscription namespace
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionRoomSignature(pub Vec<u8>);

impl SubscriptionRoomSignature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify(&self, _message: &[u8], _public_key: &NamespacePublicKey) -> bool {
        // TODO: Implement actual signature verification
        true
    }
}

/// Signature over entry content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntrySignature(pub Vec<u8>);

impl EntrySignature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify(&self, _message: &[u8], _public_key: &[u8]) -> bool {
        // TODO: Implement actual signature verification
        true
    }
}

/// Namespace public key (Meadowcap NamespacePublicKey)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespacePublicKey(pub [u8; 32]);

impl NamespacePublicKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Verify a namespace signature against this public key
    pub fn verify_namespace_signature(
        &self,
        message: &[u8],
        signature: &NamespaceSignature,
    ) -> bool {
        signature.verify(message, self)
    }
}

/// Namespace signature from the namespace authority
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespaceSignature(pub Vec<u8>);

impl NamespaceSignature {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify(&self, _message: &[u8], _namespace_key: &NamespacePublicKey) -> bool {
        // TODO: Implement actual signature verification
        true
    }
}

/// Namespace secret key for signing
#[derive(Debug, Clone)]
pub struct NamespaceSecretKey(pub [u8; 32]);

impl NamespaceSecretKey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Sign a message with this secret key
    pub fn sign(&self, _message: &[u8]) -> NamespaceSignature {
        // TODO: Implement actual signing
        NamespaceSignature(vec![0; 64])
    }

    /// Derive the public key from this secret key
    pub fn public_key(&self) -> NamespacePublicKey {
        // TODO: Implement actual public key derivation
        NamespacePublicKey([0; 32])
    }
}
