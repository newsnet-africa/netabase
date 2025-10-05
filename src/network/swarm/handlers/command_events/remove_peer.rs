use libp2p::{
    PeerId, Swarm,
    kad::{Addresses, EntryView, KBucketKey},
};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_remove_peer<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    peer: PeerId,
    response_channel: Sender<Option<EntryView<KBucketKey<PeerId>, Addresses>>>,
) {
    println!("RemovePeer command: peer={:?}", peer);

    // Call the libp2p Kademlia API
    let result = swarm.behaviour_mut().kad.remove_peer(&peer);

    // Send the result through the response channel
    if let Err(_) = response_channel.send(result) {
        println!("Failed to send RemovePeer response - receiver dropped");
    }
}
