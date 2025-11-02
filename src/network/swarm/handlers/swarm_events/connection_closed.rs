use libp2p::PeerId;
use libp2p::core::ConnectedPoint;
use libp2p::swarm::{ConnectionError, ConnectionId};
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

pub fn handle_connection_closed<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    _peer_id: PeerId,
    _connection_id: ConnectionId,
    _endpoint: ConnectedPoint,
    _num_established: u32,
    _cause: Option<ConnectionError>,
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
    // Silent - connection closures are handled internally
}
