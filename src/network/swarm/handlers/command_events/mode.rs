use libp2p::{Swarm, kad::Mode};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_mode<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    response_channel: Sender<Mode>,
) where
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    println!("Mode command received");

    // Use kad_mut() helper - works whether paxos is enabled or not
    // Note: We use kad_mut() even though mode() doesn't need mutability for consistency
    if let Some(kad) = swarm.behaviour_mut().kad_mut() {
        // Call the libp2p Kademlia API
        let mode = kad.mode();

        // Send the result through the response channel
        if let Err(_) = response_channel.send(mode) {
            println!("Failed to send Mode response - receiver dropped");
        }
    } else {
        println!("Kademlia is not available");
        // Send default Mode::Client when kad is not available
        if let Err(_) = response_channel.send(Mode::Client) {
            println!("Failed to send Mode kad-unavailable response - receiver dropped");
        }
    }
}
