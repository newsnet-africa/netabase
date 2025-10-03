use libp2p::{StreamProtocol, Swarm};
use netabase_store::traits::NetabaseSchema;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub fn handle_protocol_names<S: NetabaseSchema>(
    swarm: &mut Swarm<NetabaseBehaviour<S>>,
    response_channel: Sender<StreamProtocol>,
) {
    println!("ProtocolNames command received");

    // Call the libp2p Kademlia API
    let protocol_names = swarm.behaviour().kad.protocol_names();

    // Send the first protocol name (Kademlia typically has one main protocol)
    if let Some(protocol) = protocol_names.first() {
        if let Err(_) = response_channel.send(protocol.clone()) {
            println!("Failed to send ProtocolNames response - receiver dropped");
        }
    } else {
        println!("No protocol names available");
    }
}
