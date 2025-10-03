use libp2p::core::ConnectedPoint;
use libp2p::swarm::ConnectionId;
use libp2p::{Multiaddr, PeerId};
use netabase_store::traits::NetabaseSchema;
use std::time::Duration;

pub fn handle_connection_established<S: NetabaseSchema>(
    peer_id: PeerId,
    connection_id: ConnectionId,
    endpoint: ConnectedPoint,
    num_established: std::num::NonZero<u32>,
    concurrent_dial_errors: Option<Vec<(Multiaddr, libp2p::TransportError<std::io::Error>)>>,
    established_in: Duration,
) {
    // TODO: Implement connection established handling
    println!(
        "Connection established with peer: {:?}, connection_id: {:?}, endpoint: {:?}, num_established: {:?}, established_in: {:?}",
        peer_id, connection_id, endpoint, num_established, established_in
    );

    if let Some(errors) = &concurrent_dial_errors {
        println!("Concurrent dial errors: {:?}", errors);
    }
}
