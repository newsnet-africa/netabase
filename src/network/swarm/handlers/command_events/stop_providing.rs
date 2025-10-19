use libp2p::Swarm;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_stop_providing<D: NetabaseDefinitionTrait>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    key: D::Keys,
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
    println!("StopProviding command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Call the libp2p Kademlia API with the converted key
            swarm.behaviour_mut().kad.stop_providing(&record_key);
            println!("StopProviding: Provider registration stopped successfully");
        }
        Err(conversion_error) => {
            println!(
                "Failed to convert key to kad::RecordKey: {:?}",
                conversion_error
            );
        }
    }

    // Note: This command doesn't have a response channel as it's fire-and-forget
}
