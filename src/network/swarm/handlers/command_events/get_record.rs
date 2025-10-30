use libp2p::{Swarm, kad::QueryResult};
use netabase_store::{
    definition::NetabaseDefinitionTraitKey, traits::definition::NetabaseDefinitionTrait,
};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_get_record<D: NetabaseDefinitionTrait>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    key: D::Keys,
    response_channel: Sender<QueryResult>,
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
    println!("GetRecord command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Use kad_mut() helper - works whether paxos is enabled or not
            if let Some(kad) = swarm.behaviour_mut().kad_mut() {
                // Call the libp2p Kademlia API with the converted key
                let query_id = kad.get_record(record_key);

                // Store the response channel for when the query completes
                super::super::swarm_events::behaviour::kad::store_query_response_channel(
                    query_id,
                    response_channel,
                );

                println!(
                    "GetRecord: Query started with ID {:?}, response will be sent via event loop",
                    query_id
                );
            } else {
                println!("Kademlia is not available");
                // Send an error response indicating kad is not available
                let error_result = QueryResult::GetRecord(Err(libp2p::kad::GetRecordError::NotFound {
                    key: record_key,
                    closest_peers: vec![],
                }));
                if let Err(_) = response_channel.send(error_result) {
                    println!("Failed to send GetRecord kad-unavailable error response - receiver dropped");
                }
            }
        }
        Err(conversion_error) => {
            println!(
                "Failed to convert key to kad::RecordKey: {:?}",
                conversion_error
            );
            // Send an error response using a placeholder key
            let error_result = QueryResult::GetRecord(Err(libp2p::kad::GetRecordError::NotFound {
                key: libp2p::kad::RecordKey::new(&b"conversion_error"),
                closest_peers: vec![],
            }));
            if let Err(_) = response_channel.send(error_result) {
                println!("Failed to send GetRecord conversion error response - receiver dropped");
            }
        }
    }
}
