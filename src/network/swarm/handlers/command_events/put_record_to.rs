use libp2p::{
    PeerId, Swarm,
    kad::{QueryResult, Quorum},
};
use log::{debug, info, warn, error};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use tokio::sync::oneshot::Sender;

use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_put_record_to<D: NetabaseDefinitionTrait + RecordStoreExt>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    record: D,
    peers: Vec<PeerId>,
    quorum: Quorum,
    response_channel: Sender<QueryResult>,
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
        "PutRecordTo command: record={:?}, peers={:?}, quorum={:?}",
        record, peers, quorum
    );

    // Convert NetabaseSchema to libp2p::kad::Record
    match record.to_record() {
        Ok(kad_record) => {
            // Call the libp2p Kademlia API with the converted record
            let query_id =
                swarm
                    .behaviour_mut()
                    .kad
                    .put_record_to(kad_record, peers.into_iter(), quorum);

            // Store the response channel for when the query completes
            super::super::swarm_events::behaviour::kad::store_query_response_channel(
                query_id,
                response_channel,
            );

            debug!(
                "PutRecordTo: Query started with ID {:?}, response will be sent via event loop",
                query_id
            );
        }
        Err(conversion_error) => {
            debug!(
                "Failed to convert record to kad::Record: {:?}",
                conversion_error
            );
            // Send an error response using a placeholder key
            let error_result = QueryResult::PutRecord(Err(libp2p::kad::PutRecordError::Timeout {
                key: libp2p::kad::RecordKey::new(&b"conversion_error"),
                success: vec![],
                quorum: match quorum {
                    Quorum::One => std::num::NonZero::new(1).unwrap(),
                    Quorum::Majority => std::num::NonZero::new(2).unwrap(),
                    Quorum::All => std::num::NonZero::new(3).unwrap(),
                    Quorum::N(n) => n,
                },
            }));
            if let Err(_) = response_channel.send(error_result) {
                debug!("Failed to send PutRecordTo conversion error response - receiver dropped");
            }
        }
    }
}
