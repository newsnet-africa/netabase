use libp2p::{Swarm, kad::Mode};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_mode<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    response_channel: Sender<Mode>,
) {
    println!("Mode command received");

    // Call the libp2p Kademlia API
    let mode = swarm.behaviour().kad.mode();

    // Send the result through the response channel
    if let Err(_) = response_channel.send(mode) {
        println!("Failed to send Mode response - receiver dropped");
    }
}
