use libp2p::{Multiaddr, PeerId};
use netabase_store::traits::definition::NetabaseDefinitionTrait;

pub fn handle_new_external_addr_of_peer<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    peer_id: PeerId,
    address: Multiaddr,
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,{
    // TODO: Implement new external address of peer handling
    println!(
        "New external address discovered for peer: {:?}, address: {:?}",
        peer_id, address
    );

    // This event is fired when we discover a new external address
    // that can be used to reach a specific peer from the outside network
}
