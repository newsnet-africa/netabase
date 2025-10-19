use libp2p::{
    Multiaddr, PeerId, Swarm,
    kad::{Addresses, EntryView, KBucketKey},
};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_remove_address<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    peer: PeerId,
    address: Multiaddr,
    response_channel: Sender<Option<EntryView<KBucketKey<PeerId>, Addresses>>>,
) {
    println!(
        "RemoveAddress command: peer={:?}, address={:?}",
        peer, address
    );

    // Call the libp2p Kademlia API
    let result = swarm.behaviour_mut().kad.remove_address(&peer, &address);

    // Send the result through the response channel
    if let Err(_) = response_channel.send(result) {
        println!("Failed to send RemoveAddress response - receiver dropped");
    }
}
