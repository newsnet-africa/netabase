use libp2p::PeerId;
use libp2p::core::ConnectedPoint;
use libp2p::swarm::{ConnectionError, ConnectionId};
use netabase_store::traits::definition::NetabaseDefinition;

pub fn handle_connection_closed<D: NetabaseDefinition + Send + Sync + 'static>(
    _peer_id: PeerId,
    _connection_id: ConnectionId,
    _endpoint: ConnectedPoint,
    _num_established: u32,
    _cause: Option<ConnectionError>,
) {
    // Silent - connection closures are handled internally
}
