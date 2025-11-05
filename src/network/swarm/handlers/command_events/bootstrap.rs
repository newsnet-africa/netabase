use libp2p::{
    Swarm,
    kad::{NoKnownPeers, QueryResult},
};
use log::{debug, info, warn, error};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};
use tokio::sync::oneshot::Sender;

use super::super::swarm_events::behaviour::kad::store_query_response_channel;
use crate::network::behaviour::NetabaseBehaviour;

pub(crate) fn handle_bootstrap<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    swarm: &mut Swarm<NetabaseBehaviour<D>>,
    response_channel: Sender<Result<QueryResult, NoKnownPeers>>,
) where
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
{
    debug!("Bootstrap command received");

    // Call the libp2p Kademlia API
    match swarm.behaviour_mut().kad.bootstrap() {
        Ok(query_id) => {
            // Store the response channel for when the query completes
            store_query_response_channel(query_id, response_channel);
            debug!(
                "Bootstrap: Query started with ID {:?}, response will be sent via event loop",
                query_id
            );
        }
        Err(no_known_peers) => {
            // Send the error immediately
            if let Err(_) = response_channel.send(Err(no_known_peers)) {
                debug!("Failed to send Bootstrap error response - receiver dropped");
            }
        }
    }
}
