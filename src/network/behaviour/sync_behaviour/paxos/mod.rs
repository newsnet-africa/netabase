pub mod log_entry;
pub mod state;

use libp2p::{kad::store::RecordStore, swarm::NetworkBehaviour};

#[derive(NetworkBehaviour)]
pub struct PaxosBehaviour<S: RecordStore> {
    kad: libp2p::kad::Behaviour<S>,
    gossip: libp2p::gossipsub::Behaviour,
}
