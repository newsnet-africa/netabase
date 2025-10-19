use libp2p::core::ConnectedPoint;
use libp2p::swarm::ConnectionId;
use libp2p::{Multiaddr, PeerId};
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use std::time::Duration;

pub fn handle_connection_established<D: NetabaseDefinitionTrait + Send + Sync + 'static>(
    _peer_id: PeerId,
    _connection_id: ConnectionId,
    _endpoint: ConnectedPoint,
    _num_established: std::num::NonZero<u32>,
    _concurrent_dial_errors: Option<Vec<(Multiaddr, libp2p::TransportError<std::io::Error>)>>,
    _established_in: Duration,
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
    // Silent - connection events are handled internally
}
