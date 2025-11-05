use libp2p::{Swarm, kad};
use log::debug;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt, NetabaseDefinitionTraitKey};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_start_providing<D: NetabaseDefinitionTrait + RecordStoreExt>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    key: D::Keys,
    response_channel: Sender<Result<kad::QueryResult, kad::store::Error>>,
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
    debug!("StartProviding command: key={:?}", key);

    // Convert NetabaseSchemaKeys to libp2p::kad::RecordKey
    match key.to_record_key() {
        Ok(record_key) => {
            // Call the libp2p Kademlia API with the converted key
            match swarm.behaviour_mut().kad.start_providing(record_key) {
                Ok(query_id) => {
                    // Store the response channel for when the query completes
                    super::super::swarm_events::behaviour::kad::store_query_response_channel(
                        query_id,
                        response_channel,
                    );
                    debug!(
                        "StartProviding: Query started with ID {:?}, response will be sent via event loop",
                        query_id
                    );
                }
                Err(store_error) => {
                    // Send the error immediately
                    if let Err(_) = response_channel.send(Err(store_error)) {
                        debug!("Failed to send StartProviding error response - receiver dropped");
                    }
                }
            }
        }
        Err(conversion_error) => {
            debug!(
                "Failed to convert key to kad::RecordKey: {:?}",
                conversion_error
            );
            // Convert NetabaseError to store::Error (this is a simplification)
            if let Err(_) = response_channel.send(Err(kad::store::Error::MaxRecords)) {
                debug!(
                    "Failed to send StartProviding conversion error response - receiver dropped"
                );
            }
        }
    }
}
