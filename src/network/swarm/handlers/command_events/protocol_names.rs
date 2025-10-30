use libp2p::{StreamProtocol, Swarm};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_protocol_names<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    response_channel: Sender<StreamProtocol>,
)where
    D: netabase_store::convert::ToIVec + serde::Serialize + for<'de> serde::Deserialize<'de>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send, {
    println!("ProtocolNames command received");

    // Use kad_mut() helper - works whether paxos is enabled or not
    // Note: We use kad_mut() even though protocol_names() doesn't need mutability for consistency
    if let Some(kad) = swarm.behaviour_mut().kad_mut() {
        // Call the libp2p Kademlia API
        let protocol_names = kad.protocol_names();

        // Send the first protocol name (Kademlia typically has one main protocol)
        if let Some(protocol) = protocol_names.first() {
            if let Err(_) = response_channel.send(protocol.clone()) {
                println!("Failed to send ProtocolNames response - receiver dropped");
            }
        } else {
            println!("No protocol names available");
        }
    } else {
        println!("Kademlia is not available - cannot retrieve protocol names");
        // No default protocol to send when kad is not available
    }
}
