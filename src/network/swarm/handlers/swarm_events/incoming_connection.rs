use libp2p::Multiaddr;
use log::debug;
use libp2p::swarm::ConnectionId;
use netabase_store::traits::definition::{NetabaseDefinitionTrait, RecordStoreExt};

pub fn handle_incoming_connection<D: NetabaseDefinitionTrait + RecordStoreExt + Send + Sync + 'static>(
    connection_id: ConnectionId,
    local_addr: Multiaddr,
    send_back_addr: Multiaddr,
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
    // TODO: Implement incoming connection handling
    debug!(
        "Incoming connection: connection_id: {:?}, local_addr: {:?}, send_back_addr: {:?}",
        connection_id, local_addr, send_back_addr
    );
}
