use libp2p::{Multiaddr, PeerId, Swarm, kad::RoutingUpdate};
use netabase_store::traits::definition::NetabaseDefinition;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_add_address<D: NetabaseDefinition + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    peer: PeerId,
    address: Multiaddr,
    response_channel: Sender<RoutingUpdate>,
) {
    println!("AddAddress command: peer={:?}, address={:?}", peer, address);

    // Call the libp2p Kademlia API
    let routing_update = swarm.behaviour_mut().kad.add_address(&peer, address);

    // Send the result through the response channel
    if let Err(_) = response_channel.send(routing_update) {
        println!("Failed to send AddAddress response - receiver dropped");
    }
}
