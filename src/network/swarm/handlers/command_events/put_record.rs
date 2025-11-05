use libp2p::{Swarm, kad};
use log::{debug, info, warn, error};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_put_record<D: NetabaseDefinitionTrait + RecordStoreExt>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    record: D,
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
    // Convert NetabaseSchema to libp2p::kad::Record
    match record.to_record() {
        Ok(kad_record) => {
            // Call the libp2p Kademlia API with the converted record
            match swarm
                .behaviour_mut()
                .kad
                .put_record(kad_record, kad::Quorum::One)
            {
                Ok(query_id) => {
                    // Store the response channel for when the query completes
                    super::super::swarm_events::behaviour::kad::store_query_response_channel(
                        query_id,
                        response_channel,
                    );
                }
                Err(store_error) => {
                    // Send the error immediately
                    let _ = response_channel.send(Err(store_error));
                }
            }
        }
        Err(conversion_error) => {
            warn!(
                "Failed to convert record to kad::Record: {:?}",
                conversion_error
            );
            // Convert NetabaseError to store::Error (this is a simplification)
            let _ = response_channel.send(Err(kad::store::Error::MaxRecords));
        }
    }
}
