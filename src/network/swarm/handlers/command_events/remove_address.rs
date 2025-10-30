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
    println!(
        "RemoveAddress command: peer={:?}, address={:?}",
        peer, address
    );

    // Use kad_mut() helper - works whether paxos is enabled or not
    if let Some(kad) = swarm.behaviour_mut().kad_mut() {
        // Call the libp2p Kademlia API
        let result = kad.remove_address(&peer, &address);

        // Send the result through the response channel
        if let Err(_) = response_channel.send(result) {
            println!("Failed to send RemoveAddress response - receiver dropped");
        }
    } else {
        println!("Kademlia is not available");
        // Send None to indicate the operation couldn't be performed
        if let Err(_) = response_channel.send(None) {
            println!("Failed to send RemoveAddress kad-unavailable response - receiver dropped");
        }
    }
}
