use libp2p::Multiaddr;
use libp2p::PeerId;
use libp2p::swarm::{ConnectionId, ListenError};
use netabase_store::traits::definition::NetabaseDefinition;

pub fn handle_incoming_connection_error<D: NetabaseDefinition + Send + Sync + 'static>(
    connection_id: ConnectionId,
    local_addr: Multiaddr,
    send_back_addr: Multiaddr,
    error: ListenError,
    peer_id: Option<PeerId>,
) {
    // TODO: Implement incoming connection error handling
    println!(
        "Incoming connection error: connection_id: {:?}, local_addr: {:?}, send_back_addr: {:?}, error: {:?}",
        connection_id, local_addr, send_back_addr, error
    );

    if let Some(peer) = &peer_id {
        println!("Error occurred with peer: {:?}", peer);
    }
}
