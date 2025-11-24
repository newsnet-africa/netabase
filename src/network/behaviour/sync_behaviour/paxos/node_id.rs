use libp2p::PeerId;
use paxakos::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Newtype wrapper for PeerId to implement paxakos traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(PeerId);

impl NodeId {
    /// Create a new NodeId from a PeerId
    pub fn new(peer_id: PeerId) -> Self {
        Self(peer_id)
    }

    /// Get the inner PeerId
    pub fn peer_id(&self) -> PeerId {
        self.0
    }
}

impl From<PeerId> for NodeId {
    fn from(peer_id: PeerId) -> Self {
        Self(peer_id)
    }
}

impl From<NodeId> for PeerId {
    fn from(node_id: NodeId) -> Self {
        node_id.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Custom Serialize/Deserialize for NodeId since PeerId doesn't implement them
impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.to_bytes())
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = NodeId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a PeerId byte array")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                PeerId::from_bytes(v)
                    .map(NodeId)
                    .map_err(|e| E::custom(format!("invalid PeerId: {}", e)))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = Vec::new();
                while let Some(byte) = seq.next_element()? {
                    bytes.push(byte);
                }
                PeerId::from_bytes(&bytes)
                    .map(NodeId)
                    .map_err(|e| serde::de::Error::custom(format!("invalid PeerId: {}", e)))
            }
        }

        deserializer.deserialize_bytes(BytesVisitor)
    }
}

// NodeId gets Identifier automatically from paxakos's blanket impl
// since it implements Copy + Debug + Eq + Hash + Ord + Send + Sync + Unpin

// Implement NodeInfo for NodeId
impl paxakos::NodeInfo for NodeId {
    type Id = NodeId;

    fn id(&self) -> Self::Id {
        *self
    }
}
