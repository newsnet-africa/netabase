use libp2p::{Multiaddr, PeerId};
use netabase_store::traits::NetabaseSchema;

pub fn handle_new_external_addr_of_peer<S: NetabaseSchema>(peer_id: PeerId, address: Multiaddr) {
    // TODO: Implement new external address of peer handling
    println!(
        "New external address discovered for peer: {:?}, address: {:?}",
        peer_id, address
    );

    // This event is fired when we discover a new external address
    // that can be used to reach a specific peer from the outside network
}
