use libp2p::Swarm;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_remove_record<D: NetabaseDefinitionTrait>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    key: D::Keys,
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
    println!("RemoveRecord command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Use kad_mut() helper - works whether paxos is enabled or not
            if let Some(kad) = swarm.behaviour_mut().kad_mut() {
                // Call the libp2p Kademlia API with the converted key
                kad.remove_record(&record_key);
                println!("RemoveRecord: Record removal requested successfully");
            } else {
                println!("Kademlia is not available - cannot remove record");
            }
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
