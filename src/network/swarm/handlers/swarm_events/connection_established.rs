use libp2p::core::ConnectedPoint;
use libp2p::swarm::ConnectionId;
use libp2p::{Multiaddr, PeerId};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use std::time::Duration;

pub fn handle_connection_established<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    _peer_id: PeerId,
    _connection_id: ConnectionId,
    _endpoint: ConnectedPoint,
    _num_established: std::num::NonZero<u32>,
    _concurrent_dial_errors: Option<Vec<(Multiaddr, libp2p::TransportError<std::io::Error>)>>,
    _established_in: Duration,
) {
    // Silent - connection events are handled internally
}
