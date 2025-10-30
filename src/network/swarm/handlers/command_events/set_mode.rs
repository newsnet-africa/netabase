use libp2p::{Swarm, kad::Mode};
use netabase_store::traits::definition::NetabaseDefinitionTrait;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_set_mode<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    mode: Option<Mode>,
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
    println!("SetMode command: mode={:?}", mode);

    // Call the libp2p Kademlia API
    // Access kad through helper method which works whether paxos is enabled or not
    if let Some(kad) = swarm.behaviour_mut().kad_mut() {
        kad.set_mode(mode);
        println!("SetMode: Mode updated successfully");
    } else {
        println!("SetMode: Kad behavior not available");
    }

    // Note: This command doesn't have a response channel as it's fire-and-forget
}
