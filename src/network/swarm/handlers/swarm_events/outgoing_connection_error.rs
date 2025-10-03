use libp2p::PeerId;
use libp2p::swarm::{ConnectionId, DialError};
use netabase_store::traits::NetabaseSchema;

pub fn handle_outgoing_connection_error<S: NetabaseSchema>(
    connection_id: ConnectionId,
    peer_id: Option<PeerId>,
    error: DialError,
) {
    // TODO: Implement outgoing connection error handling
    println!(
        "Outgoing connection error: connection_id: {:?}, error: {:?}",
        connection_id, error
    );

    if let Some(peer) = &peer_id {
        println!("Error occurred while connecting to peer: {:?}", peer);
    } else {
        println!("Error occurred while dialing unknown peer");
    }
}
