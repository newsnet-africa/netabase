//! Serde helpers for types that don't implement Serialize/Deserialize

use libp2p::PeerId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Wrapper for PeerId that implements Serialize/Deserialize
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SerializablePeerId(pub PeerId);

impl Serialize for SerializablePeerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_bytes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerializablePeerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        PeerId::from_bytes(&bytes)
            .map(SerializablePeerId)
            .map_err(serde::de::Error::custom)
    }
}

impl From<PeerId> for SerializablePeerId {
    fn from(peer_id: PeerId) -> Self {
        SerializablePeerId(peer_id)
    }
}

impl From<SerializablePeerId> for PeerId {
    fn from(spid: SerializablePeerId) -> Self {
        spid.0
    }
}

impl fmt::Display for SerializablePeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Serialize PeerId as bytes
pub fn serialize_peer_id<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    peer_id.to_bytes().serialize(serializer)
}

/// Deserialize PeerId from bytes
pub fn deserialize_peer_id<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
    PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom)
}

/// Module for serializing HashMap<PeerId, T>
pub mod peer_id_map {
    use super::*;
    use std::collections::HashMap;

    pub fn serialize<S, T>(
        map: &HashMap<PeerId, T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        use serde::ser::SerializeMap;
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            ser_map.serialize_entry(&k.to_bytes(), v)?;
        }
        ser_map.end()
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<HashMap<PeerId, T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let map: HashMap<Vec<u8>, T> = HashMap::deserialize(deserializer)?;
        map.into_iter()
            .map(|(k, v)| {
                PeerId::from_bytes(&k)
                    .map(|peer_id| (peer_id, v))
                    .map_err(serde::de::Error::custom)
            })
            .collect()
    }
}

/// Module for serializing HashSet<PeerId>
pub mod peer_id_set {
    use super::*;
    use std::collections::HashSet;

    pub fn serialize<S>(
        set: &HashSet<PeerId>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut ser_seq = serializer.serialize_seq(Some(set.len()))?;
        for peer_id in set {
            ser_seq.serialize_element(&peer_id.to_bytes())?;
        }
        ser_seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashSet<PeerId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<Vec<u8>> = Vec::deserialize(deserializer)?;
        vec.into_iter()
            .map(|bytes| PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom))
            .collect()
    }
}
