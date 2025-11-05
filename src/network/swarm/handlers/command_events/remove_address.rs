use libp2p::{
    Multiaddr, PeerId, Swarm,
    kad::{Addresses, EntryView, KBucketKey},
};
use log::{debug, info, warn, error};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_remove_address<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    peer: PeerId,
    address: Multiaddr,
    response_channel: Sender<Option<EntryView<KBucketKey<PeerId>, Addresses>>>,
)where
    D: netabase_store::convert::ToIVec,
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
    debug!(
        "RemoveAddress command: peer={:?}, address={:?}",
        peer, address
    );

    // Call the libp2p Kademlia API
    let result = swarm.behaviour_mut().kad.remove_address(&peer, &address);

    // Send the result through the response channel
    if let Err(_) = response_channel.send(result) {
        debug!("Failed to send RemoveAddress response - receiver dropped");
    }
}
