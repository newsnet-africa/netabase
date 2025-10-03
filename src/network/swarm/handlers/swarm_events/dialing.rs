use libp2p::PeerId;
use libp2p::swarm::ConnectionId;
use netabase_store::traits::NetabaseSchema;

pub fn handle_dialing<S: NetabaseSchema>(peer_id: Option<PeerId>, connection_id: ConnectionId) {
    // TODO: Implement dialing handling
    if let Some(peer) = peer_id {
        println!(
            "Dialing peer: {:?}, connection_id: {:?}",
            peer, connection_id
        );
    } else {
        println!("Dialing unknown peer, connection_id: {:?}", connection_id);
    }
}
