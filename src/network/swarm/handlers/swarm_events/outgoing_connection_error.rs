use libp2p::PeerId;
use log::debug;
use libp2p::swarm::{ConnectionId, DialError};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

pub fn handle_outgoing_connection_error<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    connection_id: ConnectionId,
    peer_id: Option<PeerId>,
    error: DialError,
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
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,{
    // TODO: Implement outgoing connection error handling
    debug!(
        "Outgoing connection error: connection_id: {:?}, error: {:?}",
        connection_id, error
    );

    if let Some(peer) = &peer_id {
        debug!("Error occurred while connecting to peer: {:?}", peer);
    } else {
        debug!("Error occurred while dialing unknown peer");
    }
}
