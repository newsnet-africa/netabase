use libp2p::identity::Keypair;
use libp2p::PeerId;
use netabase_store::primitives::Signature;
use netabase_store::node_metadata::{NodePublicKey, PublicNodeData};

#[derive(Clone)]
pub struct NodeIdentity {
    pub keypair: Keypair,
    pub peer_id: PeerId,
}

impl NodeIdentity {
    /// Generate a new random Ed25519 identity
    pub fn generate() -> Self {
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        Self { keypair, peer_id }
    }

    /// Load identity from existing keypair
    pub fn from_keypair(keypair: Keypair) -> Self {
        let peer_id = PeerId::from(keypair.public());
        Self { keypair, peer_id }
    }
    
    /// Get public metadata for this node
    pub fn public_data(&self) -> PublicNodeData {
        PublicNodeData {
            node_id: self.peer_id,
            public_key: self.public_key_struct(),
        }
    }
    
    /// Extract the raw public key bytes for Netabase structs
    pub fn public_key_struct(&self) -> NodePublicKey {
        if let Ok(pk) = self.keypair.public().try_into_ed25519() {
            NodePublicKey(pk.to_bytes())
        } else {
            // Panic or fallback? Netabase assumes Ed25519 32-byte keys for now.
            panic!("Only Ed25519 keys are supported in Netabase currently");
        }
    }

    /// Sign a message using the private key
    pub fn sign(&self, msg: &[u8]) -> Signature {
        let sig_bytes = self.keypair.sign(msg).expect("Signing failed");
        
        // Netabase Signature is [u8; 64] (Ed25519 standard)
        if sig_bytes.len() == 64 {
            let mut arr = [0u8; 64];
            arr.copy_from_slice(&sig_bytes);
            Signature(arr)
        } else {
            panic!("Unexpected signature length: {}", sig_bytes.len());
        }
    }
}
