use libp2p::swarm::NetworkBehaviour;
use netabase_store::{database::NetabaseSledDatabase, traits::NetabaseSchema};
pub mod clone_impl;

#[derive(NetworkBehaviour)]
pub struct NetabaseBehaviour<S: NetabaseSchema> {
    kad: libp2p::kad::Behaviour<NetabaseSledDatabase<S>>,
    identify: libp2p::identify::Behaviour,
    mdns: libp2p::mdns::tokio::Behaviour,
    connection_limit: libp2p::connection_limits::Behaviour,
}
