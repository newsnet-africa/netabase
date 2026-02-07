//! Enumeration Capabilities
//!
//! An enumeration capability certifies read access to arbitrary SubspaceIds
//! at some unspecified Path. This is needed for resolving "awkward" cases
//! in Private Area Intersection (PAI).
//!
//! From the Meadowcap spec:
//! "An enumeration capability is an unforgeable token with two types of semantics:
//!  - each enumeration capability must have a single receiver
//!  - it must have a single granted namespace"

use serde::{Deserialize, Serialize};

use crate::data::util::encryption::{NamespacePublicKey, NamespaceSignature};

use super::meadowcap::{UserPublicKey, UserSignature};

/// Enumeration capability for resolving awkward PAI cases
///
/// This capability proves that a user is allowed to learn about arbitrary
/// SubspaceIds in use within a namespace, without revealing specific Paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McEnumerationCapability {
    /// The namespace for which this grants enumeration access
    pub namespace_key: NamespacePublicKey,
    /// The user to whom this initially grants access
    pub user_key: UserPublicKey,
    /// Authorization of the user_key by the namespace_key
    pub initial_authorisation: NamespaceSignature,
    /// Successive authorizations of new UserPublicKeys
    pub delegations: Vec<(UserPublicKey, UserSignature)>,
}

impl McEnumerationCapability {
    /// Create a new root enumeration capability
    pub fn new_root(
        namespace_key: NamespacePublicKey,
        user_key: UserPublicKey,
        initial_authorisation: NamespaceSignature,
    ) -> Self {
        Self {
            namespace_key,
            user_key,
            initial_authorisation,
            delegations: Vec::new(),
        }
    }

    /// Get the receiver of this capability
    pub fn receiver(&self) -> &UserPublicKey {
        self.delegations
            .last()
            .map(|(user, _)| user)
            .unwrap_or(&self.user_key)
    }

    /// Get the granted namespace
    pub fn granted_namespace(&self) -> &NamespacePublicKey {
        &self.namespace_key
    }

    /// Check if this capability is valid
    pub fn is_valid(&self) -> bool {
        // Verify initial authorization
        // The initial authorization signs: 0x04 || user_key
        let init_message = self.compute_initial_message();
        if !self.initial_authorisation.verify(&init_message, &self.namespace_key) {
            return false;
        }

        if self.delegations.is_empty() {
            return true;
        }

        // Verify delegation chain
        let mut prev_receiver = self.user_key.clone();
        
        for (i, (new_user, new_signature)) in self.delegations.iter().enumerate() {
            let handover = self.compute_handover(i, new_user);
            
            if !new_signature.verify(&handover, &prev_receiver) {
                return false;
            }
            
            prev_receiver = new_user.clone();
        }

        true
    }

    /// Compute the message for initial authorization
    fn compute_initial_message(&self) -> Vec<u8> {
        let mut message = Vec::new();
        message.push(0x04); // Enumeration capability marker
        message.extend_from_slice(&self.user_key.to_bytes());
        message
    }

    /// Compute the handover message for a delegation
    fn compute_handover(&self, delegation_index: usize, new_user: &UserPublicKey) -> Vec<u8> {
        let mut handover = Vec::new();

        if delegation_index == 0 {
            // First delegation: include initial authorization
            handover.extend_from_slice(self.initial_authorisation.as_bytes());
        } else {
            // Subsequent delegations: include previous signature
            let (_, prev_sig) = &self.delegations[delegation_index - 1];
            handover.extend_from_slice(prev_sig.as_bytes());
        }

        handover.extend_from_slice(&new_user.to_bytes());
        handover
    }

    /// Delegate this capability to another user
    pub fn delegate(
        &self,
        new_user: UserPublicKey,
        signature: UserSignature,
    ) -> Self {
        let mut new_cap = self.clone();
        new_cap.delegations.push((new_user, signature));
        new_cap
    }

    /// Compute hash for revocation/tracking
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(b"enumeration_capability");
        hasher.update(self.namespace_key.as_bytes());
        hasher.update(&self.receiver().to_bytes());
        hasher.finalize().into()
    }
}

/// Encoded enumeration capability (omits namespace_key for transmission)
///
/// Used during PAI when the namespace is already established context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedEnumerationCapability {
    /// The user to whom this initially grants access
    pub user_key: UserPublicKey,
    /// Authorization (opaque bytes, namespace omitted from context)
    pub initial_authorisation_bytes: Vec<u8>,
    /// Successive authorizations
    pub delegations: Vec<(UserPublicKey, Vec<u8>)>,
}

impl EncodedEnumerationCapability {
    /// Encode a capability, omitting the namespace (from context)
    pub fn encode(cap: &McEnumerationCapability) -> Self {
        Self {
            user_key: cap.user_key.clone(),
            initial_authorisation_bytes: cap.initial_authorisation.as_bytes().to_vec(),
            delegations: cap.delegations
                .iter()
                .map(|(user, sig)| (user.clone(), sig.as_bytes().to_vec()))
                .collect(),
        }
    }

    /// Decode a capability given the namespace context
    pub fn decode(self, namespace_key: NamespacePublicKey) -> McEnumerationCapability {
        McEnumerationCapability {
            namespace_key,
            user_key: self.user_key,
            initial_authorisation: NamespaceSignature::new(self.initial_authorisation_bytes),
            delegations: self.delegations
                .into_iter()
                .map(|(user, sig_bytes)| (user, UserSignature::new(sig_bytes)))
                .collect(),
        }
    }

    /// Get the receiver without full decoding
    pub fn receiver(&self) -> &UserPublicKey {
        self.delegations
            .last()
            .map(|(user, _)| user)
            .unwrap_or(&self.user_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_enumeration_capability_root() {
        let namespace_key = NamespacePublicKey::new([1u8; 32]);
        let user_key = PeerId::random();
        let sig = NamespaceSignature::new(vec![0; 64]);

        let cap = McEnumerationCapability::new_root(namespace_key.clone(), user_key.clone(), sig);

        assert_eq!(cap.receiver(), &user_key);
        assert_eq!(cap.granted_namespace(), &namespace_key);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let namespace_key = NamespacePublicKey::new([2u8; 32]);
        let user_key = PeerId::random();
        let sig = NamespaceSignature::new(vec![1; 64]);

        let cap = McEnumerationCapability::new_root(namespace_key.clone(), user_key.clone(), sig);
        let encoded = EncodedEnumerationCapability::encode(&cap);
        let decoded = encoded.decode(namespace_key.clone());

        assert_eq!(decoded.namespace_key, cap.namespace_key);
        assert_eq!(decoded.user_key, cap.user_key);
    }
}
