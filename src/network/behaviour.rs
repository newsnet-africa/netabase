use libp2p::{
    gossipsub, kad, identify, request_response,
    swarm::NetworkBehaviour,
};
use netabase_store::{
    prelude::NetabaseDefinition,
    databases::redb::libp2p::Libp2pRedbStore,
    traits::registry::definition::redb_definition::RedbDefinition,
};
use serde::{Serialize, Deserialize};

// We will define the Request/Response messages here for now
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetabaseRequest {
    Handshake { schema_hash: [u8; 32], nonce: u64 },
    QueryEnvelope(Vec<u8>), // Serialized QueryEnvelope
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetabaseResponse {
    HandshakeAck { signature: Vec<u8> },
    QueryResponse(Vec<u8>), // Serialized Result
    Error(String),
}

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour<D>
where
    D: NetabaseDefinition + RedbDefinition + Clone + 'static,
    // Constraints required by Libp2pRedbStore and Kademlia
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + Send + Sync + 'static,
    // Add other bounds if needed by Libp2pRedbStore
{
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<Libp2pRedbStore<D>>, 
    pub identify: identify::Behaviour,
    pub request_response: request_response::cbor::Behaviour<NetabaseRequest, NetabaseResponse>,
}

impl<D> NetabaseBehaviour<D>
where
    D: NetabaseDefinition + RedbDefinition + Clone + 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + Send + Sync + 'static,
{
    // Constructor would go here
}