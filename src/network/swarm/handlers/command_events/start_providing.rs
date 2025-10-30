use libp2p::{Swarm, kad};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_start_providing<D: NetabaseDefinitionTrait>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    key: D::Keys,
    response_channel: Sender<Result<kad::QueryResult, kad::store::Error>>,
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
    println!("StartProviding command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Use kad_mut() helper - works whether paxos is enabled or not
            if let Some(kad) = swarm.behaviour_mut().kad_mut() {
                // Call the libp2p Kademlia API with the converted key
                match kad.start_providing(record_key) {
                    Ok(query_id) => {
                        // Store the response channel for when the query completes
                        super::super::swarm_events::behaviour::kad::store_query_response_channel(
                            query_id,
                            response_channel,
                        );
                        println!(
                            "StartProviding: Query started with ID {:?}, response will be sent via event loop",
                            query_id
                        );
                    }
                    Err(store_error) => {
                        // Send the error immediately
                        if let Err(_) = response_channel.send(Err(store_error)) {
                            println!("Failed to send StartProviding error response - receiver dropped");
                        }
                    }
                }
            } else {
                println!("Kademlia is not available");
                // Send error response indicating kad is not available
                if let Err(_) = response_channel.send(Err(kad::store::Error::MaxRecords)) {
                    println!(
                        "Failed to send StartProviding kad-unavailable error response - receiver dropped"
                    );
                }
            }
        }
        Err(conversion_error) => {
            println!(
                "Failed to convert key to kad::RecordKey: {:?}",
                conversion_error
            );
            // Convert NetabaseError to store::Error (this is a simplification)
            if let Err(_) = response_channel.send(Err(kad::store::Error::MaxRecords)) {
                println!(
                    "Failed to send StartProviding conversion error response - receiver dropped"
                );
            }
        }
    }
}
