use libp2p::PeerId;
use libp2p::core::ConnectedPoint;
use libp2p::swarm::{ConnectionError, ConnectionId};
use netabase_store::traits::NetabaseSchema;

pub fn handle_connection_closed<S: NetabaseSchema>(
    peer_id: PeerId,
    connection_id: ConnectionId,
    endpoint: ConnectedPoint,
    num_established: u32,
    cause: Option<ConnectionError>,
) {
    // TODO: Implement connection closed handling
    println!(
        "Connection closed with peer: {:?}, connection_id: {:?}, endpoint: {:?}, num_established: {:?}",
        peer_id, connection_id, endpoint, num_established
    );

    if let Some(error) = &cause {
        println!("Connection closed due to error: {:?}", error);
    } else {
        println!("Connection closed gracefully");
    }
}
